# Plan: Inference Sidecar (Python / mlx-vlm)

You are building the AI inference sidecar — the component that runs Gemma 4 on Apple Silicon and extracts structured data from document images.

**Read `docs/SHRANK.md` first.** It has the full spec: sidecar architecture (Section 5), API contract (Section 5.2), prompt engineering (Section 11), and project structure (Section 8.2).

## What you own

- Python/FastAPI server on `127.0.0.1:3421` (localhost only, never exposed to network)
- `POST /extract` — receive a base64 document image, run Gemma 4 vision, return structured JSON
- `POST /embed` — receive text, return embedding vector (via Ollama or sentence-transformers)
- `GET /health` — report model status, GPU memory usage
- Prompt engineering: the extraction prompt that produces reliable structured JSON (Section 11)
- JSON response parsing: extract valid JSON from LLM output, handle markdown fences, partial responses
- Startup: load model into GPU memory, keep it warm between requests

## What you do NOT own

Three other agents are working in parallel:

- **Backend** — Rust/Axum on port `3420`. It calls your HTTP endpoints. Don't build this.
- **Web UI** — React/Vite. No direct interaction with you. Don't build this.
- **iOS app** — Swift/SwiftUI. No direct interaction with you. Don't build this.

## Key interfaces to respect

The sidecar API contract (Section 5.2) is the shared interface with the backend. The request/response schemas must match exactly so the backend agent's client works with your server.

**Extract response schema** (must match):
```
language, sender, sender_normalized, document_date, document_type, subject, summary,
extracted_text, amounts[], dates[], reference_ids[], tags[], entities[],
related_references[], confidence, extraction_notes
```

**Embed response schema**: `{ embedding: float[], model: string, dimensions: int }`

## Project structure

Put everything under `inference/` in the repo root. See Section 8.2 for the layout.

## Start with

1. `pyproject.toml` with dependencies (fastapi, uvicorn, mlx-vlm, pydantic)
2. FastAPI app with the three endpoints
3. Extraction prompt from Section 11
4. JSON parser that handles LLM output quirks (fences, trailing text, partial JSON)
5. Embedding endpoint (Ollama subprocess or httpx call)
6. Health endpoint with GPU memory reporting
7. `run.sh` startup script
