<!-- agent-updated: 2026-07-16T05:09:09Z -->
# API

## Official References

This crate currently targets the Volcengine Doubao Ark Agent Plan API surface.
The original Ark documentation pages are the source of truth when endpoint
contracts change:

- [Ark Agent Plan API calling docs](https://console.volcengine.com/ark/region:cn-beijing/docs/82379/2373738)
- [Volcengine Ark model list](https://www.volcengine.com/docs/82379/1330310)
- [Volcengine Ark Agent Plan overview](https://www.volcengine.com/docs/82379/2366394)

Implemented endpoint mapping:

| SDK / gRPC method | HTTP endpoint | Official doc |
|---|---|---|
| `send_message` / `SendMessage` | `POST https://ark.cn-beijing.volces.com/api/plan/v1/messages` | [Agent Plan calling docs](https://console.volcengine.com/ark/region:cn-beijing/docs/82379/2373738) |
| `generate_image` / `GenerateImage` | `POST https://ark.cn-beijing.volces.com/api/plan/v3/images/generations` | [Agent Plan calling docs](https://console.volcengine.com/ark/region:cn-beijing/docs/82379/2373738) |
| `synthesize_speech` / `SynthesizeSpeech` | `POST https://openspeech.bytedance.com/api/v3/plan/tts/unidirectional` | [Agent Plan calling docs](https://console.volcengine.com/ark/region:cn-beijing/docs/82379/2373738) |

MiniMax references:

- [MiniMax image generation docs](https://platform.minimax.io/docs/api-reference/image-generation-t2i)
- [MiniMax TTS HTTP docs](https://platform.minimax.io/docs/api-reference/speech-t2a-http)

MiniMax endpoint mapping:

| SDK method | HTTP endpoint |
|---|---|
| `MinimaxClient::generate_image` | `POST https://api.minimax.io/v1/image_generation` |
| `MinimaxClient::synthesize_speech` | `POST https://api.minimax.io/v1/t2a_v2` |

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

## MiniMax SDK

`MinimaxConfig::from_sources(None)` loads MiniMax settings from environment
variables. Passing `Some(api_key)` overrides only the key for one client
construction.

Supported environment variables:

- `MINIMAX_API_KEY`
- `MINIMAX_TOKEN_PLAN_API_KEY`
- `MINIMAX_GROUP_ID`
- `MINIMAX_IMAGE_URL`
- `MINIMAX_TTS_URL`

Defaults:

- Image endpoint: `https://api.minimax.io/v1/image_generation`
- TTS endpoint: `https://api.minimax.io/v1/t2a_v2`

`MinimaxClient` exposes two methods:

- `generate_image(&MinimaxImageRequest) -> MinimaxImageResponse`
- `synthesize_speech(&MinimaxTtsRequest) -> MinimaxTtsResponse`

The MiniMax image default model is `image-01`. The MiniMax TTS default model is
`speech-02-hd`.

## gRPC Server

`unified-generations-server` provides a tonic gRPC facade. The protobuf source is
`crates/unified-generations-server/proto/doubao/agentplan/v1/agent_plan.proto`.

Service: `doubao.agentplan.v1.AgentPlanService`

### `Health`

Returns:

```json
{"ok": true}
```

### `SendMessage`

Calls the Agent Plan Anthropic-compatible messages endpoint.

Default endpoint: `POST https://ark.cn-beijing.volces.com/api/plan/v1/messages`.

Key fields:

- `model`
- `max_tokens`
- `messages[] { role, content }`

Returns `text`, `usage_json`, and `raw_json`.

### `GenerateImage`

Calls Agent Plan image generation.

Default endpoint:
`POST https://ark.cn-beijing.volces.com/api/plan/v3/images/generations`.

Key fields:

- `model`
- `prompt`
- `size`: `1K`, `2K`, or `4K`
- `output_format`: `png` or `jpeg`
- `watermark`

Returns repeated generated images plus `usage_json` and `raw_json`.

### `SynthesizeSpeech`

Calls Agent Plan TTS.

Default endpoint:
`POST https://openspeech.bytedance.com/api/v3/plan/tts/unidirectional`.

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
