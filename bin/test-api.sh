#!/usr/bin/env sh

set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(dirname "$script_dir")
cd "$repo_root"

server_url=${SERVER_URL:-http://127.0.0.1:8020}
server_url=${server_url%/}
api_key=${API_KEY:-${REMBG_API_KEY:-}}
model=${MODEL:-}
image_path=${IMAGE_PATH:-}
format=${FORMAT:-png}
threshold=${THRESHOLD:-160}
result_dir=${RESULT_DIR:-results}

case "$format" in
    png | webp) output_extension=$format ;;
    jpeg | jpg) output_extension=jpg ;;
    *)
        printf '%s\n' "Unsupported FORMAT: $format (use png, jpeg, jpg, or webp)" >&2
        exit 1
        ;;
esac

if [ -n "$image_path" ] && [ ! -f "$image_path" ]; then
    printf '%s\n' "Image not found: $image_path" >&2
    exit 1
fi

mkdir -p "$result_dir"

request() {
    url=$1
    set -- --fail-with-body --show-error --silent
    if [ -n "$api_key" ]; then
        set -- "$@" -H "X-API-Key: $api_key"
    fi
    curl "$@" "$url"
}

printf 'Server: %s\n' "$server_url"
printf 'Health: '
request "$server_url/healthz"
printf '\nConfig:\n'
request "$server_url/api/config"
printf '\n\n'

process_image() {
    input=$1
    filename=$(basename "$input")
    stem=${filename%.*}
    model_label=${model:-default}
    model_label=$(printf '%s' "$model_label" | tr '/ _' '---')
    output="$result_dir/$stem-$model_label-removed.$output_extension"

    set -- \
        --fail-with-body \
        --show-error \
        --silent \
        --remove-on-error \
        -F "image=@$input" \
        -F "threshold=$threshold" \
        -F "format=$format" \
        --output "$output"
    if [ -n "$api_key" ]; then
        set -- "$@" -H "X-API-Key: $api_key"
    fi
    if [ -n "$model" ]; then
        set -- "$@" -F "model=$model"
    fi

    printf 'Processing %s with %s... ' "$input" "$model_label"
    curl "$@" "$server_url/api/remove-bg"
    printf '%s\n' "$output"
}

if [ -n "$image_path" ]; then
    process_image "$image_path"
else
    found=false
    for input in test-images/*.jpg test-images/*.jpeg test-images/*.png test-images/*.webp; do
        [ -f "$input" ] || continue
        found=true
        process_image "$input"
    done
    if [ "$found" = false ]; then
        printf '%s\n' "No supported images found in test-images/" >&2
        exit 1
    fi
fi
