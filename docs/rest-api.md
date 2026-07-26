# REST API

The API is served by `rembg-server`. Examples assume `http://localhost:8020` and an API key in `$API_KEY`.

## Authentication

When `REMBG_API_KEY` or `--api-key` configures a nonempty key, protected endpoints accept either header:

```http
X-API-Key: your-secret
```

```http
Authorization: Bearer your-secret
```

Missing or invalid credentials return `401 Unauthorized`, `WWW-Authenticate: Bearer`, and:

```json
{
  "error": "unauthorized",
  "message": "Invalid or missing API key"
}
```

## `GET /healthz`

Public liveness endpoint.

Response:

```text
OK
```

## `GET /api/config`

Public runtime configuration used by the web UI.

Example response:

```json
{
  "modelPath": "models/u2net.onnx",
  "host": "0.0.0.0",
  "port": 8020,
  "authRequired": true
}
```

## `GET /api/auth/validate`

Validates the supplied API key. Returns `200 OK` with plain text `OK` when authentication succeeds. If server authentication is disabled, it also returns `200 OK`.

```bash
curl --fail \
  -H "X-API-Key: $API_KEY" \
  http://localhost:8020/api/auth/validate
```

## `POST /api/remove-bg`

Processes one image supplied as `multipart/form-data`.

### Fields

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `image` or `file` | file | required | PNG, JPEG, WebP, or another format supported by `image` |
| `threshold` | integer `0..255` | `160` | Alpha threshold; higher values remove more |
| `binary` | boolean | `false` | Produce hard transparent/opaque alpha |
| `sticker` | boolean | `false` | Apply sticker-border processing |
| `compress` | boolean | `false` | Compress PNG output |
| `format` | string | `png` | `png`, `jpeg`/`jpg`, or `webp` |
| `includeMask` or `include_mask` | boolean | `false` | Include a separate grayscale PNG mask |

### Image-Only Response

With `includeMask=false`, the response body is the encoded image. `Content-Type` reflects the format and `Content-Disposition` supplies a filename such as `dragon-removed.png`.

```bash
curl --fail \
  -H "X-API-Key: $API_KEY" \
  -F image=@test-images/dragon.jpg \
  -F threshold=160 \
  -F format=png \
  --output result.png \
  http://localhost:8020/api/remove-bg
```

JPEG cannot preserve transparency, so JPEG output is composited onto white.

### Image And Mask Response

With `includeMask=true`, the server returns `application/json` with base64-encoded binary payloads:

```json
{
  "imageData": "iVBORw0KGgo...",
  "imageFormat": "image/png",
  "filename": "dragon-removed.png",
  "maskData": "iVBORw0KGgo...",
  "maskFormat": "image/png",
  "maskFilename": "dragon-mask.png"
}
```

```bash
curl --fail \
  -H "Authorization: Bearer $API_KEY" \
  -F image=@test-images/dragon.jpg \
  -F includeMask=true \
  http://localhost:8020/api/remove-bg
```

### Errors

Errors use JSON:

```json
{
  "error": "bad_request",
  "message": "No image file provided"
}
```

Possible statuses include:

| Status | Meaning |
| --- | --- |
| `400` | Missing/invalid multipart data or image |
| `401` | Missing or invalid API key |
| `500` | Model loading, inference, processing, or encoding failure |
