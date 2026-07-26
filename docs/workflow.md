# Development Workflow

## Approach Used

1. Inspected the current `rembg-rs` implementation, the Python `/data/rembg` UI, and Krasichka's WebDriver and API-key patterns.
2. Reproduced the npm dependency resolution failure and migrated the entire web build to Tailwind CSS 4.
3. Built the Rust and TypeScript projects to expose API, type, and runtime mismatches.
4. Traced the model-loading hang with `strace`, identified the absent dynamic ONNX Runtime library, and added a fail-fast runtime probe.
5. Installed CUDA ONNX Runtime locally and confirmed GPU allocation with `nvidia-smi`.
6. Ran every downloaded model against every test image, corrected BRIA's input size, and reran the matrix.
7. Added and ran a real headless Chrome upload/process/download test.
8. Reviewed the implementation for routing, encoding, alpha, authentication, and deployment problems.
9. Ran formatting, tests, Clippy, npm build, npm audit, and diff checks.

## Core Commands

Install and build the web UI:

```bash
npm --prefix web install
npm --prefix web run build
```

Build and test Rust code without model inference:

```bash
cargo build --all-features
cargo test --all-features
cargo clippy --all-features --all-targets -- -D warnings
cargo fmt --all -- --check
```

Run the local model assertion suite:

```bash
cargo test --test model_inference -- --ignored --nocapture
```

Generate inspectable outputs for every model/image pair:

```bash
bin/test-all-models.sh
```

The generator reuses one loaded session per model, runs the matrix assertions, and writes each result and grayscale mask beneath `results/<model>/`.

Run ChromeDriver E2E tests:

```bash
CHROME_BINARY=/usr/bin/google-chrome-beta bin/test-web.sh
```

Start the production build and server:

```bash
API_KEY='replace-with-a-secret' BIND_PORT=8020 bin/start-server.sh
```

## Useful Diagnostics

Check ONNX Runtime and CUDA libraries:

```bash
ldconfig -p | rg 'onnxruntime|cuda|cudnn'
ldd /usr/lib/libonnxruntime_providers_cuda.so
```

Check active GPU processes:

```bash
nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv,noheader
```

Force the missing-runtime path to verify the diagnostic:

```bash
ORT_DYLIB_PATH=/tmp/missing-libonnxruntime.so \
  target/debug/rembg-rs remove-bg \
  --input test-images/dragon.jpg \
  --output /tmp/result.png \
  --model models/u2net.onnx
```

Inspect repository changes before committing:

```bash
git status --short
git diff --check
git diff --stat
```
