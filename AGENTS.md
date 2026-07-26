# Repository Notes

## Sources Of Truth

- Prefer `Cargo.toml`, `web/package.json`, `.github/workflows/ci.yml`, and `bin/*.sh` over older README examples. The README still shows the pre-subcommand CLI and a floating-point threshold; the current CLI is `rembg-rs remove-bg` and uses a `u8` threshold from 0 to 255.
- Default Cargo features are empty. The CLI, Axum server, and model downloader require `cli`, `server`, and `download` respectively; use `--all-features` for repository-wide checks.
- Keep `.gitignore` entry `/models/` anchored. An unanchored `models/` also ignores the required Rust sources in `src/models/`.

## Layout And Flow

- This is one Rust package plus a separate frontend package in `web/`; it is not a Cargo or npm workspace.
- `src/bin/main.rs` is the feature-gated CLI. `src/bin/server.rs` is the Axum server. `src/manager.rs` owns the ONNX session, and `src/rembg.rs` implements preprocessing, inference, mask normalization, and alpha application.
- The server serves `web/build` relative to the repository root. Run it from the root unless the static paths are changed deliberately.
- The frontend talks to `/api/*` through `web/src/api/client.ts`. With `includeMask=false`, removal returns an image blob; with `includeMask=true`, it returns JSON containing base64 image and mask payloads.
- At startup the server loads `--model`, advertises sibling `.onnx` files through `/api/config`, and lazily replaces the active session when the request's model ID changes. Do not preload all models: GPU memory is constrained.
- API-key auth accepts `X-API-Key` or `Authorization: Bearer`. The launcher reads `REMBG_API_KEY`, then `API_KEY`, and generates a key when neither is set.

## Environment

- Rust 1.85+ is required by edition 2024; CI uses stable. Frontend CI uses Node 22 and `npm ci`.
- ONNX Runtime is dynamically loaded and is not bundled. Install a compatible system library or set `ORT_DYLIB_PATH`; missing libraries should fail through the explicit probe in `src/manager.rs`.
- ONNX models under `models/`, generated `results/`, `web/build/`, and `web/node_modules/` are local ignored assets.
- CUDA is registered before CPU. In practice, CUDA initialization/allocation failures can abort session creation rather than falling back; avoid concurrent inference servers or model suites when GPU memory is tight.

## Verification

- Match CI with:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-features --all-targets -- -D warnings`
  - `cargo test --all-features`
  - `npm --prefix web ci`
  - `npm --prefix web run build`
- `cargo test --all-features <test-name>` runs a focused Rust test. Standard tests do not perform inference because both integration suites are `#[ignore]`.
- `bin/test-all-models.sh` runs every `.onnx` in `MODEL_DIR` (default `models/`) against every supported image in `TEST_IMAGE_DIR` (default `test-images/`) and writes image/mask pairs to `RESULT_DIR` (default `results/`). This is expensive and requires ONNX Runtime plus models.
- `bin/test-web.sh` runs the ignored ChromeDriver flow. It requires `models/u2net.onnx` or `REMBG_E2E_MODEL`, a compatible Chrome/ChromeDriver, and ONNX Runtime; use `CHROME_BINARY` or an external `WEBDRIVER_URL` when needed.
- `bin/test-api.sh` targets an already-running server. It accepts `SERVER_URL`, `API_KEY`/`REMBG_API_KEY`, optional `MODEL` and `IMAGE_PATH`, and writes responses under `RESULT_DIR`.
- Vite's current large-chunk warning is non-failing. There is no separate frontend lint or unit-test command; `npm run build` performs the strict TypeScript production check.

## Running Locally

- Production-style launcher: `API_KEY=secret BIND_PORT=8020 bin/start-server.sh`. It builds both packages and enables auth.
- Direct server: `cargo run --features server --bin rembg-server -- --model models/u2net.onnx`. Its binary default port is `8080`, unlike the launcher default of `8020`.
- Frontend dev server: `npm --prefix web run dev`. Vite proxies `/api` and `/healthz` to `localhost:8080`, so pair it with the direct server default or update `web/vite.config.ts` intentionally.
- Current CLI example: `cargo run --features cli --bin rembg-rs -- remove-bg --input test-images/dragon.jpg --output results/dragon.png --model models/u2net.onnx --threshold 160`.
- Download models with `cargo run --features cli,download --bin rembg-rs -- download-models --name all --dir models`.
