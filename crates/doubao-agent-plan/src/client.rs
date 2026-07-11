use base64::Engine;
use bytes::Bytes;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use uuid::Uuid;

use crate::error::{AgentPlanError, Result};
use crate::image::{ImageGenerationRequest, ImageGenerationResponse};
use crate::llm::{LlmMessageRequest, LlmMessageResponse};
use crate::tts::{TtsLine, TtsRequest, TtsResponse};

#[derive(Debug, Clone)]
pub struct AgentPlanConfig {
    pub api_key: String,
    pub plan_base_url: String,
    pub anthropic_version: String,
}

impl AgentPlanConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            plan_base_url: "https://ark.cn-beijing.volces.com/api/plan".to_string(),
            anthropic_version: "2023-06-01".to_string(),
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("DOUBAO_ARK_AGENT_PLAN_API_KEY")
            .or_else(|_| std::env::var("ARK_AGENT_PLAN_API_KEY"))
            .map_err(|_| AgentPlanError::MissingApiKey)?;
        Ok(Self::new(api_key))
    }
}

#[derive(Clone)]
pub struct AgentPlanClient {
    http: reqwest::Client,
    config: AgentPlanConfig,
}

impl AgentPlanClient {
    pub fn new(config: AgentPlanConfig) -> Result<Self> {
        if config.api_key.trim().is_empty() {
            return Err(AgentPlanError::MissingApiKey);
        }
        Ok(Self {
            http: reqwest::Client::new(),
            config,
        })
    }

    pub async fn send_message(&self, request: &LlmMessageRequest) -> Result<LlmMessageResponse> {
        let url = format!(
            "{}/v1/messages",
            self.config.plan_base_url.trim_end_matches('/')
        );
        let response = self
            .http
            .post(url)
            .header("x-api-key", self.config.api_key.as_str())
            .header("anthropic-version", self.config.anthropic_version.as_str())
            .json(request)
            .send()
            .await?;
        decode_json_response(response).await
    }

    pub async fn generate_image(
        &self,
        request: &ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse> {
        let url = format!(
            "{}/v3/images/generations",
            self.config.plan_base_url.trim_end_matches('/')
        );
        let response = self
            .http
            .post(url)
            .bearer_auth(self.config.api_key.as_str())
            .json(request)
            .send()
            .await?;
        decode_json_response(response).await
    }

    pub async fn synthesize_speech(&self, request: &TtsRequest) -> Result<TtsResponse> {
        let response = self
            .http
            .post("https://openspeech.bytedance.com/api/v3/plan/tts/unidirectional")
            .headers(self.tts_headers()?)
            .json(request)
            .send()
            .await?;

        let status = response.status();
        let headers = response.headers().clone();
        let body = response.bytes().await?;
        if !status.is_success() {
            return Err(AgentPlanError::ApiStatus {
                status,
                body: String::from_utf8_lossy(&body).into_owned(),
            });
        }

        let audio = decode_tts_body(body)?;
        Ok(TtsResponse {
            audio,
            content_type: header_to_string(&headers, "content-type"),
            log_id: header_to_string(&headers, "x-tt-logid"),
        })
    }

    fn tts_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "X-Api-Key",
            HeaderValue::from_str(self.config.api_key.as_str())
                .map_err(|_| AgentPlanError::MissingApiKey)?,
        );
        headers.insert(
            "X-Api-Resource-Id",
            HeaderValue::from_static("seed-tts-2.0"),
        );
        headers.insert(
            "X-Api-Request-Id",
            HeaderValue::from_str(Uuid::new_v4().to_string().as_str())
                .expect("uuid is always a valid header value"),
        );
        headers.insert(
            "X-Control-Require-Usage-Tokens-Return",
            HeaderValue::from_static("*"),
        );
        Ok(headers)
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

fn decode_tts_body(body: Bytes) -> Result<Vec<u8>> {
    let mut audio = Vec::new();
    for raw_line in body.split(|byte| *byte == b'\n') {
        if raw_line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        let line: TtsLine = serde_json::from_slice(raw_line)?;
        match (line.code, line.data) {
            (0, Some(data)) => {
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(data.as_bytes())
                    .map_err(|err| AgentPlanError::TtsCode {
                        code: 0,
                        message: format!("invalid base64 audio chunk: {err}"),
                    })?;
                audio.extend(bytes);
            }
            (0, None) | (20000000, _) => {}
            (code, _) => {
                return Err(AgentPlanError::TtsCode {
                    code,
                    message: line.message,
                });
            }
        }
    }

    if audio.is_empty() {
        return Err(AgentPlanError::EmptyAudio);
    }
    Ok(audio)
}

fn header_to_string(headers: &HeaderMap, name: &'static str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::decode_tts_body;

    #[test]
    fn decodes_newline_delimited_tts_chunks() {
        let body = bytes::Bytes::from_static(
            br#"{"code":0,"message":"","data":"aGVs"}
{"code":0,"message":"","data":"bG8="}
"#,
        );

        let audio = decode_tts_body(body).expect("body should decode");

        assert_eq!(audio, b"hello");
    }

    #[test]
    fn surfaces_tts_error_code() {
        let body = bytes::Bytes::from_static(
            br#"{"code":55000000,"message":"resource ID is mismatched with speaker related resource"}"#,
        );

        let err = decode_tts_body(body).expect_err("body should fail");

        assert!(err.to_string().contains("55000000"));
    }
}
