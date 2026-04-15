# Shrank

Private, local-first document archive for paper mail. Local AI extracts, categorizes, and indexes scanned documents. Zero cloud.

## Architecture

Four components, each in its own directory:

| Component | Directory | Tech | Port |
|-----------|-----------|------|------|
| Backend API | `server/` | Rust / Axum / SQLite | `:3420` |
| Inference sidecar | `inference/` | Python / FastAPI / mlx-vlm | `:3421` (localhost only) |
| Web UI | `web/` | React / Vite / Tailwind | `:5173` (dev) |
| iOS app | `ios/` | Swift / SwiftUI / GRDB | — |

## Spec and plans

- `docs/SHRANK.md` — authoritative architecture spec (data model, API contracts, search, sync, prompts)
- `docs/PLAN-BACKEND.md` — backend build plan
- `docs/PLAN-INFERENCE.md` — inference sidecar build plan
- `docs/PLAN-WEB.md` — web UI build plan
- `docs/PLAN-IOS.md` — iOS app build plan

Always read the relevant plan and `docs/SHRANK.md` before making changes.

## Shared contracts

The sidecar API contract (SHRANK.md Section 5.2) and the REST API endpoints (Section 8.1) are the integration boundaries. Changes to these affect multiple components — don't modify them without considering the other side.

## Conventions

- IDs are ULIDs everywhere (sortable, timestamp-embedded)
- Dates are ISO 8601 strings
- JSON columns in SQLite store arrays/objects as TEXT
- Auth: `Authorization: Bearer <key>` header on all API requests
- Tags are always lowercase English with underscores: `health_insurance`, `premium_notice`
- Image paths are relative to `data_dir`, constructed from ULIDs — never from user input

## Dev commands

```bash
# Backend
cd server && cargo run

# Inference sidecar
cd inference && source .venv/bin/activate && uvicorn server:app --host 127.0.0.1 --port 3421

# Web UI
cd web && npm run dev

# iOS
open ios/Shrank.xcodeproj
```
