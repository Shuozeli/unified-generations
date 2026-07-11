<!-- agent-updated: 2026-07-11T06:20:00Z -->
# API

## Client SDK

`AgentPlanConfig::from_sources(None)` loads `~/.arkcli/config.toml`, then applies
environment variable overrides. Passing `Some(api_key)` overrides only the key
for one client construction.

Config file path:

```text
~/.arkcli/config.toml
```

Supported config fields:

- `api_key`
- `plan_base_url`
- `tts_url`
- `tts_resource_id`
- `anthropic_version`

Environment overrides:

- `DOUBAO_ARK_AGENT_PLAN_API_KEY`
- `ARK_AGENT_PLAN_API_KEY`
- `ARK_AGENT_PLAN_BASE_URL`
- `ARK_AGENT_PLAN_TTS_URL`
- `ARK_AGENT_PLAN_TTS_RESOURCE_ID`
- `ARK_AGENT_PLAN_ANTHROPIC_VERSION`

`AgentPlanClient` exposes three methods:

- `send_message(&LlmMessageRequest) -> LlmMessageResponse`
- `generate_image(&ImageGenerationRequest) -> ImageGenerationResponse`
- `synthesize_speech(&TtsRequest) -> TtsResponse`

The SDK also exposes `tts_voice_presets()`, a curated set of known Agent Plan
`seed-tts-2.0` speaker IDs with display names and categories.

## gRPC Server

`doubao-agent-plan-server` provides a tonic gRPC facade. The protobuf source is
`crates/doubao-agent-plan-server/proto/doubao/agentplan/v1/agent_plan.proto`.

Service: `doubao.agentplan.v1.AgentPlanService`

### `Health`

Returns:

```json
{"ok": true}
```

### `SendMessage`

Calls the Agent Plan Anthropic-compatible messages endpoint.

Key fields:

- `model`
- `max_tokens`
- `messages[] { role, content }`

Returns `text`, `usage_json`, and `raw_json`.

### `GenerateImage`

Calls Agent Plan image generation.

Key fields:

- `model`
- `prompt`
- `size`: `1K`, `2K`, or `4K`
- `output_format`: `png` or `jpeg`
- `watermark`

Returns repeated generated images plus `usage_json` and `raw_json`.

### `SynthesizeSpeech`

Calls Agent Plan TTS.

Key fields:

- `text`
- `speaker`
- `audio_format`: `mp3`, `pcm`, `wav`, or `ogg_opus`
- `sample_rate`
- `uid`
- optional `speech_rate`
- optional `loudness_rate`

Returns raw audio bytes:

```json
{
  "audio": "...",
  "content_type": "text/plain; charset=utf-8",
  "log_id": "..."
}
```

### `ListVoices`

Returns built-in TTS speaker presets. Optional filters:

- `locale`
- `gender`
- `category`

Categories currently include `general`, `role_play`, and `audiobook`.
