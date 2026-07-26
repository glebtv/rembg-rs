#!/usr/bin/env sh

set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(dirname "$script_dir")
cd "$repo_root"

bind_host=${BIND_HOST:-0.0.0.0}
bind_port=${BIND_PORT:-8020}
model_path=${MODEL_PATH:-models/u2net.onnx}
api_key=${REMBG_API_KEY:-${API_KEY:-}}

if [ ! -f "$model_path" ]; then
    printf '%s\n' "Model not found: $model_path" >&2
    exit 1
fi

if [ -z "$api_key" ]; then
    if ! command -v openssl >/dev/null 2>&1; then
        printf '%s\n' "Set REMBG_API_KEY or install openssl to generate one" >&2
        exit 1
    fi
    api_key=$(openssl rand -hex 32)
fi

printf '%s\n' "Installing and building the web UI..."
npm --prefix web install
npm --prefix web run build

printf '%s\n' "Building the server..."
cargo build --release --features server

printf 'Server: http://%s:%s\n' "$bind_host" "$bind_port"
printf 'API key: %s\n' "$api_key"
printf '%s\n' "Enter this key in the web UI or send it in X-API-Key."

export REMBG_API_KEY=$api_key
exec target/release/rembg-server \
    --no-build \
    --host "$bind_host" \
    --port "$bind_port" \
    --model "$model_path"
