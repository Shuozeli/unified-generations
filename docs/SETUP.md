<!-- agent-updated: 2026-07-13T05:12:14Z -->
# Setup

This project provides a Rust SDK, CLI, and gRPC server for unified generation
providers. The current provider implementation is Volcengine Doubao Ark Agent
Plan.

## Official Ark Docs

Use these pages when checking model names, endpoint behavior, or plan coverage:

- [Ark Agent Plan API calling docs](https://console.volcengine.com/ark/region:cn-beijing/docs/82379/2373738)
- [Volcengine Ark model list](https://www.volcengine.com/docs/82379/1330310)
- [Volcengine Ark Agent Plan overview](https://www.volcengine.com/docs/82379/2366394)

The local implementation currently maps to:

| Capability | Default endpoint |
|---|---|
| LLM messages | `https://ark.cn-beijing.volces.com/api/plan/v1/messages` |
| Image generation | `https://ark.cn-beijing.volces.com/api/plan/v3/images/generations` |
| Text-to-speech | `https://openspeech.bytedance.com/api/v3/plan/tts/unidirectional` |

## Prerequisites

- Rust toolchain compatible with the workspace `rust-version`.
- Network access to Volcengine Ark and ByteDance OpenSpeech endpoints.
- A valid Ark Agent Plan API key.
- `grpcurl` only if you want to test the gRPC server manually.

## Install And Configure

Clone the repo and build the workspace:

```bash
git clone git@github.com:Shuozeli/unified-generations.git
cd unified-generations
cargo build --workspace
```

Initialize local config:

```bash
cargo run -p unified-generations-cli -- init --api-key ark-...
```

This creates:

```text
~/.arkcli/config.toml
```

Default config shape:

```toml
api_key = "ark-..."
plan_base_url = "https://ark.cn-beijing.volces.com/api/plan"
tts_url = "https://openspeech.bytedance.com/api/v3/plan/tts/unidirectional"
tts_resource_id = "seed-tts-2.0"
anthropic_version = "2023-06-01"
```

Inspect the active config without printing the raw key:

```bash
cargo run -p unified-generations-cli -- config show
```

## Config Precedence

Configuration is resolved in this order:

1. Built-in defaults for endpoints and protocol version.
2. `~/.arkcli/config.toml`.
3. Environment variables.
4. Explicit `--api-key` for CLI commands, or `Some(api_key)` in
   `AgentPlanConfig::from_sources`.

Supported environment variables:

- `DOUBAO_ARK_AGENT_PLAN_API_KEY`
- `ARK_AGENT_PLAN_API_KEY`
- `ARK_AGENT_PLAN_BASE_URL`
- `ARK_AGENT_PLAN_TTS_URL`
- `ARK_AGENT_PLAN_TTS_RESOURCE_ID`
- `ARK_AGENT_PLAN_ANTHROPIC_VERSION`

## First Calls

Chat:

```bash
cargo run -p unified-generations-cli -- chat \
  --model doubao-seed-2.0-mini \
  "Reply only with OK."
```

Image:

```bash
cargo run -p unified-generations-cli -- image \
  --model doubao-seedream-5.0-lite \
  --size 2K \
  --out image.png \
  "A clean product photo of a red cube on a white background."
```

Speech:

```bash
cargo run -p unified-generations-cli -- tts \
  --speaker zh_female_gaolengyujie_uranus_bigtts \
  --out speech.mp3 \
  "你好，欢迎使用语音合成服务。"
```

Voice catalog:

```bash
cargo run -p unified-generations-cli -- voices
cargo run -p unified-generations-cli -- voices --gender female
cargo run -p unified-generations-cli -- voices --category audiobook
```

## gRPC Server

Start the local server:

```bash
cargo run -p unified-generations-server -- --bind 127.0.0.1:8787
```

Call the message RPC:

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

The protobuf contract is:

```text
crates/unified-generations-server/proto/doubao/agentplan/v1/agent_plan.proto
```

## Development Checks

Run the same checks used by CI:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Troubleshooting

`missing API key` means neither the config file nor supported environment
variables provided a key. Run `init` again or export one of the key variables.

`UnsupportedModel` usually means the selected model is not enabled for the
current Agent Plan key. Check the [model list](https://www.volcengine.com/docs/82379/1330310)
and the [Agent Plan overview](https://www.volcengine.com/docs/82379/2366394).

TTS `resource ID is mismatched with speaker related resource` means the speaker
ID does not belong to the configured `tts_resource_id`. The default
`seed-tts-2.0` works with the built-in presets returned by `voices`.
