use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmMessage {
    pub role: MessageRole,
    pub content: String,
}

impl LlmMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessageRequest {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<LlmMessage>,
}

impl LlmMessageRequest {
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            max_tokens: 1024,
            messages: vec![LlmMessage::user(prompt)],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessageResponse {
    pub id: String,
    pub role: String,
    pub model: String,
    pub content: Vec<MessageContent>,
    pub stop_reason: Option<String>,
    pub usage: Option<serde_json::Value>,
}

impl LlmMessageResponse {
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|part| match part {
                MessageContent::Text { text } => Some(text.as_str()),
                MessageContent::Thinking { .. } | MessageContent::Other(_) => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    Text {
        text: String,
    },
    Thinking {
        thinking: String,
    },
    #[serde(untagged)]
    Other(serde_json::Value),
}
