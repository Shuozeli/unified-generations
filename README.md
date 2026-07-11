<!-- agent-updated: 2026-07-11T05:10:00Z -->
# doubao-agent-plan-rs

Rust client, CLI, and HTTP server for Volcengine Doubao Ark Agent Plan APIs.

This repo wraps the Agent Plan endpoints that were verified on 2026-07-11:

- LLM messages: `https://ark.cn-beijing.volces.com/api/plan/v1/messages`
- Image generation: `https://ark.cn-beijing.volces.com/api/plan/v3/images/generations`
- TTS: `https://openspeech.bytedance.com/api/v3/plan/tts/unidirectional`

The API key is never hard-coded. Set one of:

```bash
export DOUBAO_ARK_AGENT_PLAN_API_KEY=ark-...
export ARK_AGENT_PLAN_API_KEY=ark-...
```

## Workspace

```text
crates/
├── doubao-agent-plan          Rust client SDK
├── doubao-agent-plan-cli      Command line wrapper
└── doubao-agent-plan-server   Axum HTTP wrapper
```

## CLI

Chat:

```bash
cargo run -p doubao-agent-plan-cli -- chat \
  --model doubao-seed-2.0-mini \
  "Reply only with OK."
```

Generate an image:

```bash
cargo run -p doubao-agent-plan-cli -- image \
  "A small red cube on a plain white background, minimal product photo."
```

Synthesize speech:

```bash
cargo run -p doubao-agent-plan-cli -- tts \
  --speaker zh_female_gaolengyujie_uranus_bigtts \
  --out speech.mp3 \
  "你好，欢迎使用语音合成服务。"
```

Note: older `seed-tts-1.0` speakers may not match Agent Plan `seed-tts-2.0`.
For example, `zh_male_jingqiangkanye_moon_bigtts` returned
`resource ID is mismatched with speaker related resource` during the probe.

## Server

```bash
cargo run -p doubao-agent-plan-server -- --bind 127.0.0.1:8787
```

Endpoints:

- `GET /healthz`
- `POST /v1/messages`
- `POST /v1/images:generate`
- `POST /v1/tts:synthesize`

Example:

```bash
curl http://127.0.0.1:8787/v1/messages \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "doubao-seed-2.0-mini",
    "max_tokens": 64,
    "messages": [{"role": "user", "content": "Reply only with OK."}]
  }'
```

## Rust API

```rust
use doubao_agent_plan::{AgentPlanClient, AgentPlanConfig, LlmMessageRequest};

# async fn example() -> anyhow::Result<()> {
let client = AgentPlanClient::new(AgentPlanConfig::from_env()?)?;
let response = client
    .send_message(&LlmMessageRequest::new("doubao-seed-2.0-mini", "Reply with OK."))
    .await?;
println!("{}", response.text());
# Ok(())
# }
```

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
