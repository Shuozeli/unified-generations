use thiserror::Error;

pub type Result<T> = std::result::Result<T, AgentPlanError>;

#[derive(Debug, Error)]
pub enum AgentPlanError {
    #[error("missing API key")]
    MissingApiKey,

    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("failed to decode response JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API returned status {status}: {body}")]
    ApiStatus {
        status: reqwest::StatusCode,
        body: String,
    },

    #[error("TTS returned code {code}: {message}")]
    TtsCode { code: i64, message: String },

    #[error("TTS response contained no audio chunks")]
    EmptyAudio,
}
