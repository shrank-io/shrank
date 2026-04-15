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

## Quick Start

```bash
git clone https://github.com/shrank-io/shrank.git
cd shrank
make setup     # Install dependencies
make dev       # Start all services
# Open http://localhost:5173
```

See [docs/SHRANK.md](docs/SHRANK.md) for the full architecture document.

## Status

Early development. See the [implementation phases](docs/SHRANK.md#10-implementation-phases) for the roadmap.

## License

[Apache-2.0](LICENSE)
