use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: i32,
    pub output_tokens: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Content {
    pub text: String,
    #[serde(alias = "type")]
    pub text_type: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MessageResponse {
    pub content: Option<Vec<Content>>,
    pub id: String,
    pub model: String,
    pub role: String,
    pub stop_reason: String,
    pub stop_sequence: Option<String>,
    #[serde(alias = "type")]
    pub message_type: String,
}
