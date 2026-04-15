# Plan: Rust Backend (Axum)

You are building the Shrank backend — the central server that all other components talk to.

**Read `docs/SHRANK.md` first.** It has the full spec: data model, API endpoints, search architecture, sync protocol, and crate structure (Section 8.1).

## What you own

- Rust/Axum HTTP server on port `3420`
- SQLite database: documents table, FTS5 index, sqlite-vec, entities, graph edges, sync cursors (Section 4)
- All REST API endpoints (Section 8.1 — API Endpoints)
- Image storage: save originals, generate thumbnails via libvips
- Inference client: HTTP calls to vllm-mlx at `127.0.0.1:8000` (OpenAI-compatible API)
- Extraction prompt building and LLM response parsing (`server/src/inference/prompt.rs`, `extraction.rs`)
- Post-extraction relationship inference (Section 5.4)
- Search: query router, FTS5, structured filters, sqlite-vec, graph traversal, RRF fusion (Section 6)
- Sync protocol: delta sync for the iOS app (Section 7)
- API key auth middleware (Section 13)
- Serve the web UI as static files in production via `tower-http::services::ServeDir`
- Configuration via TOML (Section 12.1)

## What you do NOT own

Three other agents are working in parallel:

- **vllm-mlx** — Runs separately on port `8000`. You call its OpenAI-compatible API (`/v1/chat/completions`, `/v1/embeddings`). Don't build this.
- **Web UI** — React/Vite. In dev it runs on port `5173` and proxies to you. In production you serve its `dist/` as static files. Don't build this.
- **iOS app** — Swift/SwiftUI. It talks to your REST API over Tailscale. Don't build this.

## Key interfaces to respect

The REST API endpoints (Section 8.1) are the shared contract with the web UI and iOS app.

## Project structure

Put everything under `server/` in the repo root. See Section 8.1 for the crate structure.

## Start with

1. `cargo init` under `server/`
2. SQLite schema + migrations
3. `POST /api/documents` (multipart upload) + image storage + thumbnail generation
4. Inference client calling vllm-mlx
5. `GET /api/documents`, `GET /api/documents/:id`
6. Search endpoints
7. Sync endpoints
8. Auth middleware
