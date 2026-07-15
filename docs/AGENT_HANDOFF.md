<!-- agent-updated: 2026-07-15T17:52:20Z -->
# Agent Handoff

This document is for another coding agent taking over `unified-generations`.

## Project Location

Local checkout:

```text
/home/cyuan/projects/unified-generations
```

GitHub repository:

```text
https://github.com/Shuozeli/unified-generations
```

Default branch:

```text
main
```

## What This Repo Does

`unified-generations` is a Rust workspace for generation providers. It currently
implements Volcengine Doubao Ark Agent Plan access through:

- `crates/unified-generations`: Rust SDK.
- `crates/unified-generations-cli`: CLI binary named `unified-generations`.
- `crates/unified-generations-server`: tonic gRPC server.

Implemented capabilities:

- LLM messages.
- Image generation.
- Text-to-speech.
- Built-in TTS voice preset listing.

## Doubao Ark Coding / Agent Plan Key

Do not commit the raw key to this repo.

The expected local key location is:

```text
~/.arkcli/config.toml
```

The config should contain:

```toml
api_key = "ark-..."
plan_base_url = "https://ark.cn-beijing.volces.com/api/plan"
tts_url = "https://openspeech.bytedance.com/api/v3/plan/tts/unidirectional"
tts_resource_id = "seed-tts-2.0"
anthropic_version = "2023-06-01"
```

On this machine, if `~/.arkcli/config.toml` does not exist, initialize it with:

```bash
cargo run -p unified-generations-cli -- init --api-key ark-...
```

The SDK and server also read these environment variables:

- `DOUBAO_ARK_AGENT_PLAN_API_KEY`
- `ARK_AGENT_PLAN_API_KEY`
- `ARK_AGENT_PLAN_BASE_URL`
- `ARK_AGENT_PLAN_TTS_URL`
- `ARK_AGENT_PLAN_TTS_RESOURCE_ID`
- `ARK_AGENT_PLAN_ANTHROPIC_VERSION`

Config precedence:

1. Built-in defaults.
2. `~/.arkcli/config.toml`.
3. Environment variables.
4. Explicit CLI `--api-key` or SDK `AgentPlanConfig::from_sources(Some(...))`.

Check the active config without printing the raw key:

```bash
cargo run -p unified-generations-cli -- config show
```

## Official Ark Docs

Use the official Ark docs as the source of truth when model names, quotas, or
endpoint behavior change:

- [Ark Agent Plan API calling docs](https://console.volcengine.com/ark/region:cn-beijing/docs/82379/2373738)
- [Volcengine Ark model list](https://www.volcengine.com/docs/82379/1330310)
- [Volcengine Ark Agent Plan overview](https://www.volcengine.com/docs/82379/2366394)

Current endpoint mapping:

| Capability | Endpoint |
|---|---|
| LLM messages | `POST https://ark.cn-beijing.volces.com/api/plan/v1/messages` |
| Image generation | `POST https://ark.cn-beijing.volces.com/api/plan/v3/images/generations` |
| Text-to-speech | `POST https://openspeech.bytedance.com/api/v3/plan/tts/unidirectional` |

## Common Commands

Run CI-equivalent local checks:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Run a chat smoke test:

```bash
cargo run -p unified-generations-cli -- chat \
  --model doubao-seed-2.0-mini \
  "Reply only with OK."
```

Generate an image:

```bash
cargo run -p unified-generations-cli -- image \
  --model doubao-seedream-5.0-lite \
  --size 2K \
  --out image.png \
  "A clean product photo of a red cube on a white background."
```

Generate speech:

```bash
cargo run -p unified-generations-cli -- tts \
  --speaker zh_female_gaolengyujie_uranus_bigtts \
  --out speech.mp3 \
  "你好，欢迎使用语音合成服务。"
```

Start the gRPC server:

```bash
cargo run -p unified-generations-server -- --bind 127.0.0.1:8787
```

## Related Docs

- [Setup guide](SETUP.md)
- [API reference](API.md)
- [README](../README.md)
