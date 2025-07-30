use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::wacraft::models::MessagePayloadBase;

/// Represents the top-level structure of the `settings.json` file.
/// It contains configurations for all external services.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub wacraft: WacraftConfig,
    pub email: EmailConfig,
}

/// Contains the necessary settings to interact with the Wacraft API.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WacraftConfig {
    pub base_url: String,
    pub email: String,
    pub password: String,
    // Tokens are managed dynamically but can be stored for persistence.
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    // Expiration timestamp (Unix epoch) for the access token.
    pub token_expires_at: Option<i64>,
}

/// Contains the settings for the email service (SMTP).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password: String,
    pub from_address: String,
}

/// Details for the action of sending a Wacraft message.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WacraftMessageAction {
    // #[serde(flatten)]
    pub sender_data: MessagePayloadBase,
}

/// Represents a single rule in the `reminders.json` file.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReminderRule {
    pub name: String,
    pub inactive_for_hours: u64,
    pub action: Option<Action>,
}

/// An enum representing the different types of actions that can be taken for a reminder.
/// Using an enum with `#[serde(tag = "type")]` allows for clean parsing of the
/// different action objects in the JSON, which is a robust and safe Rust pattern.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Action {
    #[serde(rename = "wacraft_message")]
    WacraftMessage(WacraftMessageAction),
    #[serde(rename = "email")]
    Email(EmailAction),
    #[serde(rename = "http_request")]
    HttpRequest(HttpRequestAction),
}

/// Details for the action of sending a Wacraft message.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendWhatsAppMessage {
    // Generate from documentation
}

/// Details for the action of sending an email.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailAction {
    pub subject: String,
    /// Path to the email template file (e.g., an HTML file).
    pub template: String,
}

/// Details for the action of making an HTTP request (e.g., to a webhook).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpRequestAction {
    pub method: String, // e.g., "POST", "GET"
    pub url: String,
    #[serde(default)] // Makes the `headers` field optional in the JSON.
    pub headers: HashMap<String, String>,
    /// The body of the request, represented as a flexible JSON value.
    /// This allows for sending arbitrary JSON structures.
    #[serde(default)]
    pub body: serde_json::Value,
}
