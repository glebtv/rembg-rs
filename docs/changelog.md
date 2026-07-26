# Changelog

## 2026-07-27

### Web Application

- Ported the background-removal interface to React 19, HeroUI, Vite, and Tailwind CSS 4.
- Replaced the incompatible Tailwind 3 dependency and configuration with a consistent Tailwind 4 toolchain.
- Added upload, original/result previews, processing state, threshold, binary mask, sticker, PNG compression, output-format, and mask controls.
- Added independent result and mask previews and download buttons with server-provided filenames.
- Added responsive single-column output cards on narrow screens.
- Removed the stale Vite favicon reference.
- Added API-key entry and validation without storing the raw key in local or session storage.
- Refined the responsive layout, aligned the settings and action controls, and added a model selector.
- Removed the ONNX Runtime attribution footer.

### HTTP Server And API

- Added the Axum server binary and static Vite build serving.
- Added `/healthz`, `/api/config`, `/api/auth/validate`, and `/api/remove-bg` endpoints.
- Added optional API-key protection through `X-API-Key` and `Authorization: Bearer` headers.
- Added JSON error responses and `WWW-Authenticate: Bearer` on authentication failures.
- Added multipart image uploads and processing options.
- Added raw image responses for ordinary requests and base64 JSON image-plus-mask responses when `includeMask=true`.
- Fixed multipart output corruption by replacing binary parsing through text with an explicit binary-safe JSON contract.
- Fixed uploaded filename stem extraction.
- Added PNG, JPEG, and WebP encoding; JPEG output is composited onto white because JPEG has no alpha channel.
- Restricted static serving to `web/build` and added an SPA index fallback without exposing repository files.
- Added server-advertised model discovery and lazy model switching for ONNX files beside the startup model.
- Fixed the server CLI host shorthand conflict by assigning `-H` to `--host`.

### Inference And Images

- Added CUDA execution with explicit CPU fallback.
- Added runtime model input-dimension inspection.
- Added a 1024x1024 fallback for BRIA RMBG 2.0, whose graph declares dynamic dimensions despite requiring a fixed internal size.
- Added a dynamic-library probe that reports a clear ONNX Runtime installation error instead of hanging.
- Replaced the visualization heatmap with a normalized grayscale mask.
- Applied sigmoid only to logit-range output and normalized model output to the full mask range.
- Fixed soft-alpha scaling and preserved transparency already present in the source image.
- Changed `RemovalResult::mask` from RGB to grayscale.
- Made the CLI fall back to an uncompressed PNG when optional quantization rejects an image.
- Added `--no-compress` so batch generation can skip unnecessary PNG quantization.

### Tests And Tooling

- Added an ignored model integration test that serially runs every local ONNX model against every test image.
- Verified 15 models against 3 images: 45 successful inference cases on CUDA.
- Added a ChromeDriver test covering upload, mask selection, processing, decoded previews, and real result/mask downloads.
- Added `bin/test-all-models.sh`, `bin/test-web.sh`, and `bin/start-server.sh`.
- Added `bin/test-api.sh` for manual health, config, and real image-request smoke tests against a running server.
- Added GitHub Actions jobs for formatting, Clippy, non-inference Rust tests, npm installation, and the production web build.
- Added API-key unit tests.
- Added generated web dependencies, assets, results, and local model files to ignore rules where applicable.

### Machine Setup

- Installed the CUDA-enabled ONNX Runtime package on the CUDA development machine.
- Confirmed the inference test process used the NVIDIA GPU.
- Updated npm dependencies and verified zero reported npm vulnerabilities.
