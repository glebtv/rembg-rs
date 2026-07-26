# Web Server

`rembg-server` is an Axum application that loads an ONNX model at startup, discovers other ONNX models in the same directory, serializes access to its active ONNX Runtime session, exposes the REST API, and serves the production Vite build from `web/build`.

## Requirements

- Rust toolchain compatible with edition 2024.
- Node.js 22 or another version supported by the current frontend dependencies.
- A compatible dynamic ONNX Runtime library discoverable as `libonnxruntime.so`, `libonnxruntime.dylib`, or `onnxruntime.dll`.
- At least one model file, normally `models/u2net.onnx`.
- CUDA, cuDNN, and CUDA-enabled ONNX Runtime for GPU acceleration. CPU ONNX Runtime remains supported through fallback.

On the Arch-based development machine, CUDA ONNX Runtime was installed with:

```bash
sudo pacman -S onnxruntime-cuda
```

## Recommended Startup

```bash
API_KEY='a-long-random-secret' bin/start-server.sh
```

The script:

1. Verifies the selected model exists.
2. Generates a 256-bit key with OpenSSL when no key is supplied.
3. Runs `npm install` and builds `web/build`.
4. Builds `rembg-server` in release mode.
5. Starts it on `0.0.0.0:${BIND_PORT:-8020}`.

Supported environment variables:

| Variable | Default | Purpose |
| --- | --- | --- |
| `BIND_HOST` | `0.0.0.0` | Listening interface |
| `BIND_PORT` | `8020` | Listening port |
| `MODEL_PATH` | `models/u2net.onnx` | ONNX model loaded at startup |
| `REMBG_API_KEY` | generated | Preferred API-key variable |
| `API_KEY` | generated | Alternate API-key variable |
| `RUST_LOG` | server defaults | Rust tracing filter |

The generated key is printed once by the launcher. Enter it in the web form or use it as documented in [rest-api.md](rest-api.md).

## Manual API Test

With a server already running, process every image in `test-images/` and save the responses under `results/`:

```bash
API_KEY='your-secret' bin/test-api.sh
```

`SERVER_URL` defaults to `http://127.0.0.1:8020`. Set `IMAGE_PATH` to process one image, `MODEL` to select a model ID returned by `/api/config`, and `FORMAT`, `THRESHOLD`, or `RESULT_DIR` to override their defaults.

## Direct Startup

```bash
npm --prefix web install
npm --prefix web run build
cargo run --release --features server --bin rembg-server -- \
  --host 0.0.0.0 \
  --port 8020 \
  --model models/u2net.onnx \
  --no-build
```

Set `REMBG_API_KEY` to enable authentication. Authentication is optional when invoking the binary directly, but the launcher always enables it.

## Browser Authentication

When authentication is enabled, `/api/config` tells the web application that a key is required. The UI presents a password field, validates the key with `/api/auth/validate`, clears the input, and retains the accepted key only in React memory for the current page lifetime.

## Runtime Behavior

- CUDA is requested first and CPU is the explicit fallback.
- The startup model is loaded immediately. Selecting another discovered model replaces the active session lazily, so all models are not held in GPU memory at once.
- Requests lock the model session, so inference is serialized.
- `/healthz`, `/api/config`, and static frontend assets remain public.
- Authentication protects validation and background-removal API operations.
- Unknown frontend routes fall back to `web/build/index.html`.

## Production Notes

- Put a TLS reverse proxy in front of the server before using it across an untrusted network.
- Restrict CORS instead of using the current permissive policy for public deployments.
- Add body-size and rate limits when accepting untrusted traffic.
- Do not place API keys in URLs, logs, source files, or the frontend build.
