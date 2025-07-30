// Contains the primary data structures for Wacraft API requests and responses.
use super::components::{Interactive, TextData, UseMedia, UseTemplate};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum_macros::{Display, EnumString};

// --- Structures for SENDING messages ---

/// This is the top-level object for the `POST /message/whatsapp` endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendWhatsAppMessage {
    pub to_id: String,
    pub sender_data: MessagePayload,
}

/// Represents the `sender_data` part of the request without the `to` field, which contains the actual message content.
/// It has many optional fields because a message can only be of one type at a time.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MessagePayloadBase {
    pub messaging_product: String,
    pub recipient_type: String,
    #[serde(rename = "type")]
    pub message_type: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<TextData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<UseMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<UseMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<UseMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<UseMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sticker: Option<UseMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<UseTemplate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactive: Option<Interactive>,
}

/// Represents the `sender_data` part of the request
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MessagePayload {
    #[serde(flatten)]
    pub base: MessagePayloadBase,
    pub to: String,
}

// --- Structures for RECEIVING data ---

/// This is based on the `message_entity.Message` definition from the Swagger doc.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Conversation {
    pub id: String,
    pub from_id: Option<String>,
    pub to_id: Option<String>,
    #[serde(rename = "from")]
    pub from_contact: Option<MessagingProductContact>,
    #[serde(rename = "to")]
    pub to_contact: Option<MessagingProductContact>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // --- Add the missing fields here ---
    pub messaging_product_id: String,
    pub receiver_data: Option<Value>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Represents a contact linked to a messaging product (e.g., a WhatsApp user).
/// Based on `messaging_product_entity.MessagingProductContact`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessagingProductContact {
    pub contact_id: Option<String>,
    pub messaging_product_id: Option<String>,
    pub blocked: Option<bool>,
    pub last_read_at: Option<DateTime<Utc>>,

    pub contact: Option<Contact>,
    pub product_details: Option<WhatsAppProductDetails>,

    pub id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents the core contact details. Based on `contact_entity.Contact`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub photo_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// WhatsApp-specific details for a contact.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WhatsAppProductDetails {
    pub wa_id: String,
    pub phone_number: String,
}

/// Represents the `product_data` field within a received message, which holds the
/// actual content from the WhatsApp webhook. Based on `message_model.ReceiverData`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductData {
    pub text: Option<TextData>,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub message_type: String,
    // Add other potential received message types here as needed
    pub image: Option<UseMedia>,
    pub video: Option<UseMedia>,
    pub audio: Option<UseMedia>,
    pub document: Option<UseMedia>,
    pub sticker: Option<UseMedia>,
}

/// Represents the request body for obtaining an OAuth token.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenRequest<'a> {
    pub grant_type: &'a str, // "password" or "refresh_token"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<&'a str>,
}

/// Represents the successful response from the OAuth token endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64, // seconds
    pub token_type: String,
}

#[derive(Debug, EnumString, Display)]
pub enum Order {
    #[strum(serialize = "asc")]
    Asc,
    #[strum(serialize = "desc")]
    Desc,
}
