#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

MODEL="${SHRANK_MODEL:-mlx-community/gemma-4-26b-a4b-it-4bit}"
EMBED_MODEL="${SHRANK_EMBED_MODEL:-mlx-community/all-MiniLM-L6-v2-4bit}"
HOST="${SHRANK_VLLM_HOST:-127.0.0.1}"
PORT="${SHRANK_VLLM_PORT:-8000}"

uv sync --quiet

echo "Starting vllm-mlx on ${HOST}:${PORT}"
echo "  Model:     ${MODEL}"
echo "  Embedding: ${EMBED_MODEL}"
exec uv run vllm-mlx serve "$MODEL" \
    --embedding-model "$EMBED_MODEL" \
    --host "$HOST" \
    --port "$PORT"
