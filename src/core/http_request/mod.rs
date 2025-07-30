use crate::config::models::HttpRequestAction;
use crate::core::wacraft::models::Contact;
use anyhow::{Context, Result, anyhow};
use reqwest::{Client, Method};
use serde_json::Value;

/// Sends a generic HTTP request based on a configured action.
/// It replaces placeholders in the URL and body with contact data.
pub async fn send_http_request(action: &HttpRequestAction, contact: &Contact) -> Result<()> {
    let client = Client::new();

    // 1. Replace placeholders in the URL.
    let url = replace_placeholders(&action.url, contact);

    // 2. Determine the HTTP method.
    let method = Method::from_bytes(action.method.to_uppercase().as_bytes())
        .map_err(|_| anyhow!("Invalid HTTP method: '{}'", action.method))?;

    // 3. Build the request.
    let mut request_builder = client.request(method, url);

    // 4. Add headers, replacing placeholders.
    for (key, value) in &action.headers {
        request_builder = request_builder.header(key, replace_placeholders(value, contact));
    }

    // 5. Add JSON body if present, replacing placeholders.
    if action.body != Value::Null {
        let body_str = action.body.to_string();
        let processed_body_str = replace_placeholders(&body_str, contact);
        let processed_body_json: Value = serde_json::from_str(&processed_body_str)
            .context("Failed to parse JSON body after placeholder replacement")?;
        request_builder = request_builder.json(&processed_body_json);
    }

    // 6. Send the request.
    let response = request_builder
        .send()
        .await
        .context("Failed to send HTTP request")?;

    // 7. Check the response status.
    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Could not read response body".to_string());
        anyhow::bail!(
            "HTTP request failed with status {}. Response: {}",
            status,
            body
        );
    }

    Ok(())
}

/// A simple placeholder replacement function.
fn replace_placeholders(template: &str, contact: &Contact) -> String {
    let email = contact.email.as_deref().unwrap_or("");
    template
        .replace("{contact_id}", &contact.id)
        .replace("{contact_name}", &contact.name)
        .replace("{contact_email}", email)
}
