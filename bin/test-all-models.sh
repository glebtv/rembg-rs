#!/usr/bin/env sh

set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(dirname "$script_dir")
cd "$repo_root"

model_dir=${MODEL_DIR:-models}
image_dir=${TEST_IMAGE_DIR:-test-images}
result_dir=${RESULT_DIR:-results}

mkdir -p "$result_dir"
MODEL_DIR="$model_dir" \
TEST_IMAGE_DIR="$image_dir" \
RESULT_DIR="$result_dir" \
    cargo test --test model_inference -- --ignored --nocapture

printf 'All generated files are in %s\n' "$result_dir"
