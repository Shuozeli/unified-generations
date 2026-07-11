<!-- agent-updated: 2026-07-11T06:20:00Z -->
# doubao-agent-plan-rs

Rust client, CLI, and gRPC server for Volcengine Doubao Ark Agent Plan APIs.

This repo wraps the Agent Plan endpoints that were verified on 2026-07-11:

- LLM messages: `https://ark.cn-beijing.volces.com/api/plan/v1/messages`
- Image generation: `https://ark.cn-beijing.volces.com/api/plan/v3/images/generations`
- TTS: `https://openspeech.bytedance.com/api/v3/plan/tts/unidirectional`

The API key is never hard-coded. Initialize a local Ark CLI config first:

```bash
cargo run -p doubao-agent-plan-cli -- init --api-key ark-...
```

This writes `~/.arkcli/config.toml`:

```toml
api_key = "ark-..."
plan_base_url = "https://ark.cn-beijing.volces.com/api/plan"
tts_url = "https://openspeech.bytedance.com/api/v3/plan/tts/unidirectional"
tts_resource_id = "seed-tts-2.0"
anthropic_version = "2023-06-01"
```

Use `cargo run -p doubao-agent-plan-cli -- config show` to inspect the active
file with the key masked. The CLI, SDK helper, and gRPC server all read this
file by default.

For automation, command-line and environment overrides still work:

```bash
export DOUBAO_ARK_AGENT_PLAN_API_KEY=ark-...
export ARK_AGENT_PLAN_API_KEY=ark-...
export ARK_AGENT_PLAN_BASE_URL=https://ark.cn-beijing.volces.com/api/plan
export ARK_AGENT_PLAN_TTS_URL=https://openspeech.bytedance.com/api/v3/plan/tts/unidirectional
export ARK_AGENT_PLAN_TTS_RESOURCE_ID=seed-tts-2.0
export ARK_AGENT_PLAN_ANTHROPIC_VERSION=2023-06-01
```

## Workspace

```text
crates/
├── doubao-agent-plan          Rust client SDK
├── doubao-agent-plan-cli      Command line wrapper
└── doubao-agent-plan-server   tonic gRPC wrapper
```

## CLI

Initialize config:

```bash
cargo run -p doubao-agent-plan-cli -- init --api-key ark-...
cargo run -p doubao-agent-plan-cli -- config show
```

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

List built-in Agent Plan `seed-tts-2.0` voice presets:

```bash
cargo run -p doubao-agent-plan-cli -- voices
cargo run -p doubao-agent-plan-cli -- voices --gender male
cargo run -p doubao-agent-plan-cli -- voices --category audiobook
```

Probe the first few built-in TTS presets and write sample MP3s:

```bash
cargo run -p doubao-agent-plan-cli -- tts-probe \
  --limit 6 \
  --out-dir out/tts-probe \
  --text "你好，欢迎使用语音合成服务。"
```

Note: older `seed-tts-1.0` speakers may not match Agent Plan `seed-tts-2.0`.
For example, `zh_male_jingqiangkanye_moon_bigtts` returned
`resource ID is mismatched with speaker related resource` during the probe.

## Server

```bash
cargo run -p doubao-agent-plan-server -- --bind 127.0.0.1:8787
```

gRPC service:

- `doubao.agentplan.v1.AgentPlanService/Health`
- `doubao.agentplan.v1.AgentPlanService/SendMessage`
- `doubao.agentplan.v1.AgentPlanService/GenerateImage`
- `doubao.agentplan.v1.AgentPlanService/ListVoices`
- `doubao.agentplan.v1.AgentPlanService/SynthesizeSpeech`

The protobuf contract lives at:

```text
crates/doubao-agent-plan-server/proto/doubao/agentplan/v1/agent_plan.proto
```

Example with `grpcurl`:

```bash
grpcurl -plaintext \
  -d '{
    "model": "doubao-seed-2.0-mini",
    "maxTokens": 64,
    "messages": [{"role": "user", "content": "Reply only with OK."}]
  }' \
  127.0.0.1:8787 \
  doubao.agentplan.v1.AgentPlanService/SendMessage
```

## Rust API

```rust
use doubao_agent_plan::{AgentPlanClient, AgentPlanConfig, LlmMessageRequest};

# async fn example() -> anyhow::Result<()> {
let client = AgentPlanClient::new(AgentPlanConfig::from_sources(None)?)?;
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
