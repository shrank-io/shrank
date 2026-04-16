# Shrank

**Your paper pile just shrank.**

Private, local-first document archive. Snap a photo of any paper mail — local AI extracts, categorizes, and indexes it — searchable from your phone and web UI. Zero cloud. Your data never leaves your network.

> **Shrank** /ʃræŋk/ — from German *Schrank* (cabinet) meets English *shrank* (past tense of shrink). Your documents go into the cabinet; your paper pile shrinks.

## Experiment Status

**This is an experiment, not a product.** The entire codebase was built using [Claude Code](https://claude.ai/claude-code) with Claude Opus 4.6 (1M context). Four sessions ran in parallel — one per component (backend, web UI, iOS app, inference sidecar) — each in its own terminal. A fifth session integrated everything and fixed cross-component issues.

### What worked

The result is a functioning demo / proof of concept. You can upload a document photo, a local LLM (Gemma 4 26B on Apple Silicon) extracts structured metadata, and you can browse, search, and view documents in a web UI. The iOS app captures and syncs. The knowledge graph connects documents via shared entities. It works end to end.

### What didn't

Claude Code with Opus 4.6 gets you the 80% solution. The remaining 20% is where you'd normally spend 80% of your time: small bugs across all components, edge cases in LLM output parsing (Gemma 4's thinking channel tokens leaking into responses), API response shapes not matching what the frontend expects, the graph visualization needing data shape fixes, tag rendering breaking because the backend returned strings where the frontend expected objects. Each individually small, but they add up.

### Investment

- **Personal time**: ~4 hours of prompting, reviewing, and debugging
- **Wall clock**: ~28 hours (mostly waiting for LLM inference and Claude responses)
- **Claude tokens**: significant (Opus 4.6 with 1M context, multiple long sessions)
- **Lines of code generated**: ~5,000+ across Rust, TypeScript, Swift, Python

### Takeaway

Claude Code is exceptional at scaffolding and getting a complex multi-component system to a working demo. It writes competent Rust, TypeScript, and Swift. It understands architectural specs and follows them. But the last mile — making everything actually robust, handling real-world LLM output quirks, getting the frontend and backend contracts perfectly aligned — still requires human debugging and iteration. The tool is a force multiplier, not a replacement.

This experiment will be repeated with other models for comparison.

## How It Works

1. **Capture** — Photograph a document with the iOS app
2. **Extract** — Local LLM (Gemma 4 on Apple Silicon) reads the document in two passes: first OCR (image to markdown), then structured extraction (markdown to JSON)
3. **Search** — Find anything via keyword, semantic, or structured search from phone or web UI
4. **Connect** — Documents are automatically linked via shared entities (policy numbers, senders, references)

## Architecture

```
iPhone (SwiftUI)  <--  Tailscale  -->  Mac (M1 Max)
  Camera capture                         Rust/Axum backend (SQLite + FTS5 + sqlite-vec)
  Local SQLite+FTS5                      vllm-mlx (Gemma 4 26B MoE)
  Offline search                         React web UI
```

| Component | Tech | Status |
|-----------|------|--------|
| Backend API | Rust / Axum / SQLite | Working (bugs remain) |
| Inference | Gemma 4 26B MoE via vllm-mlx | Working (thinking channel issues) |
| Web UI | React / Vite / Tailwind | Working (UI bugs) |
| iOS app | Swift / SwiftUI / GRDB | Working (sync issues) |
| Search | FTS5 + structured filters | Working (semantic search not wired) |
| Knowledge graph | Recursive CTEs + entity linking | Working (sparse with few docs) |

## Running (Development)

Three services need to run, each in its own terminal. Start them in this order:

### 1. Inference (vllm-mlx) — port 8000

```bash
cd inference
uv run vllm-mlx serve mlx-community/gemma-4-26b-a4b-it-8bit \
  --host 127.0.0.1 --port 8000
```

Wait until you see `Uvicorn running on http://127.0.0.1:8000` before starting the backend.

### 2. Backend API — port 3420

```bash
cd server
SHRANK_CONFIG=~/.config/shrank/config.toml cargo run
```

On first run this creates the SQLite database at `~/.local/share/shrank/shrank.db` and runs migrations.

Requires a config file at the path above. Minimal config:

```toml
[server]
host = "0.0.0.0"
port = 3420
data_dir = "~/.local/share/shrank"

[auth]
api_key = "your-secret-key"

[inference]
endpoint = "http://127.0.0.1:8000"

[embeddings]
endpoint = "http://127.0.0.1:8000"
```

### 3. Web UI — port 5173

```bash
cd web
npm run dev
```

Open [http://localhost:5173](http://localhost:5173) in your browser.

## Spec

See [docs/SHRANK.md](docs/SHRANK.md) for the full architecture document.

## License

[Apache-2.0](LICENSE)
