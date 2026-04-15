# Plan: Web UI (React / Vite / Tailwind)

You are building the Shrank web interface — the browser-based UI for browsing, searching, and managing the document archive.

**Read `docs/SHRANK.md` first.** It has the full spec: feature set, project structure, libraries, and API endpoints (Sections 8.4 and 8.1).

## What you own

- React/Vite app with TailwindCSS
- Dashboard: recent documents, processing queue status, storage stats, tag cloud
- Document browser: infinite scroll grid with thumbnails, click to open detail
- Search: search bar with instant results, faceted filters (sender, date range, type, tags) as sidebar
- Document detail: full-size image viewer (zoom/pan), all extracted metadata, editable fields, related documents
- Graph explorer: interactive d3-force visualization of document relationships
- Manual upload: drag-and-drop images
- Bulk actions: re-process, delete, export as ZIP
- Typed API client (fetch-based, no axios) and React Query hooks

## What you do NOT own

Three other agents are working in parallel:

- **Backend** — Rust/Axum on port `3420`. You talk to it via REST API. Don't build this.
- **Inference sidecar** — Python/FastAPI on port `3421`. You never talk to this directly. Don't build this.
- **iOS app** — Swift/SwiftUI. Separate client, same backend API. Don't build this.

## Key interfaces to respect

The backend REST API (Section 8.1) is your data source. Key endpoints:

- `GET/POST /api/documents` — list, create
- `GET/PUT/DELETE /api/documents/:id` — detail, update, delete
- `GET /api/search?q=...` — unified search with facets
- `GET /api/documents/:id/related?depth=2` — graph traversal
- `GET /api/images/original/:id`, `/api/images/thumbnail/:id` — images
- `GET /api/stats` — dashboard data
- `POST /api/documents/:id/reprocess` — re-run extraction

All requests include `Authorization: Bearer <key>` header.

## Project structure

Put everything under `web/` in the repo root. See Section 8.4 for the layout.

## Dev setup

Vite dev server on `:5173` with proxy to backend at `:3420`. In production, `vite build` output is served as static files by the backend.

## Start with

1. `npm create vite@latest` under `web/`, add Tailwind, React Router, React Query, lucide-react
2. Layout: sidebar + header shell
3. API client (`web/src/api/client.ts`) and TypeScript types matching the backend schemas
4. Document list page with thumbnail grid
5. Document detail page with image viewer and metadata
6. Search page with results and facet filters
7. Dashboard with stats
8. Upload drop zone
9. Graph explorer (d3-force)
