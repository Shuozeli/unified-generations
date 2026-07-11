use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsRequest {
    pub user: TtsUser,
    pub req_params: TtsRequestParams,
}

impl TtsRequest {
    pub fn new(text: impl Into<String>, speaker: impl Into<String>) -> Self {
        Self {
            user: TtsUser {
                uid: "doubao-agent-plan-rs".to_string(),
            },
            req_params: TtsRequestParams {
                text: text.into(),
                speaker: speaker.into(),
                audio_params: AudioParams {
                    format: AudioFormat::Mp3,
                    sample_rate: 24_000,
                    speech_rate: None,
                    loudness_rate: None,
                },
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsUser {
    pub uid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsRequestParams {
    pub text: String,
    pub speaker: String,
    pub audio_params: AudioParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioParams {
    pub format: AudioFormat,
    pub sample_rate: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loudness_rate: Option<i32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Mp3,
    Pcm,
    Wav,
    OggOpus,
}

#[derive(Debug, Clone)]
pub struct TtsResponse {
    pub audio: Vec<u8>,
    pub content_type: Option<String>,
    pub log_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct TtsLine {
    pub code: i64,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub data: Option<String>,
}
