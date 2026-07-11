<!-- agent-updated: 2026-07-11T05:10:00Z -->
# API

## Client SDK

`AgentPlanClient` exposes three methods:

- `send_message(&LlmMessageRequest) -> LlmMessageResponse`
- `generate_image(&ImageGenerationRequest) -> ImageGenerationResponse`
- `synthesize_speech(&TtsRequest) -> TtsResponse`

## Server

`doubao-agent-plan-server` provides a thin JSON HTTP facade.

### `GET /healthz`

Returns:

```json
{"ok": true}
```

### `POST /v1/messages`

Request shape matches `LlmMessageRequest`.

### `POST /v1/images:generate`

Request shape matches `ImageGenerationRequest`.

### `POST /v1/tts:synthesize`

Request shape matches `TtsRequest`.

The server returns base64 audio:

```json
{
  "audio_base64": "...",
  "content_type": "text/plain; charset=utf-8",
  "log_id": "..."
}
```
