use serde::{Deserialize, Serialize};

/// Represents a simple text message payload.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextData {
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<bool>,
}

/// Represents a media object (image, video, document) to be sent.
/// You can use either a public link or an ID of a previously uploaded media file.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UseMedia {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

/// Represents a message template to be used.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UseTemplate {
    pub name: String,
    pub language: Language,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<Component>>,
}

/// Specifies the language of the template.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Language {
    pub code: String, // e.g., "en_US", "pt_BR"
}

/// A component of a message template (header, body, buttons).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Component {
    #[serde(rename = "type")]
    pub component_type: String, // "header", "body", "button"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,
    // ... other component fields like sub_type and index can be added if needed.
}

/// A parameter for a template component, allowing for dynamic content.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parameter {
    #[serde(rename = "type")]
    pub parameter_type: String, // "text", "image", "document", etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<UseMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<UseMedia>,
    // ... other parameter types like video, currency, etc. can be added.
}

// NOTE: Interactive messages have a very complex structure.
// For now, we'll stub it out. It can be fully implemented if needed.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interactive {
    // ... fields for interactive messages like lists, buttons, etc.
    pub action: serde_json::Value,
    pub body: TextData,
}
