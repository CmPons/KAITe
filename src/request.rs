use anyhow::Context;
use serde::Serialize;

use crate::response::MessageResponse;

#[derive(Clone, Debug, Serialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl TryFrom<MessageResponse> for Message {
    type Error = anyhow::Error;

    fn try_from(value: MessageResponse) -> Result<Self, Self::Error> {
        let content = value.content.unwrap_or_default();
        let last = content
            .iter()
            .last()
            .context("Response returned empty content?")?;

        Ok(Self {
            role: value.role,
            content: last.text.clone(),
        })
    }
}

impl Message {
    pub fn new(role: &str, content: &str) -> Self {
        Self {
            role: role.to_string(),
            content: content.to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct MessageRequest {
    model: String,
    max_tokens: i32,
    messages: Vec<Message>,
    system: String,
}

impl MessageRequest {
    pub fn new(model: String, max_tokens: i32, messages: Vec<Message>) -> Self {
        Self {
            model: model.to_string(),
            max_tokens,
            messages,
            system: String::new(),
        }
    }
}
