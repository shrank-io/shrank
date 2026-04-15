# Plan: Inference (vllm-mlx)

The inference layer runs Gemma 4 on Apple Silicon via vllm-mlx. The Rust backend calls its OpenAI-compatible API directly — there is no Python sidecar.

**Read `docs/SHRANK.md` first.** It has the full spec: inference architecture (Section 5), prompt engineering (Section 11), and project structure (Section 8.2).

## What you own

- `inference/run.sh` — launcher script for vllm-mlx
- `inference/pyproject.toml` — vllm-mlx dependency (managed by uv)
- Model selection and configuration via env vars

## How it works

vllm-mlx runs as a standalone server on `127.0.0.1:8000` (localhost only). The Rust backend calls it via:

- `POST /v1/chat/completions` — vision extraction (base64 image + extraction prompt) and chat
- `POST /v1/embeddings` — text embeddings for semantic search
- `GET /v1/models` — health check

Prompt building, JSON response parsing, and relationship inference all happen in the Rust backend (`server/src/inference/`).

## What you do NOT own

- **Rust backend** (`server/src/inference/`) — prompt construction, JSON parsing, relationship inference. These used to live in Python but have been moved to Rust.
- **Web UI** — no direct interaction with inference.
- **iOS app** — no direct interaction with inference.

## Configuration

Environment variables for `run.sh`:

| Variable | Default | Description |
|----------|---------|-------------|
| `SHRANK_MODEL` | `mlx-community/gemma-4-26b-a4b-it-4bit` | Vision/chat model |
| `SHRANK_EMBED_MODEL` | `mlx-community/all-MiniLM-L6-v2-4bit` | Embedding model |
| `SHRANK_VLLM_HOST` | `127.0.0.1` | Bind address |
| `SHRANK_VLLM_PORT` | `8000` | Port |

## Start with

```bash
cd inference && ./run.sh
```

Or directly:

```bash
vllm-mlx serve mlx-community/gemma-4-26b-a4b-it-4bit \
    --embedding-model mlx-community/all-MiniLM-L6-v2-4bit
```
