# Shrank

Private, local-first document archive for paper mail. Local AI extracts, categorizes, and indexes scanned documents. Zero cloud.

## Architecture

Three components + vllm-mlx for inference:

| Component | Directory | Tech | Port |
|-----------|-----------|------|------|
| Backend API | `server/` | Rust / Axum / SQLite | `:3420` |
| Web UI | `web/` | React / Vite / Tailwind | `:5173` (dev) |
| iOS app | `ios/` | Swift / SwiftUI / GRDB | — |
| vllm-mlx | (external) | MLX / Gemma 4 | `:8000` (localhost) |

The Rust backend calls vllm-mlx directly via its OpenAI-compatible API (`/v1/chat/completions`, `/v1/embeddings`). Prompt building, JSON parsing, and relationship inference all happen in Rust. The `inference/` directory contains legacy Python sidecar code (no longer used at runtime).

## Spec and plans

- `docs/SHRANK.md` — authoritative architecture spec (data model, API contracts, search, sync, prompts)
- `docs/PLAN-BACKEND.md` — backend build plan
- `docs/PLAN-INFERENCE.md` — inference sidecar build plan
- `docs/PLAN-WEB.md` — web UI build plan
- `docs/PLAN-IOS.md` — iOS app build plan

Always read the relevant plan and `docs/SHRANK.md` before making changes.

## Shared contracts

The REST API endpoints (Section 8.1) are the integration boundary for the web UI and iOS app. The Rust backend talks to vllm-mlx via the OpenAI-compatible API.

## Conventions

- IDs are ULIDs everywhere (sortable, timestamp-embedded)
- Dates are ISO 8601 strings
- JSON columns in SQLite store arrays/objects as TEXT
- Auth: `Authorization: Bearer <key>` header on all API requests
- Tags are always lowercase English with underscores: `health_insurance`, `premium_notice`
- Image paths are relative to `data_dir`, constructed from ULIDs — never from user input

## Dev commands

```bash
# vllm-mlx (must be running before backend)
vllm-mlx serve mlx-community/gemma-4-26b-a4b-it-4bit --embedding-model mlx-community/all-MiniLM-L6-v2-4bit

# Backend
SHRANK_CONFIG=~/.config/shrank/config.toml cargo run   # from server/

# Web UI
cd web && npm run dev

# iOS
open ios/Shrank.xcodeproj
```
