# Plan: iOS App (Swift / SwiftUI)

You are building the Shrank iOS app — the primary capture device for photographing paper mail and the mobile interface for browsing the archive offline.

**Read `docs/SHRANK.md` first.** It has the full spec: feature set, project structure, sync protocol, and security model (Sections 8.3, 7, and 13.3).

## What you own

- Swift/SwiftUI app with VisionKit document scanner
- Camera capture: full-screen scanner with document edge detection (`VNDocumentCameraViewController`), multi-page support
- Local SQLite database (via GRDB.swift): mirrors server schema for offline access
- Local FTS5 search
- Local image storage + thumbnail generation
- Upload queue with retry logic (pending → uploading → confirmed → failed)
- Sync engine: delta sync with server, pull metadata + thumbnails + embeddings
- Tailscale reachability detection (NWPathMonitor / NWBrowser)
- Background sync via BGTaskScheduler
- Document list, detail, and search views
- Settings: server address (Tailscale hostname), API key (stored in Keychain), storage usage, manual sync trigger

## What you do NOT own

Three other agents are working in parallel:

- **Backend** — Rust/Axum on port `3420`. You talk to it over Tailscale via REST API. Don't build this.
- **vllm-mlx** — Inference server on port `8000`. The backend calls it directly. You never interact with this.
- **Web UI** — React/Vite. Separate client, same backend API. Don't build this.

## Key interfaces to respect

The backend REST API (Section 8.1) and sync protocol (Section 7) are the shared contracts:

- `POST /api/documents` — multipart upload (image + captured_at + device_id)
- `GET /api/sync?since=<cursor>&limit=50` — delta sync
- `POST /api/sync/register` — register this device
- `GET /api/images/thumbnail/:id` — download thumbnails for local cache
- `GET /api/search?q=...` — remote search (when online)
- `GET /api/health` — reachability check
- All requests include `Authorization: Bearer <key>` header.

## Project structure

Put everything under `ios/` in the repo root. See Section 8.3 for the layout.

## Start with

1. Xcode project under `ios/` with SwiftUI lifecycle
2. Local SQLite database with GRDB.swift, schema mirroring server
3. VisionKit document scanner integration
4. Local image storage + thumbnail generation
5. API client (URLSession-based) with auth
6. Upload queue with retry
7. Sync engine: register device, delta pull
8. Document list and detail views
9. Local FTS5 search
10. Background sync via BGTaskScheduler
11. Settings view with Keychain-stored API key
