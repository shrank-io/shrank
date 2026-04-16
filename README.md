# Shrank

**Your paper pile just shrank.**

Private, local-first document archive. Snap a photo of any paper mail — local AI extracts, categorizes, and indexes it — searchable from your phone and web UI. Zero cloud. Your data never leaves your network.

> **Shrank** /ʃræŋk/ — from German *Schrank* (cabinet) meets English *shrank* (past tense of shrink). Your documents go into the cabinet; your paper pile shrinks.

## How It Works

1. **Capture** — Photograph a document with the iOS app
2. **Extract** — Local LLM (Gemma 4 on Apple Silicon) reads the document and extracts sender, dates, amounts, reference numbers, and tags
3. **Search** — Find anything via keyword, semantic, or structured search from phone or web UI
4. **Connect** — Documents are automatically linked via shared entities (policy numbers, senders, references)

## Architecture

```
iPhone (SwiftUI)  ◄──  Tailscale  ──►  Mac (M1 Max)
  Camera capture                         Rust/Axum backend (SQLite + FTS5 + sqlite-vec)
  Local SQLite+FTS5                      vllm-mlx (Gemma 4 26B MoE)
  Offline search                         React web UI
```

- **Backend**: Rust / Axum — single binary, SQLite for everything
- **AI**: Gemma 4 26B MoE via vllm-mlx on Apple Silicon
- **Search**: 4-layer hybrid — structured filters, full-text (FTS5), semantic (sqlite-vec), knowledge graph (recursive CTEs)
- **Networking**: Tailscale (WireGuard) — encrypted P2P, no port forwarding
- **iOS**: Swift / SwiftUI with VisionKit document scanner, offline-first with background sync

## Running (Development)

Three services need to run, each in its own terminal. Start them in this order:

### 1. Inference sidecar (vllm-mlx) — port 8000

```bash
cd inference
uv run vllm-mlx serve mlx-community/gemma-4-26b-a4b-it-4bit \
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

See [docs/SHRANK.md](docs/SHRANK.md) for the full architecture document.

## Status

Early development. See the [implementation phases](docs/SHRANK.md#10-implementation-phases) for the roadmap.

## License

[Apache-2.0](LICENSE)
