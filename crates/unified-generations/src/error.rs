use thiserror::Error;

pub type Result<T> = std::result::Result<T, AgentPlanError>;

#[derive(Debug, Error)]
pub enum AgentPlanError {
    #[error("missing API key")]
    MissingApiKey,

    #[error("could not determine home directory for ~/.arkcli")]
    MissingHome,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("failed to decode response JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("failed to parse Ark CLI config: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("failed to serialize Ark CLI config: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("API returned status {status}: {body}")]
    ApiStatus {
        status: reqwest::StatusCode,
        body: String,
    },

    #[error("API returned code {code}: {message}")]
    ApiCode { code: i64, message: String },

    #[error("TTS returned code {code}: {message}")]
    TtsCode { code: i64, message: String },

    #[error("TTS response contained no audio chunks")]
    EmptyAudio,

    #[error("invalid audio hex payload")]
    InvalidAudioHex,
}
