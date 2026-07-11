use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use clap::Parser;
use doubao_agent_plan::{
    AgentPlanClient, AgentPlanConfig, ImageGenerationRequest, LlmMessageRequest, TtsRequest,
};
use serde::Serialize;
use tower_http::trace::TraceLayer;

#[derive(Debug, Parser)]
#[command(version, about = "HTTP server for Doubao Ark Agent Plan")]
struct Args {
    #[arg(long, env = "DOUBAO_ARK_AGENT_PLAN_API_KEY")]
    api_key: Option<String>,

    #[arg(long, default_value = "127.0.0.1:8787")]
    bind: SocketAddr,
}

#[derive(Clone)]
struct AppState {
    client: Arc<AgentPlanClient>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let config = match args.api_key {
        Some(api_key) => AgentPlanConfig::new(api_key),
        None => AgentPlanConfig::from_env()?,
    };
    let state = AppState {
        client: Arc::new(AgentPlanClient::new(config)?),
    };
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/messages", post(messages))
        .route("/v1/images:generate", post(images))
        .route("/v1/tts:synthesize", post(tts))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(args.bind).await?;
    tracing::info!(addr = %args.bind, "listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz() -> Json<Health> {
    Json(Health { ok: true })
}

async fn messages(
    State(state): State<AppState>,
    Json(request): Json<LlmMessageRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let response = state.client.send_message(&request).await?;
    Ok(Json(serde_json::to_value(response)?))
}

async fn images(
    State(state): State<AppState>,
    Json(request): Json<ImageGenerationRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let response = state.client.generate_image(&request).await?;
    Ok(Json(serde_json::to_value(response)?))
}

async fn tts(
    State(state): State<AppState>,
    Json(request): Json<TtsRequest>,
) -> Result<Json<SynthesizeSpeechResponse>, ApiError> {
    let response = state.client.synthesize_speech(&request).await?;
    Ok(Json(SynthesizeSpeechResponse {
        audio_base64: base64::engine::general_purpose::STANDARD.encode(response.audio),
        content_type: response.content_type,
        log_id: response.log_id,
    }))
}

#[derive(Serialize)]
struct Health {
    ok: bool,
}

#[derive(Serialize)]
struct SynthesizeSpeechResponse {
    audio_base64: String,
    content_type: Option<String>,
    log_id: Option<String>,
}

struct ApiError(anyhow::Error);

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = Json(serde_json::json!({
            "error": self.0.to_string(),
        }));
        (StatusCode::BAD_GATEWAY, body).into_response()
    }
}
