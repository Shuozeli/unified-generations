<!-- agent-updated: 2026-07-11T05:24:00Z -->
# API

## Client SDK

`AgentPlanClient` exposes three methods:

- `send_message(&LlmMessageRequest) -> LlmMessageResponse`
- `generate_image(&ImageGenerationRequest) -> ImageGenerationResponse`
- `synthesize_speech(&TtsRequest) -> TtsResponse`

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
