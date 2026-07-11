use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationRequest {
    pub model: String,
    pub prompt: String,
    pub size: ImageSize,
    pub output_format: ImageOutputFormat,
    pub watermark: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
}

impl ImageGenerationRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            model: "doubao-seedream-5.0-lite".to_string(),
            prompt: prompt.into(),
            size: ImageSize::TwoK,
            output_format: ImageOutputFormat::Png,
            watermark: false,
            response_format: Some("url".to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ImageSize {
    #[serde(rename = "1K")]
    OneK,
    #[serde(rename = "2K")]
    TwoK,
    #[serde(rename = "4K")]
    FourK,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageOutputFormat {
    Png,
    Jpeg,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationResponse {
    pub model: String,
    pub created: u64,
    pub data: Vec<GeneratedImage>,
    pub usage: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedImage {
    pub url: Option<String>,
    pub b64_json: Option<String>,
    pub size: Option<String>,
}
