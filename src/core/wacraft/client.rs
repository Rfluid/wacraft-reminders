use crate::config::models::WacraftConfig;
use crate::core::wacraft::models::{
    Conversation, SendWhatsAppMessage, TokenRequest, TokenResponse,
};
use anyhow::{Context, Result, anyhow};
use log::{debug, info};
use reqwest::Client;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

use super::models::{MessagingProductContact, Order};

/// A client for interacting with the Wacraft API, with built-in token management.
#[derive(Debug, Clone)]
pub struct WacraftClient {
    http_client: Client,
    // Use an Arc<RwLock<>> to allow for safe, concurrent access and modification of the config.
    // This is crucial for managing token state across multiple async tasks.
    config: Arc<RwLock<WacraftConfig>>,
}

impl WacraftClient {
    /// Creates a new Wacraft API client.
    pub fn new(config: WacraftConfig) -> Self {
        Self {
            http_client: Client::new(),
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Retrieves a valid access token. It handles token expiration and refreshing automatically.
    pub async fn get_valid_token(&self) -> Result<String> {
        // First, try to get a read lock to check the current token.
        let config_read_guard = self.config.read().await;
        if let (Some(token), Some(expires_at)) = (
            &config_read_guard.access_token,
            config_read_guard.token_expires_at,
        ) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
            // Check if the token is valid for at least another 60 seconds.
            if expires_at > now + 60 {
                debug!("Using existing, valid access token.");
                return Ok(token.clone());
            }
        }
        // Drop the read lock so we can acquire a write lock later if needed.
        drop(config_read_guard);

        info!("Access token is expired or missing. Attempting to refresh.");

        // Acquire a write lock to ensure only one task refreshes the token.
        let mut config_write_guard = self.config.write().await;

        // Re-check the token validity after acquiring the write lock.
        // Another task might have refreshed it while we were waiting for the lock.
        if let (Some(token), Some(expires_at)) = (
            &config_write_guard.access_token,
            config_write_guard.token_expires_at,
        ) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
            if expires_at > now + 60 {
                debug!("Token was refreshed by another task. Using new token.");
                return Ok(token.clone());
            }
        }

        // --- Perform Token Refresh ---
        // Try to use the refresh token first.
        let refresh_token_opt = config_write_guard.refresh_token.clone();
        if let Some(refresh_token) = refresh_token_opt {
            debug!("Attempting to refresh token using refresh_token.");
            let request = TokenRequest {
                grant_type: "refresh_token",
                username: None,
                password: None,
                refresh_token: Some(&refresh_token),
            };
            if let Ok(response) = self
                ._get_token(&request, Some(&config_write_guard.base_url))
                .await
            {
                self._update_config_tokens(&mut config_write_guard, response);
                info!("Successfully refreshed access token.");
                return Ok(config_write_guard.access_token.clone().unwrap());
            }
        }

        // If refresh fails or no refresh token exists, fall back to password credentials.
        debug!("Falling back to password credentials for new token.");
        let request = TokenRequest {
            grant_type: "password",
            username: Some(&config_write_guard.email),
            password: Some(&config_write_guard.password),
            refresh_token: None,
        };

        debug!("Executing get token request...");
        let response = self
            ._get_token(&request, Some(&config_write_guard.base_url))
            .await
            .context("Failed to get token with password credentials")?;
        debug!("Successfully executed get token request!");

        self._update_config_tokens(&mut config_write_guard, response);
        info!("Successfully obtained new access token using password.");
        Ok(config_write_guard.access_token.clone().unwrap())
    }

    /// Internal function to request a token from the `/user/oauth/token` endpoint.
    async fn _get_token(
        &self,
        request_body: &TokenRequest<'_>,
        base_url: Option<&str>,
    ) -> Result<TokenResponse> {
        let api_base_url = if let Some(url) = base_url {
            url.to_string()
        } else {
            self.config.read().await.base_url.clone()
        };
        let url = format!("{}/user/oauth/token", api_base_url);
        let response = self
            .http_client
            .post(&url)
            .json(request_body)
            .send()
            .await
            .context("Failed to send token request to Wacraft API")?;
        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await?;
            return Err(anyhow!(
                "Token request failed. Status: {}, Body: {}",
                status,
                error_body
            ));
        }

        response
            .json::<TokenResponse>()
            .await
            .context("Failed to parse token response")
    }

    /// Helper function to update the config with new token data.
    fn _update_config_tokens(&self, config: &mut WacraftConfig, response: TokenResponse) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        config.access_token = Some(response.access_token);
        config.refresh_token = Some(response.refresh_token);
        config.token_expires_at = Some(now + response.expires_in);
        // TODO: Persist the updated config to `settings.json`
    }

    // --- Public API Methods ---

    /// Sends a WhatsApp message using the `/message/whatsapp` endpoint.
    pub async fn send_message(&self, message: &SendWhatsAppMessage) -> Result<()> {
        let payload_json = serde_json::to_string_pretty(&message)?;
        info!("Sending Wacraft message with payload:\n{}", payload_json);

        let token = self.get_valid_token().await?;
        let url = format!("{}/message/whatsapp", self.config.read().await.base_url);

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(token)
            .json(message)
            .send()
            .await
            .context("Failed to send 'send_message' request to Wacraft API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            anyhow::bail!(
                "Failed to send WhatsApp message. Status: {}, Body: {}",
                status,
                error_body
            );
        }

        Ok(())
    }

    /// Fetches a paginated list of conversations.
    pub async fn get_conversations(
        &self,
        limit: u32,
        offset: u32,
        created_at_leq: Option<&str>,
    ) -> Result<Vec<Conversation>> {
        let token = self.get_valid_token().await?;
        let url = format!("{}/message/conversation", self.config.read().await.base_url);

        let mut query_params = vec![("limit", limit.to_string()), ("offset", offset.to_string())];
        if let Some(date) = created_at_leq {
            query_params.push(("created_at_leq", date.to_string()));
        }

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(token)
            .query(&query_params)
            .send()
            .await
            .context("Failed to send 'get_conversations' request to Wacraft API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            anyhow::bail!(
                "Failed to fetch conversations. Status: {}, Body: {}",
                status,
                error_body
            );
        }

        response
            .json::<Vec<Conversation>>()
            .await
            .context("Failed to parse conversations response")
    }

    /// Fetches a paginated list of conversations.
    pub async fn get_conversation_messages(
        &self,
        contact_id: &str,
        limit: u32,
        offset: u32,
        created_at_leq: Option<&str>,
        created_at_order: Option<Order>,
        updated_at_order: Option<Order>,
    ) -> Result<Vec<Conversation>> {
        let token = self.get_valid_token().await?;
        let url = format!(
            "{}/message/conversation/messaging-product-contact/{}",
            self.config.read().await.base_url,
            contact_id
        );

        let mut query_params = vec![("limit", limit.to_string()), ("offset", offset.to_string())];
        if let Some(date) = created_at_leq {
            query_params.push(("created_at_leq", date.to_string()));
        }
        if let Some(order) = created_at_order {
            query_params.push(("created_at", order.to_string()));
        }
        if let Some(order) = updated_at_order {
            query_params.push(("updated_at", order.to_string()));
        }

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(token)
            .query(&query_params)
            .send()
            .await
            .context("Failed to send 'get_conversation_messages' request to Wacraft API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            anyhow::bail!(
                "Failed to fetch conversations. Status: {}, Body: {}",
                status,
                error_body
            );
        }

        response
            .json::<Vec<Conversation>>()
            .await
            .context("Failed to parse conversations response")
    }

    /// Fetches a single messaging product contact by its unique ID.
    pub async fn get_messaging_product_contact_by_id(
        &self,
        contact_id: &str,
    ) -> Result<Option<MessagingProductContact>> {
        let token = self.get_valid_token().await?;
        let url = format!(
            "{}/messaging-product/contact",
            self.config.read().await.base_url
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(token)
            .query(&[("id", contact_id), ("limit", "1"), ("offset", "0")])
            .send()
            .await
            .context(
                "Failed to send 'get_messaging_product_contact_by_id' request to Wacraft API",
            )?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await?;
            anyhow::bail!(
                "Failed to fetch messaging product contact. Status: {}, Body: {}",
                status,
                error_body
            );
        }

        let mut contacts = response
            .json::<Vec<MessagingProductContact>>()
            .await
            .context("Failed to parse messaging product contact response")?;

        let contact = contacts.pop();

        info!("Found contact {:?}", contact);

        // The API returns an array, so we take the first element if it exists.
        Ok(contact)
    }
}
