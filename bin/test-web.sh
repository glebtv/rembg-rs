#!/usr/bin/env sh

set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(dirname "$script_dir")
cd "$repo_root"

if [ ! -f "${REMBG_E2E_MODEL:-models/u2net.onnx}" ]; then
    printf '%s\n' "E2E model not found; set REMBG_E2E_MODEL or download models/u2net.onnx" >&2
    exit 1
fi

npm --prefix web ci
npm --prefix web run build

if [ -n "${WEBDRIVER_URL:-}" ]; then
    exec cargo test --features server --test web_e2e "$@" -- --ignored --nocapture
fi

webdriver_url="http://127.0.0.1:9515"
chromedriver_pid=""

cleanup() {
    if [ -n "$chromedriver_pid" ]; then
        kill "$chromedriver_pid" 2>/dev/null || true
        wait "$chromedriver_pid" 2>/dev/null || true
    fi
}

trap cleanup EXIT HUP INT TERM

if ! curl --fail --silent "$webdriver_url/status" >/dev/null 2>&1; then
    if ! command -v chromedriver >/dev/null 2>&1; then
        printf '%s\n' "chromedriver is required for web E2E tests" >&2
        exit 1
    fi

    chromedriver_log="${CHROMEDRIVER_LOG:-/tmp/rembg-rs-chromedriver.log}"
    chromedriver --port=9515 >"$chromedriver_log" 2>&1 &
    chromedriver_pid=$!

    attempts=0
    until curl --fail --silent "$webdriver_url/status" >/dev/null 2>&1; do
        attempts=$((attempts + 1))
        if [ "$attempts" -ge 50 ] || ! kill -0 "$chromedriver_pid" 2>/dev/null; then
            printf '%s\n' "chromedriver failed to start; see $chromedriver_log" >&2
            exit 1
        fi
        sleep 0.1
    done
fi

cargo test --features server --test web_e2e "$@" -- --ignored --nocapture
