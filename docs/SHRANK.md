# Shrank 📬

## Your paper pile just shrank.

**Private, local-first document archive. Snap a photo of any paper mail → local AI extracts, categorizes, and indexes it → searchable from your phone and web UI. Zero cloud. Your data never leaves your network.**

> **Shrank** /ʃræŋk/ — from German *Schrank* (cabinet) meets English *shrank* (past tense of shrink). Your documents go into the cabinet; your paper pile shrinks.

- **Website**: [shrank.io](https://shrank.io)
- **GitHub**: [github.com/shrank-io/shrank](https://github.com/shrank-io/shrank)
- **License**: Apache-2.0

---

## Table of Contents

1. [Vision & Principles](#1-vision--principles)
2. [Architecture Overview](#2-architecture-overview)
3. [Technology Stack](#3-technology-stack)
4. [Data Model](#4-data-model)
5. [Inference Pipeline — Gemma 4 via mlx-vlm](#5-inference-pipeline--gemma-4-via-mlx-vlm)
6. [Search Architecture](#6-search-architecture)
7. [Sync Protocol](#7-sync-protocol)
8. [Component Specs](#8-component-specs)
   - 8.1 [Rust Backend (Axum)](#81-rust-backend-axum)
   - 8.2 [Inference Sidecar (Python / mlx-vlm)](#82-inference-sidecar-python--mlx-vlm)
   - 8.3 [iOS App (Swift / SwiftUI)](#83-ios-app-swift--swiftui)
   - 8.4 [Web UI (React / Vite)](#84-web-ui-react--vite)
9. [Project Structure](#9-project-structure)
10. [Implementation Phases](#10-implementation-phases)
11. [Gemma 4 Prompt Engineering](#11-gemma-4-prompt-engineering)
12. [Configuration & Deployment](#12-configuration--deployment)
13. [Security Model](#13-security-model)
14. [Future Considerations](#14-future-considerations)
15. [Open-Source Strategy](#15-open-source-strategy)

---

## 1. Vision & Principles

Shrank is a fully private, self-hosted document archive for paper mail. The name is a play on the German word *Schrank* (cabinet/wardrobe — where you store your important documents) and the English word *shrank* (your paper pile just shrank). It is designed for people who receive important documents — insurance letters, tax notices, bank statements, contracts — and want them digitized, searchable, and organized without trusting any cloud service.

### Core Principles

- **Privacy-first**: All data stays on your local network. No cloud services, no telemetry, no accounts. Documents are processed by a local LLM running on your own hardware.
- **Zero-config intelligence**: No predefined categories, no language settings, no OCR configuration. The LLM detects language, extracts entities, infers categories, and builds relationships — all from the document image alone.
- **Offline-first**: The phone app works fully offline. Documents queue locally and sync when the server becomes reachable. The phone maintains a complete searchable copy of the archive.
- **Single-user simplicity**: This is a personal tool. No multi-tenancy, no RBAC, no user management. One person, one archive.
- **LLM-agnostic**: Default stack is Gemma 4 26B MoE via mlx-vlm on Apple Silicon, but the inference interface is a clean HTTP contract — swap in Ollama, vLLM, or any OpenAI-compatible endpoint.

---

## 2. Architecture Overview

```
┌─────────────────────┐          Tailscale (WireGuard)          ┌─────────────────────────────────┐
│   iPhone (Swift)    │◄──────────────────────────────────────►│        Mac (M1 Max)             │
│                     │              HTTPS/mTLS                 │                                 │
│  Camera Capture     │                                         │  ┌───────────────────────────┐  │
│  Local SQLite+FTS5  │    POST /api/documents (image upload)   │  │  Rust/Axum Backend        │  │
│  sqlite-vec         │   ─────────────────────────────────►    │  │                           │  │
│  Thumbnail cache    │                                         │  │  REST API                 │  │
│  Offline search     │    GET /api/sync (metadata+thumbnails)  │  │  SQLite + FTS5            │  │
│  Pending queue      │   ◄─────────────────────────────────    │  │  sqlite-vec               │  │
│                     │                                         │  │  Graph edges (SQL)        │  │
└─────────────────────┘                                         │  │  Image storage            │  │
                                                                │  │  Sync engine              │  │
                                                                │  └──────────┬────────────────┘  │
                                                                │             │ HTTP localhost     │
                                                                │  ┌──────────▼────────────────┐  │
                                                                │  │  mlx-vlm Sidecar          │  │
                                                                │  │  (Python/FastAPI)         │  │
                                                                │  │                           │  │
                                                                │  │  Gemma 4 26B MoE          │  │
                                                                │  │  Vision + Extraction      │  │
                                                                │  │  Embedding generation     │  │
                                                                │  └───────────────────────────┘  │
                                                                │                                 │
                                                                │  ┌───────────────────────────┐  │
                                                                │  │  React Web UI (Vite)      │  │
                                                                │  │  localhost:5173            │  │
                                                                │  │  served by Axum in prod   │  │
                                                                │  └───────────────────────────┘  │
                                                                └─────────────────────────────────┘
```

### Data Flow

1. **Capture**: User photographs a document with the iOS app.
2. **Local storage**: Image saved to phone filesystem; metadata stub created in local SQLite.
3. **Queue**: Document added to sync queue with status `pending`.
4. **Sync**: When Tailscale peer (Mac) is reachable, phone uploads image via `POST /api/documents`.
5. **Ingest**: Axum backend receives image, stores original + generates thumbnail.
6. **Extract**: Backend sends image to mlx-vlm sidecar → Gemma 4 returns structured JSON (language, sender, date, type, categories/tags, entities, summary, extracted text).
7. **Embed**: Backend requests embedding vector for the extracted text (via sidecar or dedicated embedding model).
8. **Index**: Metadata written to SQLite, text indexed in FTS5, vector stored in sqlite-vec, entity edges created in graph tables.
9. **Sync-back**: Next time phone syncs, it pulls new metadata + thumbnail + embedding for local search.

---

## 3. Technology Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **Backend API** | Rust / Axum | Single binary, high performance, shared ecosystem with Shrike. Axum's tower middleware for auth, logging, error handling. |
| **Database** | SQLite (via rusqlite) | Zero-config, single-file, embedded. Perfect for single-user. Backup = copy one file. |
| **Full-text search** | SQLite FTS5 | Built into SQLite. BM25 ranking. Supports custom tokenizers for German compound words. |
| **Vector search** | sqlite-vec | SQLite extension for vector similarity search. No extra process. Sub-millisecond for <100k docs. |
| **Graph relationships** | SQLite tables + recursive CTEs | Entities and document relationships stored as edges. Graph traversal via `WITH RECURSIVE`. No Neo4j overhead. |
| **LLM inference** | mlx-vlm + FastAPI sidecar | Gemma 4 26B MoE on Apple Silicon via MLX. Native Metal acceleration. ~75 tok/s on M1 Max. |
| **Embeddings** | nomic-embed-text (via Ollama) OR Gemma 4 E4B | Lightweight embedding model for semantic search vectors. Runs alongside main model. |
| **iOS app** | Swift / SwiftUI | Native camera integration, background sync, offline SQLite. No Flutter overhead for a capture-first app. |
| **Web UI** | React / Vite / TailwindCSS | Modern, fast dev experience. Served as static files by Axum in production. |
| **Networking** | Tailscale | WireGuard-based mesh VPN. Encrypted P2P. Already proven in irrigation project. |
| **Image processing** | libvips (via Rust bindings) | Fast thumbnail generation, EXIF handling, format conversion. Much lighter than ImageMagick. |

---

## 4. Data Model

### 4.1 Core Tables (SQLite)

```sql
-- Documents: the primary entity
CREATE TABLE documents (
    id              TEXT PRIMARY KEY,  -- ULID (sortable, timestamp-embedded)
    created_at      TEXT NOT NULL,     -- ISO 8601
    updated_at      TEXT NOT NULL,
    captured_at     TEXT NOT NULL,     -- when photo was taken (from EXIF or phone clock)
    synced_at       TEXT,              -- when uploaded from phone

    -- Image storage (relative paths)
    original_path   TEXT NOT NULL,     -- e.g. "originals/01J5K3.../scan.jpg"
    thumbnail_path  TEXT NOT NULL,     -- e.g. "thumbnails/01J5K3.../thumb.webp"

    -- Processing state
    status          TEXT NOT NULL DEFAULT 'pending',
    -- pending → processing → complete → error
    processing_error TEXT,
    raw_llm_response TEXT,            -- full JSON response from Gemma, for debugging/reprocessing

    -- Extracted metadata (populated after LLM processing)
    language        TEXT,              -- ISO 639-1: "de", "en", etc.
    sender          TEXT,              -- extracted sender/organization name
    sender_normalized TEXT,            -- normalized for grouping: "AOK Bayern" not "AOK Bayern - Die Gesundheitskasse"
    document_date   TEXT,              -- date ON the document (not capture date)
    document_type   TEXT,              -- invoice, letter, policy, statement, contract, receipt, notification, certificate
    subject         TEXT,              -- one-line subject/summary extracted by LLM
    extracted_text  TEXT,              -- full OCR text from the document

    -- Structured data (JSON columns)
    amounts         TEXT,              -- JSON array: [{"value": 127.50, "currency": "EUR", "label": "Monatsbeitrag"}]
    dates           TEXT,              -- JSON array: [{"date": "2026-06-01", "label": "Fälligkeitsdatum"}]
    reference_ids   TEXT,              -- JSON array: [{"type": "policy", "value": "VN-123456"}, {"type": "iban", "value": "DE89..."}]

    -- Categorization (LLM-generated, emergent)
    tags            TEXT,              -- JSON array: ["health_insurance", "aok", "premium_notice"]
    confidence      REAL               -- LLM self-reported extraction confidence 0.0-1.0
);

-- Full-text search index
CREATE VIRTUAL TABLE documents_fts USING fts5(
    sender,
    subject,
    extracted_text,
    tags,
    content='documents',
    content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 2'
    -- unicode61 tokenizer handles German umlauts, compounds need post-processing
);

-- Triggers to keep FTS in sync
CREATE TRIGGER documents_ai AFTER INSERT ON documents BEGIN
    INSERT INTO documents_fts(rowid, sender, subject, extracted_text, tags)
    VALUES (new.rowid, new.sender, new.subject, new.extracted_text, new.tags);
END;

CREATE TRIGGER documents_ad AFTER DELETE ON documents BEGIN
    INSERT INTO documents_fts(documents_fts, rowid, sender, subject, extracted_text, tags)
    VALUES ('delete', old.rowid, old.sender, old.subject, old.extracted_text, old.tags);
END;

CREATE TRIGGER documents_au AFTER UPDATE ON documents BEGIN
    INSERT INTO documents_fts(documents_fts, rowid, sender, subject, extracted_text, tags)
    VALUES ('delete', old.rowid, old.sender, old.subject, old.extracted_text, old.tags);
    INSERT INTO documents_fts(rowid, sender, subject, extracted_text, tags)
    VALUES (new.rowid, new.sender, new.subject, new.extracted_text, new.tags);
END;
```

### 4.2 Vector Embeddings (sqlite-vec)

```sql
-- Vector table for semantic search
-- Dimension depends on embedding model: nomic-embed-text = 768, Gemma = varies
CREATE VIRTUAL TABLE documents_vec USING vec0(
    document_id TEXT PRIMARY KEY,
    embedding FLOAT[768]
);
```

### 4.3 Knowledge Graph (Entity-Relationship)

```sql
-- Entities: things that appear across multiple documents
CREATE TABLE entities (
    id              TEXT PRIMARY KEY,  -- ULID
    entity_type     TEXT NOT NULL,     -- "organization", "policy", "person", "vehicle", "property", "account"
    value           TEXT NOT NULL,     -- "AOK Bayern", "VN-123456", "B-XX-1234"
    display_name    TEXT,              -- human-friendly label
    metadata        TEXT,              -- JSON: extra info about this entity
    created_at      TEXT NOT NULL,
    UNIQUE(entity_type, value)
);

-- Links between documents and entities
CREATE TABLE document_entities (
    document_id     TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    entity_id       TEXT NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    role            TEXT NOT NULL,     -- "sender", "referenced_policy", "referenced_vehicle", "beneficiary"
    confidence      REAL DEFAULT 1.0,
    PRIMARY KEY (document_id, entity_id, role)
);

-- Direct document-to-document relationships
CREATE TABLE document_edges (
    source_id       TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    target_id       TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    relation_type   TEXT NOT NULL,     -- "references", "follows_up", "renews", "invoices_for", "responds_to"
    confidence      REAL DEFAULT 1.0,
    inferred_by     TEXT NOT NULL,     -- "llm" | "reference_match" | "user"
    created_at      TEXT NOT NULL,
    PRIMARY KEY (source_id, target_id, relation_type)
);

-- Indexes for graph traversal
CREATE INDEX idx_doc_entities_entity ON document_entities(entity_id);
CREATE INDEX idx_doc_entities_doc ON document_entities(document_id);
CREATE INDEX idx_doc_edges_source ON document_edges(source_id);
CREATE INDEX idx_doc_edges_target ON document_edges(target_id);
```

### 4.4 Sync State

```sql
-- Track what the phone has seen (server-side)
CREATE TABLE sync_cursors (
    client_id       TEXT PRIMARY KEY,  -- phone device ID
    last_sync_at    TEXT NOT NULL,
    last_document_id TEXT              -- ULID cursor for pagination
);

-- Pending uploads queue (phone-side, mirrored in iOS SQLite)
-- This same schema exists in the iOS app's local database
CREATE TABLE upload_queue (
    id              TEXT PRIMARY KEY,
    local_image_path TEXT NOT NULL,
    captured_at     TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',
    -- pending → uploading → confirmed → failed
    retry_count     INTEGER DEFAULT 0,
    last_attempt_at TEXT,
    error_message   TEXT
);
```

### 4.5 Graph Traversal Example

```sql
-- "Show me everything related to this document within 2 hops"
WITH RECURSIVE related(doc_id, depth, path) AS (
    -- Start node
    SELECT :start_doc_id, 0, :start_doc_id

    UNION ALL

    -- Follow edges (both directions)
    SELECT
        CASE WHEN de.source_id = r.doc_id THEN de.target_id ELSE de.source_id END,
        r.depth + 1,
        r.path || '→' || CASE WHEN de.source_id = r.doc_id THEN de.target_id ELSE de.source_id END
    FROM related r
    JOIN document_edges de ON de.source_id = r.doc_id OR de.target_id = r.doc_id
    WHERE r.depth < 2
      AND INSTR(r.path, CASE WHEN de.source_id = r.doc_id THEN de.target_id ELSE de.source_id END) = 0

    UNION ALL

    -- Follow entity connections (documents sharing entities)
    SELECT
        de2.document_id,
        r.depth + 1,
        r.path || '⇢' || de2.document_id
    FROM related r
    JOIN document_entities de1 ON de1.document_id = r.doc_id
    JOIN document_entities de2 ON de2.entity_id = de1.entity_id AND de2.document_id != r.doc_id
    WHERE r.depth < 2
      AND INSTR(r.path, de2.document_id) = 0
)
SELECT DISTINCT d.*
FROM related r
JOIN documents d ON d.id = r.doc_id
ORDER BY d.document_date DESC;
```

---

## 5. Inference Pipeline — Gemma 4 via mlx-vlm

### 5.1 Sidecar Architecture

The inference engine runs as a separate Python process alongside the Rust backend. Communication is via HTTP on localhost. This separation means:

- Rust backend can restart independently of the model
- Model stays warm in memory between requests
- Easy to swap inference backends (mlx-vlm → Ollama → vLLM)
- Python ecosystem for ML, Rust ecosystem for everything else

```
Axum Backend                    mlx-vlm Sidecar
     │                               │
     │  POST /extract                 │
     │  {image: base64}  ───────────► │  Load image
     │                                │  Run Gemma 4 26B MoE vision
     │                                │  Parse structured JSON
     │  ◄─────────────────────────    │  Return extraction result
     │  {sender, date, tags, ...}     │
     │                                │
     │  POST /embed                   │
     │  {text: "..."}    ───────────► │  Run embedding model
     │  ◄─────────────────────────    │  Return [768-dim vector]
     │  {embedding: [...]}            │
     │                                │
     │  GET /health                   │
     │               ───────────────► │  Model loaded? GPU ok?
     │  ◄─────────────────────────    │  {status: "ready", model: "gemma-4-26b-a4b-it"}
     │                                │
```

### 5.2 Sidecar API Contract

```
POST /extract
Content-Type: application/json

Request:
{
  "image_base64": "<base64 encoded JPEG/PNG>",
  "existing_tags": ["health_insurance", "aok", "tax", ...],  // for tag consistency
  "existing_senders": ["AOK Bayern", "Deutsche Rentenversicherung", ...]  // for sender normalization
}

Response:
{
  "language": "de",
  "sender": "AOK Bayern",
  "sender_normalized": "AOK Bayern",
  "document_date": "2026-03-15",
  "document_type": "notification",
  "subject": "Beitragsanpassung zum 01.07.2026",
  "extracted_text": "Sehr geehrter Herr ...",
  "amounts": [
    {"value": 274.50, "currency": "EUR", "label": "Neuer Monatsbeitrag"},
    {"value": 262.30, "currency": "EUR", "label": "Bisheriger Beitrag"}
  ],
  "dates": [
    {"date": "2026-07-01", "label": "Gültig ab"},
    {"date": "2026-04-30", "label": "Widerspruchsfrist"}
  ],
  "reference_ids": [
    {"type": "policy", "value": "VN-987654321"},
    {"type": "reference", "value": "BN/2026/03/4567"}
  ],
  "tags": ["health_insurance", "aok", "premium_adjustment", "beitragsanpassung"],
  "entities": [
    {"type": "organization", "value": "AOK Bayern", "role": "sender"},
    {"type": "policy", "value": "VN-987654321", "role": "referenced_policy"}
  ],
  "summary": "AOK Bayern notifies of a premium increase from €262.30 to €274.50 effective July 2026, with objection deadline April 30.",
  "confidence": 0.92,
  "related_references": ["BN/2026/03/4567"]  // references to look up in existing docs
}

POST /embed
Content-Type: application/json

Request:  { "text": "..." }
Response: { "embedding": [0.012, -0.034, ...], "model": "nomic-embed-text", "dimensions": 768 }

GET /health
Response: { "status": "ready", "model": "gemma-4-26b-a4b-it", "backend": "mlx-vlm", "gpu_memory_used_gb": 14.2 }
```

### 5.3 Inference Sidecar Implementation

```python
# inference/server.py — thin FastAPI wrapper around mlx-vlm
# This is the complete sidecar. Keep it minimal.

from fastapi import FastAPI
from pydantic import BaseModel
from mlx_vlm import load, generate
import base64, json, tempfile, os

app = FastAPI()

# Load model once at startup — stays warm in GPU memory
MODEL_PATH = os.environ.get("SHRANK_MODEL", "mlx-community/gemma-4-26b-a4b-it-4bit")
model, processor = load(MODEL_PATH)

class ExtractionRequest(BaseModel):
    image_base64: str
    existing_tags: list[str] = []
    existing_senders: list[str] = []

class EmbedRequest(BaseModel):
    text: str

@app.post("/extract")
async def extract(req: ExtractionRequest):
    # Write image to temp file (mlx-vlm needs file path)
    img_bytes = base64.b64decode(req.image_base64)
    with tempfile.NamedTemporaryFile(suffix=".jpg", delete=False) as f:
        f.write(img_bytes)
        img_path = f.name

    try:
        prompt = build_extraction_prompt(req.existing_tags, req.existing_senders)
        messages = [{"role": "user", "content": [
            {"type": "image", "url": img_path},
            {"type": "text", "text": prompt},
        ]}]
        formatted = processor.tokenizer.apply_chat_template(
            messages, add_generation_prompt=True, tokenize=False
        )
        # Use high token budget for OCR quality
        result = generate(
            model, processor, formatted, [img_path],
            max_tokens=4096, temperature=0.1,
            repetition_penalty=1.1
        )
        parsed = parse_llm_json(result.text)
        return parsed
    finally:
        os.unlink(img_path)

@app.post("/embed")
async def embed(req: EmbedRequest):
    # For embeddings, use a lightweight model via ollama or sentence-transformers
    # This is a placeholder — actual implementation depends on chosen embedding model
    import subprocess, json
    result = subprocess.run(
        ["ollama", "embed", "nomic-embed-text", req.text],
        capture_output=True, text=True
    )
    data = json.loads(result.stdout)
    return {"embedding": data["embedding"], "model": "nomic-embed-text", "dimensions": 768}

@app.get("/health")
async def health():
    return {"status": "ready", "model": MODEL_PATH, "backend": "mlx-vlm"}

def build_extraction_prompt(existing_tags, existing_senders):
    """See Section 11 for full prompt engineering details."""
    # ... (defined in detail in Section 11)
    pass

def parse_llm_json(raw_text):
    """Extract JSON from LLM response, handling markdown fences etc."""
    text = raw_text.strip()
    if text.startswith("```"):
        text = text.split("\n", 1)[1].rsplit("```", 1)[0]
    return json.loads(text)
```

### 5.4 Relationship Inference

After initial extraction, the backend performs a second pass to find connections:

```
1. For each reference_id in the new document:
   → Query existing documents for matching reference_ids
   → If match found: create document_edge (type: "references")

2. For each entity extracted:
   → UPSERT into entities table
   → Create document_entities link
   → Any other documents linked to the same entity = implicit graph connection

3. If sender matches existing sender:
   → Find most recent document from same sender
   → If within 90 days: create document_edge (type: "follows_up", confidence: 0.6)
   → If reference numbers match: upgrade to (type: "follows_up", confidence: 0.95)

4. Check for "Bezug auf" / "In Antwort auf" / "Ihr Schreiben vom" patterns:
   → Extract referenced date
   → Find documents from same sender near that date
   → Create document_edge (type: "responds_to")
```

---

## 6. Search Architecture

### 6.1 Four-Layer Search

Every search query can hit up to four search backends, with results fused:

| Layer | Technology | Handles | Example |
|-------|-----------|---------|---------|
| **Structured** | SQL WHERE | Exact filters on metadata | `sender:"AOK" date:>2025-06 type:invoice` |
| **Full-text** | FTS5 / BM25 | Keyword search in document text | `"Beitragsanpassung" OR "premium increase"` |
| **Semantic** | sqlite-vec cosine similarity | Conceptual/fuzzy queries | "anything about my car insurance" |
| **Graph** | Recursive CTEs | Related document traversal | "show me everything connected to this letter" |

### 6.2 Query Router

The Rust backend parses the search query and decides which layers to activate:

```rust
enum SearchIntent {
    Structured(Vec<Filter>),       // detected field:value syntax
    Keyword(String),               // plain text → FTS5
    Semantic(String),              // natural language question → embed + vector search
    GraphTraversal(DocumentId),    // "related to <doc>" → recursive CTE
    Hybrid(Box<SearchIntent>, Box<SearchIntent>),  // combine multiple
}

// Heuristics:
// - Contains ":" → parse as structured filter
// - Contains known field names (sender, date, type, tag) → structured
// - Short keyword-like query → FTS5
// - Question or natural language → semantic
// - "related to" / "connected" / "similar" → graph/semantic
// - Default: FTS5 + semantic hybrid
```

### 6.3 Score Fusion

When multiple layers return results, merge using Reciprocal Rank Fusion (RRF):

```
RRF_score(doc) = Σ  1 / (k + rank_in_layer_i)
                 i

k = 60 (standard constant)
```

This is simple, doesn't require score normalization across layers, and works well in practice.

### 6.4 Search API

```
GET /api/search?q=<query>&limit=20&offset=0

Response:
{
  "results": [
    {
      "document": { ... full document object ... },
      "score": 0.87,
      "match_sources": ["fts5", "semantic"],  // which layers matched
      "highlights": {
        "extracted_text": "...Ihr <mark>Beitrag</mark> wird zum 01.07.2026 <mark>angepasst</mark>..."
      }
    }
  ],
  "facets": {
    "senders": [{"name": "AOK Bayern", "count": 12}, ...],
    "tags": [{"name": "health_insurance", "count": 8}, ...],
    "types": [{"name": "invoice", "count": 5}, ...],
    "years": [{"name": "2026", "count": 15}, ...]
  },
  "total": 42,
  "query_intent": "hybrid(keyword, semantic)"
}
```

---

## 7. Sync Protocol

### 7.1 Network Layer

- Both phone and Mac join the same Tailscale tailnet.
- Mac runs Axum on `0.0.0.0:3420` — accessible via Tailscale IP (e.g., `100.x.y.z:3420`).
- Phone discovers Mac via Tailscale DNS name (e.g., `macbook.tail1234.ts.net:3420`).
- All traffic is encrypted end-to-end by WireGuard (Tailscale).
- Additional API key authentication as defense-in-depth (see Section 13).

### 7.2 Reachability Detection

```swift
// iOS: Background task checks Mac availability
// Runs every 15 minutes when on Wi-Fi (iOS BGTaskScheduler)
func checkServerReachability() async -> Bool {
    let url = URL(string: "https://\(serverHost):3420/api/health")!
    var request = URLRequest(url: url, timeoutInterval: 5)
    request.setValue("Bearer \(apiKey)", forHTTPHeaderField: "Authorization")
    do {
        let (_, response) = try await URLSession.shared.data(for: request)
        return (response as? HTTPURLResponse)?.statusCode == 200
    } catch {
        return false
    }
}
```

### 7.3 Upload Flow

```
Phone                                Server
  │                                     │
  │  1. POST /api/documents             │
  │     multipart/form-data:            │
  │     - image (JPEG, full resolution) │
  │     - captured_at (ISO 8601)        │
  │     - device_id                     │
  │  ──────────────────────────────►    │
  │                                     │  2. Store image
  │                                     │  3. Generate thumbnail
  │                                     │  4. Create document (status: pending)
  │                                     │  5. Queue for LLM processing
  │  ◄──────────────────────────────    │
  │  201 Created                        │
  │  { id: "01J5K3...", status: "pending" }
  │                                     │
  │  ... LLM processes async ...        │
  │                                     │
  │  6. GET /api/sync?since=<cursor>    │
  │  ──────────────────────────────►    │
  │                                     │
  │  ◄──────────────────────────────    │
  │  {                                  │
  │    documents: [                     │
  │      { id, metadata, thumbnail_url, │
  │        embedding, status }          │
  │    ],                               │
  │    next_cursor: "01J5K4..."         │
  │  }                                  │
  │                                     │
  │  7. GET /api/thumbnails/<id>        │
  │  ──────────────────────────────►    │  (download thumbnails for local cache)
  │                                     │
```

### 7.4 Conflict Resolution

Simple last-write-wins. Since this is single-user, conflicts only arise if the user edits metadata on the web UI while the phone is offline and then tries to sync. The server is always the source of truth for metadata. The phone is the source of truth for new captures.

### 7.5 Bandwidth Optimization

- Thumbnails: WebP, 400px wide, typically 20-40KB each.
- Sync responses: Paginated, 50 documents per page.
- Embeddings: 768 × 4 bytes = ~3KB per document.
- Delta sync: Only documents updated since last cursor.
- Full sync (initial): All metadata + thumbnails. For 1000 docs ≈ 30-40MB.

---

## 8. Component Specs

### 8.1 Rust Backend (Axum)

#### Crate Structure

```
shrank-server/
├── Cargo.toml
├── src/
│   ├── main.rs              # Startup, config, server init
│   ├── config.rs            # TOML-based configuration
│   ├── db/
│   │   ├── mod.rs
│   │   ├── migrations.rs    # Schema creation/migration
│   │   ├── documents.rs     # Document CRUD
│   │   ├── entities.rs      # Entity CRUD
│   │   ├── graph.rs         # Edge management + traversal queries
│   │   ├── search.rs        # FTS5 + sqlite-vec + structured + fusion
│   │   └── sync.rs          # Sync cursor management
│   ├── api/
│   │   ├── mod.rs
│   │   ├── documents.rs     # POST/GET/PUT/DELETE documents
│   │   ├── search.rs        # GET /search
│   │   ├── sync.rs          # GET /sync, sync protocol endpoints
│   │   ├── images.rs        # GET /originals/:id, /thumbnails/:id
│   │   └── graph.rs         # GET /graph/:id (related documents)
│   ├── inference/
│   │   ├── mod.rs
│   │   ├── client.rs        # HTTP client to mlx-vlm sidecar
│   │   ├── extraction.rs    # Parse + validate LLM response
│   │   └── relationships.rs # Post-extraction relationship inference
│   ├── images/
│   │   ├── mod.rs
│   │   ├── storage.rs       # File storage, path management
│   │   └── processing.rs    # Thumbnail generation via libvips
│   ├── auth.rs              # API key middleware
│   └── errors.rs            # Error types + responses
```

#### Key Dependencies

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
rusqlite = { version = "0.32", features = ["bundled", "fts5"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ulid = "1"
reqwest = { version = "0.12", features = ["json"] }
tower-http = { version = "0.6", features = ["cors", "fs", "trace"] }
tracing = "0.1"
tracing-subscriber = "0.3"
libvips = "1"  # or image crate as fallback
toml = "0.8"
base64 = "0.22"
```

#### API Endpoints

```
# Documents
POST   /api/documents              # Upload new document (multipart)
GET    /api/documents              # List documents (paginated, filtered)
GET    /api/documents/:id          # Get single document with full metadata
PUT    /api/documents/:id          # Update metadata (manual corrections)
DELETE /api/documents/:id          # Delete document and all associated data
POST   /api/documents/:id/reprocess  # Re-run LLM extraction

# Search
GET    /api/search?q=...&limit=&offset=  # Unified search (all 4 layers)

# Graph
GET    /api/documents/:id/related?depth=2  # Graph traversal from document
GET    /api/entities                       # List all entities
GET    /api/entities/:id/documents         # Documents linked to entity

# Sync (phone ↔ server)
GET    /api/sync?since=<cursor>&limit=50   # Delta sync for phone
POST   /api/sync/register                  # Register phone device

# Images
GET    /api/images/original/:id    # Full resolution original
GET    /api/images/thumbnail/:id   # WebP thumbnail

# System
GET    /api/health                 # Server + inference sidecar status
GET    /api/stats                  # Document count, tag cloud, storage used
```

### 8.2 Inference Sidecar (Python / mlx-vlm)

```
shrank-inference/
├── pyproject.toml
├── server.py              # FastAPI app (see Section 5.3)
├── prompts/
│   └── extraction.py      # Prompt templates (see Section 11)
├── parsers/
│   └── response.py        # JSON extraction + validation from LLM output
└── run.sh                 # Startup script: uvicorn server:app --host 127.0.0.1 --port 3421
```

#### Dependencies

```toml
[project]
dependencies = [
    "fastapi>=0.115",
    "uvicorn>=0.32",
    "mlx-vlm>=0.4.3",
    "pydantic>=2.0",
]
```

The sidecar listens only on `127.0.0.1:3421` — never exposed to the network.

### 8.3 iOS App (Swift / SwiftUI)

#### Feature Set

1. **Camera capture**: Full-screen camera with document edge detection (VisionKit `VNDocumentCameraViewController`). Supports multi-page scanning in one session.
2. **Local gallery**: Grid view of all captured documents with thumbnails. Synced documents show extracted metadata; pending ones show "Processing..." badge.
3. **Offline search**: Search bar with FTS5 against local SQLite. Semantic search available when embeddings have synced.
4. **Document detail**: Full metadata view, original image (if synced back), related documents via local graph edges.
5. **Background sync**: `BGTaskScheduler` with `BGProcessingTaskRequest`. Checks server reachability, uploads pending, downloads new metadata + thumbnails.
6. **Settings**: Server address (Tailscale hostname), API key, storage usage, manual sync trigger, export database.

#### Project Structure

```
Shrank-iOS/
├── Shrank.xcodeproj
├── Shrank/
│   ├── App/
│   │   ├── ShrankApp.swift      # App entry, BGTaskScheduler registration
│   │   └── ContentView.swift          # Tab-based navigation
│   ├── Models/
│   │   ├── Document.swift             # Core data model (mirrors server schema)
│   │   ├── Entity.swift
│   │   └── SyncState.swift
│   ├── Database/
│   │   ├── DatabaseManager.swift      # SQLite wrapper (GRDB.swift or raw SQLite3)
│   │   ├── Migrations.swift
│   │   └── SearchEngine.swift         # Local FTS5 + sqlite-vec queries
│   ├── Networking/
│   │   ├── APIClient.swift            # HTTP client to Axum backend
│   │   ├── SyncEngine.swift           # Upload queue + delta sync logic
│   │   └── ReachabilityMonitor.swift  # Tailscale peer detection
│   ├── Camera/
│   │   ├── CameraView.swift           # VisionKit document scanner
│   │   └── ImageProcessor.swift       # EXIF extraction, local thumbnail gen
│   ├── Views/
│   │   ├── DocumentList/
│   │   │   ├── DocumentListView.swift
│   │   │   └── DocumentRow.swift
│   │   ├── DocumentDetail/
│   │   │   ├── DocumentDetailView.swift
│   │   │   └── RelatedDocumentsView.swift
│   │   ├── Search/
│   │   │   ├── SearchView.swift
│   │   │   └── FacetedFilterView.swift
│   │   └── Settings/
│   │       └── SettingsView.swift
│   └── Storage/
│       ├── ImageStore.swift           # Local image file management
│       └── ThumbnailCache.swift
├── ShrankTests/
└── ShrankUITests/
```

#### Key iOS Libraries

- **GRDB.swift**: SQLite wrapper with FTS5 support. More ergonomic than raw C API.
- **VisionKit**: Apple's document scanner with auto edge detection and perspective correction.
- **Network.framework**: NWPathMonitor for connectivity, NWBrowser for Tailscale peer.
- **BackgroundTasks**: BGTaskScheduler for periodic sync.
- No third-party dependencies for networking (URLSession is sufficient).

### 8.4 Web UI (React / Vite)

#### Feature Set

1. **Dashboard**: Recent documents, processing queue status, storage stats, tag cloud.
2. **Document browser**: Infinite scroll grid with thumbnails. Click to open detail view.
3. **Search**: Search bar with instant results. Faceted filters (sender, date range, type, tags) as sidebar.
4. **Document detail**: Full-size image viewer (zoom/pan), all extracted metadata, editable fields (sender, tags, type), related documents graph visualization.
5. **Graph explorer**: Interactive visualization of document relationships. Click entity to see all connected documents.
6. **Bulk actions**: Re-process selected, delete selected, export selected as ZIP.
7. **Manual upload**: Drag-and-drop images for documents not captured via phone.

#### Project Structure

```
shrank-web/
├── package.json
├── vite.config.ts
├── index.html
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── api/
│   │   ├── client.ts          # Typed API client (fetch-based, no axios)
│   │   ├── types.ts           # TypeScript interfaces matching server schemas
│   │   └── hooks.ts           # React Query hooks for data fetching
│   ├── components/
│   │   ├── Layout/
│   │   │   ├── Sidebar.tsx
│   │   │   └── Header.tsx
│   │   ├── Documents/
│   │   │   ├── DocumentGrid.tsx
│   │   │   ├── DocumentCard.tsx
│   │   │   └── DocumentDetail.tsx
│   │   ├── Search/
│   │   │   ├── SearchBar.tsx
│   │   │   ├── SearchResults.tsx
│   │   │   └── FacetFilters.tsx
│   │   ├── Graph/
│   │   │   └── GraphExplorer.tsx   # d3-force or vis.js for graph viz
│   │   ├── Upload/
│   │   │   └── DropZone.tsx
│   │   └── common/
│   │       ├── ImageViewer.tsx
│   │       ├── TagBadge.tsx
│   │       └── StatusBadge.tsx
│   ├── pages/
│   │   ├── Dashboard.tsx
│   │   ├── Browse.tsx
│   │   ├── Search.tsx
│   │   ├── DocumentPage.tsx
│   │   └── GraphPage.tsx
│   └── styles/
│       └── tailwind.css
├── tailwind.config.js
└── tsconfig.json
```

#### Key Web Libraries

```json
{
  "dependencies": {
    "react": "^19",
    "react-dom": "^19",
    "react-router-dom": "^7",
    "@tanstack/react-query": "^5",
    "d3": "^7",                    // graph visualization
    "react-zoom-pan-pinch": "^3",  // image viewer
    "tailwindcss": "^4",
    "lucide-react": "^0.400"       // icons
  }
}
```

#### Production Serving

In development: Vite dev server with proxy to Axum backend.
In production: `vite build` → static files served directly by Axum via `tower-http::services::ServeDir`.

---

## 9. Project Structure

```
shrank/
├── README.md
├── LICENSE                        # Apache-2.0
├── CONTRIBUTING.md
├── docker-compose.yml             # For non-Mac Linux/NVIDIA users
├── Makefile                       # dev, build, test, run shortcuts
├── .github/
│   └── workflows/
│       ├── ci.yml                 # Rust tests + clippy + web build
│       └── release.yml            # Binary releases
│
├── server/                        # Rust backend (see 8.1)
│   ├── Cargo.toml
│   └── src/
│
├── inference/                     # Python mlx-vlm sidecar (see 8.2)
│   ├── pyproject.toml
│   ├── server.py
│   └── prompts/
│
├── ios/                           # Swift iOS app (see 8.3)
│   ├── Shrank.xcodeproj
│   └── Shrank/
│
├── web/                           # React web UI (see 8.4)
│   ├── package.json
│   └── src/
│
├── docs/                          # Documentation site (deployed to shrank.io)
│   ├── setup.md                   # Getting started guide
│   ├── architecture.md            # This document, expanded
│   ├── api.md                     # Full API reference
│   ├── llm-config.md             # How to swap LLM backends
│   └── screenshots/
│
├── scripts/
│   ├── setup-mac.sh              # Install mlx-vlm, download model, configure Tailscale
│   ├── setup-linux-nvidia.sh     # Alternative: Ollama + NVIDIA GPU
│   └── dev.sh                    # Start all services for development
│
└── config/
    └── shrank.example.toml  # Example configuration
```

---

## 10. Implementation Phases

### Phase 1: Foundation (Week 1-2)

**Goal**: Server receives an image, runs it through Gemma 4, stores structured result, basic web UI shows it.

- [ ] Initialize monorepo with Rust workspace, Python sidecar, React scaffold
- [ ] SQLite schema: documents table, FTS5 index, migrations
- [ ] Axum server: `POST /api/documents` (multipart upload), `GET /api/documents`, `GET /api/documents/:id`
- [ ] Image storage: save original, generate thumbnail via libvips
- [ ] mlx-vlm sidecar: `/extract` endpoint with Gemma 4 26B MoE
- [ ] Extraction prompt v1 (see Section 11)
- [ ] Async processing: upload → queue → process → update document
- [ ] Web UI: document list + detail view with metadata display
- [ ] Manual upload via web UI drag-and-drop
- [ ] `make dev` command that starts all services

**Milestone**: Upload a photo of a German insurance letter via web UI, see extracted metadata appear.

### Phase 2: Search (Week 3-4)

**Goal**: All four search layers working, web UI has search with facets.

- [ ] FTS5 search endpoint with BM25 ranking and snippet highlighting
- [ ] Structured filter parsing (field:value syntax)
- [ ] sqlite-vec integration: generate embeddings at ingest, vector search endpoint
- [ ] Embedding model setup (nomic-embed-text via Ollama or dedicated)
- [ ] Reciprocal Rank Fusion for hybrid search results
- [ ] Query router: detect intent, dispatch to appropriate layers
- [ ] Web UI: search bar, instant results, faceted filters (sender, type, tags, date range)
- [ ] Web UI: tag cloud on dashboard

**Milestone**: Search "Krankenkasse 2026" returns relevant documents ranked by hybrid score.

### Phase 3: Knowledge Graph (Week 5-6)

**Goal**: Documents are connected via entities and references. Web UI shows graph.

- [ ] Entity extraction and UPSERT at ingest time
- [ ] Document-entity linking
- [ ] Reference matching: find existing docs by shared reference IDs
- [ ] Document-to-document edges (follows_up, references, responds_to)
- [ ] "Bezug auf" / date-based heuristic matching for German correspondence
- [ ] Recursive CTE graph traversal endpoint
- [ ] Web UI: "Related documents" section on document detail
- [ ] Web UI: Graph explorer page (d3-force visualization)
- [ ] Entity detail page: all documents linked to an entity

**Milestone**: View a Krankenkasse letter, see related letters/invoices auto-linked via policy number.

### Phase 4: iOS App (Week 7-9)

**Goal**: Capture documents on iPhone, sync to server, search offline.

- [ ] Xcode project setup with SwiftUI
- [ ] VisionKit document scanner integration (multi-page)
- [ ] Local SQLite database with GRDB.swift
- [ ] Local image storage + thumbnail generation
- [ ] Upload queue with retry logic
- [ ] Tailscale reachability detection
- [ ] Background sync via BGTaskScheduler
- [ ] Delta sync: pull new metadata + thumbnails from server
- [ ] Local FTS5 search
- [ ] Document list, detail, and search views
- [ ] Settings: server address, API key, manual sync

**Milestone**: Photograph a letter on iPhone, it appears in web UI within 30 seconds when Mac is online.

### Phase 5: Polish & Open-Source (Week 10-12)

**Goal**: Production-ready, documented, public.

- [ ] sqlite-vec on iOS for local semantic search
- [ ] Local graph edges on iOS for offline related-document browsing
- [ ] Reprocessing: re-run extraction on existing documents (prompt improvements)
- [ ] Bulk operations in web UI
- [ ] Export: ZIP of selected documents + metadata as JSON
- [ ] Configuration documentation + example TOML
- [ ] Setup scripts for Mac (mlx-vlm) and Linux/NVIDIA (Ollama)
- [ ] docker-compose.yml for Linux users
- [ ] README with demo GIF
- [ ] CONTRIBUTING.md
- [ ] CI: GitHub Actions for Rust tests, clippy, web build
- [ ] First GitHub release with pre-built binaries

**Milestone**: Public repo, HN post with demo GIF, first external contributor.

---

## 11. Gemma 4 Prompt Engineering

### 11.1 Core Extraction Prompt

This is the most critical piece of the system. The prompt must:
- Work for any language without configuration
- Extract structured JSON reliably
- Produce consistent tags by referencing existing vocabulary
- Handle poor image quality, partial documents, handwriting
- Return a confidence score so the UI can flag uncertain extractions

```python
EXTRACTION_SYSTEM_PROMPT = """You are a document analysis engine for a personal paper mail archive.
You receive photographs of physical mail documents (letters, invoices, statements, contracts, notices).
Your job is to extract all structured information from the document image.

CRITICAL RULES:
1. Detect the document language automatically. Do NOT assume any language.
2. Extract ALL visible text from the document (OCR).
3. Identify the sender/organization, dates, amounts, reference numbers.
4. Suggest descriptive tags for categorization. Tags should be in English for consistency,
   but include important domain-specific terms in the original language as additional tags.
5. Generate a one-sentence English summary regardless of document language.
6. Report your confidence (0.0-1.0) in the extraction quality.
7. If the image is blurry, partially cut off, or illegible, still extract what you can
   and set confidence accordingly.
8. Output ONLY valid JSON. No markdown, no explanation, no preamble."""

def build_extraction_prompt(existing_tags: list[str], existing_senders: list[str]) -> str:
    tag_hint = ""
    if existing_tags:
        tag_hint = f"""
EXISTING TAGS IN ARCHIVE (reuse these when applicable, add new ones if needed):
{', '.join(existing_tags[:100])}
"""

    sender_hint = ""
    if existing_senders:
        sender_hint = f"""
KNOWN SENDERS (use exact spelling if this document is from one of these):
{', '.join(existing_senders[:50])}
"""

    return f"""Analyze this document image and extract all information.
{tag_hint}
{sender_hint}
Respond with a single JSON object in this exact schema:

{{
  "language": "<ISO 639-1 code>",
  "sender": "<full sender name as printed>",
  "sender_normalized": "<cleaned/canonical sender name>",
  "document_date": "<YYYY-MM-DD or null if not visible>",
  "document_type": "<one of: invoice, letter, policy, statement, contract, receipt, notification, certificate, form, reminder, other>",
  "subject": "<one-line summary in the document's language>",
  "summary": "<one-sentence English summary of the document's purpose and key information>",
  "extracted_text": "<full OCR text, preserving line breaks>",
  "amounts": [
    {{"value": <number>, "currency": "<ISO 4217>", "label": "<what this amount represents>"}}
  ],
  "dates": [
    {{"date": "<YYYY-MM-DD>", "label": "<what this date represents>"}}
  ],
  "reference_ids": [
    {{"type": "<policy|invoice|account|iban|tax_id|reference|customer_id|contract>", "value": "<the ID>"}}
  ],
  "tags": ["<tag1>", "<tag2>", "..."],
  "entities": [
    {{"type": "<organization|person|policy|vehicle|property|account>", "value": "<entity value>", "role": "<sender|recipient|referenced|beneficiary>"}}
  ],
  "related_references": ["<any explicit references to other documents, e.g. 'Bezug auf Ihr Schreiben vom 15.03.2026'>"],
  "confidence": <0.0 to 1.0>,
  "extraction_notes": "<any issues: blurry text, cut-off sections, handwritten parts>"
}}

IMPORTANT: Output ONLY the JSON object. No other text."""
```

### 11.2 Prompt Optimization Notes

- **Temperature 0.1**: Low temperature for consistent, deterministic extraction. We want the same document to produce the same output every time.
- **Token budget for images**: Use high token budget (1120) for OCR-heavy documents. This is configurable in mlx-vlm and significantly affects OCR quality.
- **Existing tags injection**: By passing the current tag vocabulary, we guide Gemma to reuse existing tags rather than inventing synonyms. This creates a naturally convergent taxonomy.
- **Existing senders injection**: Prevents "AOK Bayern" vs "AOK Bayern - Die Gesundheitskasse" drift.
- **English summary + original-language subject**: The summary is always English for searchability; the subject preserves the document's own language for authenticity.
- **Confidence score**: Lets the UI highlight documents that need manual review. Threshold of 0.7 for auto-acceptance.

### 11.3 Handling Edge Cases

| Case | Handling |
|------|----------|
| Blurry/low-quality photo | Extract what's possible, set confidence < 0.5, populate extraction_notes |
| Multi-page document | iOS scanner captures multiple pages; send all pages as a single request |
| Handwritten notes | Gemma 4 has handwriting recognition; confidence may be lower |
| Non-Latin scripts | Gemma 4 supports 140+ languages; tag/summary still in English |
| Envelopes (no content) | document_type: "envelope", extract sender/recipient addresses |
| Photographs of screens | Works but note in extraction_notes; may contain reflection artifacts |
| Very old/yellowed documents | Image preprocessing (contrast enhancement) before sending to LLM |

---

## 12. Configuration & Deployment

### 12.1 Configuration File

```toml
# ~/.config/shrank/config.toml

[server]
host = "0.0.0.0"
port = 3420
data_dir = "~/.local/share/shrank"  # SQLite DB + images stored here

[auth]
api_key = "sk_your_secret_key_here"  # Generate with: openssl rand -hex 32

[inference]
# mlx-vlm sidecar (default for Apple Silicon)
backend = "mlx-vlm"
endpoint = "http://127.0.0.1:3421"
model = "mlx-community/gemma-4-26b-a4b-it-4bit"

# Alternative: Ollama
# backend = "ollama"
# endpoint = "http://127.0.0.1:11434"
# model = "gemma4:26b"

# Alternative: Any OpenAI-compatible API
# backend = "openai-compatible"
# endpoint = "http://127.0.0.1:8000/v1"
# model = "gemma-4-26b"

[embeddings]
# Embedding model for semantic search
backend = "ollama"
model = "nomic-embed-text"
endpoint = "http://127.0.0.1:11434"
dimensions = 768

[images]
thumbnail_width = 400
thumbnail_format = "webp"
thumbnail_quality = 80

[processing]
# Number of concurrent document processing tasks
workers = 1  # Keep at 1 for single-GPU setups
# Auto-reprocess documents when prompt version changes
auto_reprocess = false
# Confidence threshold below which documents are flagged for review
confidence_threshold = 0.7
```

### 12.2 Mac Setup (Primary Target)

```bash
# 1. Install prerequisites
brew install python@3.12 rust node

# 2. Clone repo
git clone https://github.com/shrank-io/shrank.git
cd shrank

# 3. Setup inference sidecar
cd inference
python3.12 -m venv .venv
source .venv/bin/activate
pip install -e .
# Download model (first run only, ~14GB for 26B MoE 4-bit)
python -c "from mlx_vlm import load; load('mlx-community/gemma-4-26b-a4b-it-4bit')"

# 4. Install embedding model
brew install ollama
ollama pull nomic-embed-text

# 5. Build Rust backend
cd ../server
cargo build --release

# 6. Build web UI
cd ../web
npm install
npm run build

# 7. Generate config
mkdir -p ~/.config/shrank
cp ../config/shrank.example.toml ~/.config/shrank/config.toml
# Edit config: set api_key

# 8. Setup Tailscale (if not already)
brew install tailscale
# Follow Tailscale setup, note your Mac's Tailscale hostname

# 9. Run everything
cd ..
make run
# Or individually:
# Terminal 1: cd inference && source .venv/bin/activate && uvicorn server:app --host 127.0.0.1 --port 3421
# Terminal 2: cd server && cargo run --release
# Web UI: http://localhost:3420
```

### 12.3 Linux + NVIDIA GPU Setup

```bash
# Alternative for Linux users with NVIDIA GPUs
# Uses Ollama instead of mlx-vlm

# 1. Install Ollama
curl -fsSL https://ollama.com/install.sh | sh
ollama pull gemma4:26b
ollama pull nomic-embed-text

# 2. Update config.toml
[inference]
backend = "ollama"
endpoint = "http://127.0.0.1:11434"
model = "gemma4:26b"

# 3. Rest of setup is identical (Rust + Node)
```

### 12.4 Docker Compose (Linux)

```yaml
# docker-compose.yml — for Linux/NVIDIA users who want one-command setup
version: '3.8'
services:
  ollama:
    image: ollama/ollama:latest
    runtime: nvidia
    environment:
      - NVIDIA_VISIBLE_DEVICES=all
    volumes:
      - ollama_data:/root/.ollama
    ports:
      - "11434:11434"

  server:
    build: ./server
    ports:
      - "3420:3420"
    volumes:
      - shrank_data:/data
    environment:
      - SHRANK_DATA_DIR=/data
      - SHRANK_INFERENCE_ENDPOINT=http://ollama:11434
      - SHRANK_INFERENCE_BACKEND=ollama
    depends_on:
      - ollama

volumes:
  ollama_data:
  shrank_data:
```

### 12.5 LaunchAgent (macOS Auto-Start)

```xml
<!-- ~/Library/LaunchAgents/com.shrank.server.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.shrank.server</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/shrank-server</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/shrank.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/shrank.err</string>
</dict>
</plist>
```

---

## 13. Security Model

### 13.1 Threat Model

This is a personal, local-network tool. The threat model is:
- **Primary threat**: Accidental exposure of the API to the public internet.
- **Secondary threat**: Compromise of another device on the local network.
- **Non-threat**: Nation-state attackers, physical access (if they have your Mac, you have bigger problems).

### 13.2 Defense Layers

1. **Tailscale (WireGuard)**: All phone↔Mac traffic is encrypted end-to-end. The Axum server is only reachable via Tailscale IPs or localhost. Never bind to `0.0.0.0` on a public interface.

2. **API Key Authentication**: Every request from the phone includes `Authorization: Bearer sk_<key>`. The key is generated during setup and stored in iOS Keychain. This prevents other Tailscale devices on the same tailnet from accessing documents.

3. **Inference sidecar isolation**: The mlx-vlm sidecar binds to `127.0.0.1:3421` only. Not accessible from any network interface.

4. **No remote code execution**: The LLM output is parsed as JSON only. No `eval()`, no template injection, no code execution paths from LLM responses. The Rust JSON parser (serde_json) is memory-safe.

5. **File path sanitization**: All image paths are constructed by the server using ULIDs. User input never controls filesystem paths.

6. **SQLite**: Database file permissions set to `600` (owner-only). No network-accessible database server.

### 13.3 iOS Security

- API key stored in iOS Keychain (encrypted at rest).
- Local SQLite database in app sandbox (encrypted at rest by iOS data protection).
- Images stored in app's Documents directory (backed up to iCloud only if user enables).
- No analytics, no crash reporting, no third-party SDKs.

---

## 14. Future Considerations

### v2 Features (Post-Launch)

- **On-device preview extraction**: Run Gemma 4 E4B on iPhone for instant pre-classification before sync. Show predicted sender/type immediately after capture. Refine with 26B on Mac.
- **Multi-user support**: Shared family archive with per-user access. Requires auth upgrade.
- **PDF import**: Bulk import existing digital documents (bank statements, email attachments).
- **Email integration**: Monitor a local mailbox (IMAP) for digital documents, auto-ingest.
- **Smart reminders**: Detect deadlines (Widerspruchsfrist, Zahlungsziel) and surface them.
- **Export to Paperless-ngx**: Migration path for users who want to switch.
- **Android app**: Kotlin/Jetpack Compose. Same sync protocol.
- **Apple Watch**: Quick capture shortcut.
- **Fine-tuned model**: LoRA fine-tune on user's actual documents for improved extraction accuracy.
- **Backup**: Encrypted backup to external drive or NAS. Single command.

### Technical Debt Awareness

- **FTS5 German compound words**: The unicode61 tokenizer doesn't decompose compounds like "Krankenversicherungsbeitrag". A custom tokenizer or decomposition preprocessing may be needed. Evaluate ICU tokenizer or pre-splitting with a German morphology library.
- **sqlite-vec maturity**: Still young. Monitor for stability issues. Fallback plan: write embeddings to a flat file and use faiss or hnswlib.
- **mlx-vlm API stability**: The library is evolving rapidly. Pin versions carefully.
- **iOS background sync limits**: iOS aggressively limits background task frequency. The sync interval depends on user behavior (how often they open the app, battery level, network). Document this limitation.

---

## 15. Open-Source Strategy

### Repository

- **GitHub Org**: `shrank-io`
- **Main repo**: [`shrank-io/shrank`](https://github.com/shrank-io/shrank)
- **Website repo**: `shrank-io/shrank.io` (GitHub Pages → shrank.io)
- **License**: Apache-2.0
- **Domain**: [shrank.io](https://shrank.io)
- **Tagline**: "Your paper pile just shrank."

### README Structure

1. Hero: one-sentence description + demo GIF
2. Features: bullet list with emojis
3. Quick Start: 5-step setup
4. Screenshots: web UI, iOS app, graph view
5. Architecture: simplified diagram
6. Configuration: link to docs
7. Contributing: link to CONTRIBUTING.md
8. License: Apache-2.0

### Community

- GitHub Discussions for questions
- GitHub Issues for bugs
- No Discord (keep it simple early on)
- Blog post on launch at shrank.io explaining the architecture decisions
- HN submission: "Show HN: Shrank – your paper pile just shrank (local AI document archive)"

### Contribution Areas

Mark good-first-issue items:
- Additional document type detection
- Improved German compound word tokenization
- Android app
- Additional LLM backend adapters (e.g., llama.cpp direct, ONNX)
- Localization of the web UI
- Accessibility improvements
- Additional deployment guides (Synology NAS, Raspberry Pi 5, etc.)

---

## Appendix: Quick Reference

### Start Development

```bash
# Clone and setup
git clone https://github.com/shrank-io/shrank.git
cd shrank
make setup     # Install all dependencies
make dev       # Start all services in dev mode

# Individual services
make server    # Rust backend on :3420
make inference # mlx-vlm sidecar on :3421
make web       # Vite dev server on :5173 (proxies to :3420)
```

### Key URLs (Development)

| Service | URL |
|---------|-----|
| Web UI | http://localhost:5173 |
| API | http://localhost:3420/api |
| API docs | http://localhost:3420/api/docs |
| Inference health | http://localhost:3421/health |
| Ollama | http://localhost:11434 |

### Database Location

```
~/.local/share/shrank/
├── shrank.db          # SQLite database (all metadata, FTS, vectors, graph)
├── originals/              # Full-resolution scans
│   └── <ULID>/scan.jpg
└── thumbnails/             # WebP thumbnails
    └── <ULID>/thumb.webp
```
