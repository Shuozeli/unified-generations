use serde::{Deserialize, Serialize};

use crate::error::{AgentPlanError, Result};

pub const DEFAULT_MINIMAX_IMAGE_URL: &str = "https://api.minimax.io/v1/image_generation";
pub const DEFAULT_MINIMAX_TTS_URL: &str = "https://api.minimax.io/v1/t2a_v2";

#[derive(Debug, Clone)]
pub struct MinimaxConfig {
    pub api_key: String,
    pub group_id: Option<String>,
    pub image_url: String,
    pub tts_url: String,
}

impl MinimaxConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            group_id: None,
            image_url: DEFAULT_MINIMAX_IMAGE_URL.to_string(),
            tts_url: DEFAULT_MINIMAX_TTS_URL.to_string(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("MINIMAX_API_KEY")
            .or_else(|_| std::env::var("MINIMAX_TOKEN_PLAN_API_KEY"))
            .map_err(|_| AgentPlanError::MissingApiKey)?;
        let mut config = Self::new(api_key);
        config.apply_env_overrides();
        Ok(config)
    }

    pub fn from_sources(api_key_override: Option<String>) -> Result<Self> {
        let mut config = if let Some(api_key) = api_key_override {
            Self::new(api_key)
        } else {
            Self::from_env()?
        };
        config.apply_env_overrides();
        if config.api_key.trim().is_empty() {
            return Err(AgentPlanError::MissingApiKey);
        }
        Ok(config)
    }

    fn apply_env_overrides(&mut self) {
        if let Ok(api_key) = std::env::var("MINIMAX_API_KEY")
            .or_else(|_| std::env::var("MINIMAX_TOKEN_PLAN_API_KEY"))
        {
            self.api_key = api_key;
        }
        if let Ok(group_id) = std::env::var("MINIMAX_GROUP_ID") {
            self.group_id = Some(group_id);
        }
        if let Ok(image_url) = std::env::var("MINIMAX_IMAGE_URL") {
            self.image_url = image_url;
        }
        if let Ok(tts_url) = std::env::var("MINIMAX_TTS_URL") {
            self.tts_url = tts_url;
        }
    }
}

#[derive(Clone)]
pub struct MinimaxClient {
    http: reqwest::Client,
    config: MinimaxConfig,
}

impl MinimaxClient {
    pub fn new(config: MinimaxConfig) -> Result<Self> {
        if config.api_key.trim().is_empty() {
            return Err(AgentPlanError::MissingApiKey);
        }
        Ok(Self {
            http: reqwest::Client::new(),
            config,
        })
    }

    pub async fn generate_image(
        &self,
        request: &MinimaxImageRequest,
    ) -> Result<MinimaxImageResponse> {
        let response = self
            .http
            .post(self.config.image_url.as_str())
            .bearer_auth(self.config.api_key.as_str())
            .json(request)
            .send()
            .await?;
        let response: MinimaxImageResponse = decode_json_response(response).await?;
        response.ensure_success()?;
        Ok(response)
    }

    pub async fn synthesize_speech(
        &self,
        request: &MinimaxTtsRequest,
    ) -> Result<MinimaxTtsResponse> {
        let mut builder = self
            .http
            .post(self.config.tts_url.as_str())
            .bearer_auth(self.config.api_key.as_str())
            .json(request);
        if let Some(group_id) = self.config.group_id.as_deref() {
            builder = builder.query(&[("GroupId", group_id)]);
        }
        let response: RawMinimaxTtsResponse = decode_json_response(builder.send().await?).await?;
        response.into_tts_response()
    }
}

async fn decode_json_response<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T> {
    let status = response.status();
    let body = response.text().await?;
    if !status.is_success() {
        return Err(AgentPlanError::ApiStatus { status, body });
    }
    Ok(serde_json::from_str(&body)?)
}

#[derive(Debug, Clone, Serialize)]
pub struct MinimaxImageRequest {
    pub model: String,
    pub prompt: String,
    pub response_format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_optimizer: Option<bool>,
}

impl MinimaxImageRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            model: "image-01".to_string(),
            prompt: prompt.into(),
            response_format: "url".to_string(),
            n: Some(1),
            prompt_optimizer: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MinimaxImageResponse {
    pub created: Option<u64>,
    pub data: Option<MinimaxImageData>,
    pub base_resp: Option<MinimaxBaseResp>,
}

impl MinimaxImageResponse {
    fn ensure_success(&self) -> Result<()> {
        if let Some(base_resp) = &self.base_resp {
            base_resp.ensure_success()?;
        }
        Ok(())
    }

    pub fn image_urls(&self) -> &[String] {
        self.data
            .as_ref()
            .map(|data| data.image_urls.as_slice())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MinimaxImageData {
    #[serde(default)]
    pub image_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MinimaxTtsRequest {
    pub model: String,
    pub text: String,
    pub voice_setting: MinimaxVoiceSetting,
    pub audio_setting: MinimaxAudioSetting,
}

impl MinimaxTtsRequest {
    pub fn new(text: impl Into<String>, voice_id: impl Into<String>) -> Self {
        Self {
            model: "speech-02-hd".to_string(),
            text: text.into(),
            voice_setting: MinimaxVoiceSetting {
                voice_id: voice_id.into(),
                speed: 1.0,
                vol: 1.0,
                pitch: 0,
            },
            audio_setting: MinimaxAudioSetting {
                sample_rate: 32_000,
                bitrate: 128_000,
                format: MinimaxAudioFormat::Mp3,
                channel: 1,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MinimaxVoiceSetting {
    pub voice_id: String,
    pub speed: f32,
    pub vol: f32,
    pub pitch: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct MinimaxAudioSetting {
    pub sample_rate: u32,
    pub bitrate: u32,
    pub format: MinimaxAudioFormat,
    pub channel: u32,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MinimaxAudioFormat {
    Mp3,
    Wav,
    Flac,
    Pcm,
}

#[derive(Debug, Clone)]
pub struct MinimaxTtsResponse {
    pub audio: Vec<u8>,
    pub trace_id: Option<String>,
    pub extra_info: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RawMinimaxTtsResponse {
    pub data: Option<RawMinimaxTtsData>,
    pub trace_id: Option<String>,
    pub extra_info: Option<serde_json::Value>,
    pub base_resp: Option<MinimaxBaseResp>,
}

impl RawMinimaxTtsResponse {
    fn into_tts_response(self) -> Result<MinimaxTtsResponse> {
        if let Some(base_resp) = &self.base_resp {
            base_resp.ensure_success()?;
        }
        let audio_hex = self
            .data
            .and_then(|data| data.audio)
            .ok_or(AgentPlanError::EmptyAudio)?;
        let audio = decode_hex_audio(&audio_hex)?;
        if audio.is_empty() {
            return Err(AgentPlanError::EmptyAudio);
        }
        Ok(MinimaxTtsResponse {
            audio,
            trace_id: self.trace_id,
            extra_info: self.extra_info,
        })
    }
}

#[derive(Debug, Deserialize)]
struct RawMinimaxTtsData {
    pub audio: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MinimaxBaseResp {
    pub status_code: i64,
    #[serde(default)]
    pub status_msg: String,
}

impl MinimaxBaseResp {
    fn ensure_success(&self) -> Result<()> {
        if self.status_code == 0 {
            return Ok(());
        }
        Err(AgentPlanError::ApiCode {
            code: self.status_code,
            message: self.status_msg.clone(),
        })
    }
}

fn decode_hex_audio(value: &str) -> Result<Vec<u8>> {
    let compact: Vec<u8> = value
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    if !compact.len().is_multiple_of(2) {
        return Err(AgentPlanError::InvalidAudioHex);
    }
    compact
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_value(pair[0])?;
            let low = hex_value(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn hex_value(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(AgentPlanError::InvalidAudioHex),
    }
}

#[cfg(test)]
mod tests {
    use super::decode_hex_audio;

    #[test]
    fn decodes_hex_audio() {
        assert_eq!(decode_hex_audio("68656c6c6f").unwrap(), b"hello");
        assert_eq!(decode_hex_audio("68 65\n6c6c6f").unwrap(), b"hello");
    }

    #[test]
    fn rejects_invalid_hex_audio() {
        assert!(decode_hex_audio("abc").is_err());
        assert!(decode_hex_audio("xx").is_err());
    }
}
