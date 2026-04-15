#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

uv sync --quiet

echo "Starting Shrank inference sidecar on 127.0.0.1:3421..."
exec uv run uvicorn server:app --host 127.0.0.1 --port 3421
