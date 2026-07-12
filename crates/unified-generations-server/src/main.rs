use std::net::SocketAddr;
use std::sync::Arc;

use clap::Parser;
use tonic::{Request, Response, Status, transport::Server};
use unified_generations::{
    AgentPlanClient, AgentPlanConfig, AudioFormat, ImageGenerationRequest, ImageOutputFormat,
    ImageSize, LlmMessage, LlmMessageRequest, MessageRole, TtsRequest, tts_voice_presets,
};

pub mod pb {
    tonic::include_proto!("doubao.agentplan.v1");
}

use pb::agent_plan_service_server::{AgentPlanService, AgentPlanServiceServer};
use pb::{
    ChatMessage, GenerateImageRequest, GenerateImageResponse, GeneratedImage, HealthRequest,
    HealthResponse, ListVoicesRequest, ListVoicesResponse, SendMessageRequest, SendMessageResponse,
    SynthesizeSpeechRequest, SynthesizeSpeechResponse, VoicePreset as ProtoVoicePreset,
};

#[derive(Debug, Parser)]
#[command(version, about = "gRPC server for Doubao Ark Agent Plan")]
struct Args {
    #[arg(long, env = "DOUBAO_ARK_AGENT_PLAN_API_KEY")]
    api_key: Option<String>,

    #[arg(long, default_value = "127.0.0.1:8787")]
    bind: SocketAddr,
}

#[derive(Clone)]
struct AgentPlanGrpc {
    client: Arc<AgentPlanClient>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let config = AgentPlanConfig::from_sources(args.api_key)?;
    let service = AgentPlanGrpc {
        client: Arc::new(AgentPlanClient::new(config)?),
    };

    tracing::info!(addr = %args.bind, "listening");
    Server::builder()
        .add_service(AgentPlanServiceServer::new(service))
        .serve(args.bind)
        .await?;
    Ok(())
}

#[tonic::async_trait]
impl AgentPlanService for AgentPlanGrpc {
    async fn health(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthResponse>, Status> {
        Ok(Response::new(HealthResponse { ok: true }))
    }

    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageResponse>, Status> {
        let request = request.into_inner();
        let response = self
            .client
            .send_message(&LlmMessageRequest {
                model: request.model,
                max_tokens: request.max_tokens,
                messages: request
                    .messages
                    .into_iter()
                    .map(map_chat_message)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Status::invalid_argument)?,
            })
            .await
            .map_err(to_status)?;

        Ok(Response::new(SendMessageResponse {
            id: response.id.clone(),
            role: response.role.clone(),
            model: response.model.clone(),
            text: response.text(),
            stop_reason: response.stop_reason.clone().unwrap_or_default(),
            usage_json: json_string(response.usage.as_ref()).map_err(to_status)?,
            raw_json: serde_json::to_string(&response).map_err(to_status)?,
        }))
    }

    async fn generate_image(
        &self,
        request: Request<GenerateImageRequest>,
    ) -> Result<Response<GenerateImageResponse>, Status> {
        let request = request.into_inner();
        let response = self
            .client
            .generate_image(&ImageGenerationRequest {
                model: default_if_empty(request.model, "doubao-seedream-5.0-lite"),
                prompt: request.prompt,
                size: parse_image_size(&request.size).map_err(Status::invalid_argument)?,
                output_format: parse_image_output_format(&request.output_format)
                    .map_err(Status::invalid_argument)?,
                watermark: request.watermark,
                response_format: Some("url".to_string()),
            })
            .await
            .map_err(to_status)?;

        let images = response
            .data
            .iter()
            .map(|image| GeneratedImage {
                url: image.url.clone().unwrap_or_default(),
                b64_json: image.b64_json.clone().unwrap_or_default(),
                size: image.size.clone().unwrap_or_default(),
            })
            .collect();

        Ok(Response::new(GenerateImageResponse {
            model: response.model.clone(),
            created: response.created,
            images,
            usage_json: json_string(response.usage.as_ref()).map_err(to_status)?,
            raw_json: serde_json::to_string(&response).map_err(to_status)?,
        }))
    }

    async fn list_voices(
        &self,
        request: Request<ListVoicesRequest>,
    ) -> Result<Response<ListVoicesResponse>, Status> {
        let request = request.into_inner();
        let voices = tts_voice_presets()
            .iter()
            .filter(|voice| request.locale.is_empty() || voice.locale == request.locale)
            .filter(|voice| request.gender.is_empty() || voice.gender == request.gender)
            .filter(|voice| request.category.is_empty() || voice.category == request.category)
            .map(|voice| ProtoVoicePreset {
                id: voice.id.to_string(),
                display_name: voice.display_name.to_string(),
                locale: voice.locale.to_string(),
                gender: voice.gender.to_string(),
                category: voice.category.to_string(),
                notes: voice.notes.to_string(),
            })
            .collect();
        Ok(Response::new(ListVoicesResponse { voices }))
    }

    async fn synthesize_speech(
        &self,
        request: Request<SynthesizeSpeechRequest>,
    ) -> Result<Response<SynthesizeSpeechResponse>, Status> {
        let request = request.into_inner();
        let mut tts = TtsRequest::new(request.text, request.speaker);
        if !request.uid.is_empty() {
            tts.user.uid = request.uid;
        }
        tts.req_params.audio_params.format =
            parse_audio_format(&request.audio_format).map_err(Status::invalid_argument)?;
        if request.sample_rate != 0 {
            tts.req_params.audio_params.sample_rate = request.sample_rate;
        }
        tts.req_params.audio_params.speech_rate = request.speech_rate;
        tts.req_params.audio_params.loudness_rate = request.loudness_rate;

        let response = self
            .client
            .synthesize_speech(&tts)
            .await
            .map_err(to_status)?;

        Ok(Response::new(SynthesizeSpeechResponse {
            audio: response.audio,
            content_type: response.content_type.unwrap_or_default(),
            log_id: response.log_id.unwrap_or_default(),
        }))
    }
}

fn map_chat_message(message: ChatMessage) -> Result<LlmMessage, String> {
    let role = match message.role.as_str() {
        "" | "user" => MessageRole::User,
        "assistant" => MessageRole::Assistant,
        "system" => MessageRole::System,
        other => {
            return Err(format!("unsupported message role: {other}"));
        }
    };
    Ok(LlmMessage {
        role,
        content: message.content,
    })
}

fn parse_image_size(value: &str) -> Result<ImageSize, String> {
    match value {
        "" | "2K" | "two_k" => Ok(ImageSize::TwoK),
        "1K" | "one_k" => Ok(ImageSize::OneK),
        "4K" | "four_k" => Ok(ImageSize::FourK),
        other => Err(format!("unsupported image size: {other}")),
    }
}

fn parse_image_output_format(value: &str) -> Result<ImageOutputFormat, String> {
    match value {
        "" | "png" => Ok(ImageOutputFormat::Png),
        "jpeg" | "jpg" => Ok(ImageOutputFormat::Jpeg),
        other => Err(format!("unsupported image output format: {other}")),
    }
}

fn parse_audio_format(value: &str) -> Result<AudioFormat, String> {
    match value {
        "" | "mp3" => Ok(AudioFormat::Mp3),
        "pcm" => Ok(AudioFormat::Pcm),
        "wav" => Ok(AudioFormat::Wav),
        "ogg_opus" => Ok(AudioFormat::OggOpus),
        other => Err(format!("unsupported audio format: {other}")),
    }
}

fn default_if_empty(value: String, default: &str) -> String {
    if value.is_empty() {
        default.to_string()
    } else {
        value
    }
}

fn json_string(value: Option<&serde_json::Value>) -> serde_json::Result<String> {
    match value {
        Some(value) => serde_json::to_string(value),
        None => Ok(String::new()),
    }
}

fn to_status(error: impl std::fmt::Display) -> Status {
    Status::internal(error.to_string())
}
