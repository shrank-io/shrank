"""Shrank inference sidecar — thin wrapper around vllm-mlx.

Runs on 127.0.0.1:3421 (localhost only). The Rust backend calls these endpoints.
vllm-mlx runs separately and handles model loading, GPU, and inference.
"""

import logging
import os

import httpx
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

from parsers.response import parse_llm_json
from prompts.extraction import EXTRACTION_SYSTEM_PROMPT, build_extraction_prompt

logger = logging.getLogger("shrank.inference")

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
VLLM_ENDPOINT = os.environ.get("SHRANK_VLLM_ENDPOINT", "http://127.0.0.1:8000")
MODEL = os.environ.get("SHRANK_MODEL", "mlx-community/gemma-4-26b-a4b-it-4bit")
EMBED_MODEL = os.environ.get("SHRANK_EMBED_MODEL", "mlx-community/all-MiniLM-L6-v2-4bit")

app = FastAPI(title="Shrank Inference Sidecar")


# ---------------------------------------------------------------------------
# Request / response schemas
# ---------------------------------------------------------------------------
class ExtractionRequest(BaseModel):
    image_base64: str
    existing_tags: list[str] = []
    existing_senders: list[str] = []


class EmbedRequest(BaseModel):
    text: str


class ChatMessage(BaseModel):
    role: str
    content: str


class ChatRequest(BaseModel):
    messages: list[ChatMessage]
    max_tokens: int = 2048
    temperature: float = 0.7


# ---------------------------------------------------------------------------
# POST /extract
# ---------------------------------------------------------------------------
@app.post("/extract")
async def extract(req: ExtractionRequest):
    prompt = build_extraction_prompt(req.existing_tags, req.existing_senders)

    messages = [
        {"role": "system", "content": EXTRACTION_SYSTEM_PROMPT},
        {
            "role": "user",
            "content": [
                {
                    "type": "image_url",
                    "image_url": {"url": f"data:image/jpeg;base64,{req.image_base64}"},
                },
                {"type": "text", "text": prompt},
            ],
        },
    ]

    try:
        async with httpx.AsyncClient(timeout=120.0) as client:
            resp = await client.post(
                f"{VLLM_ENDPOINT}/v1/chat/completions",
                json={
                    "model": MODEL,
                    "messages": messages,
                    "max_tokens": 4096,
                    "temperature": 0.1,
                },
            )
            resp.raise_for_status()
            data = resp.json()

        raw_text = data["choices"][0]["message"]["content"]
        return parse_llm_json(raw_text)

    except httpx.HTTPError as exc:
        logger.exception("Extraction request to vllm-mlx failed")
        raise HTTPException(status_code=502, detail=f"vllm-mlx error: {exc}")


# ---------------------------------------------------------------------------
# POST /embed
# ---------------------------------------------------------------------------
@app.post("/embed")
async def embed(req: EmbedRequest):
    if not req.text.strip():
        raise HTTPException(status_code=400, detail="Empty text")

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.post(
                f"{VLLM_ENDPOINT}/v1/embeddings",
                json={"model": EMBED_MODEL, "input": [req.text]},
            )
            resp.raise_for_status()
            data = resp.json()

        embedding = data["data"][0]["embedding"]
        return {
            "embedding": embedding,
            "model": data.get("model", EMBED_MODEL),
            "dimensions": len(embedding),
        }
    except httpx.HTTPError as exc:
        logger.exception("Embedding request to vllm-mlx failed")
        raise HTTPException(status_code=502, detail=f"Embedding backend error: {exc}")


# ---------------------------------------------------------------------------
# POST /chat
# ---------------------------------------------------------------------------
@app.post("/chat")
async def chat(req: ChatRequest):
    messages = [{"role": m.role, "content": m.content} for m in req.messages]

    try:
        async with httpx.AsyncClient(timeout=120.0) as client:
            resp = await client.post(
                f"{VLLM_ENDPOINT}/v1/chat/completions",
                json={
                    "model": MODEL,
                    "messages": messages,
                    "max_tokens": req.max_tokens,
                    "temperature": req.temperature,
                },
            )
            resp.raise_for_status()
            data = resp.json()

        return {
            "content": data["choices"][0]["message"]["content"],
            "model": data.get("model", MODEL),
            "usage": data.get("usage"),
        }
    except httpx.HTTPError as exc:
        logger.exception("Chat request to vllm-mlx failed")
        raise HTTPException(status_code=502, detail=f"vllm-mlx error: {exc}")


# ---------------------------------------------------------------------------
# GET /health
# ---------------------------------------------------------------------------
@app.get("/health")
async def health():
    try:
        async with httpx.AsyncClient(timeout=5.0) as client:
            resp = await client.get(f"{VLLM_ENDPOINT}/v1/models")
            resp.raise_for_status()
            data = resp.json()

        models = [m["id"] for m in data.get("data", [])]
        return {
            "status": "ready",
            "model": MODEL,
            "backend": "vllm-mlx",
            "available_models": models,
        }
    except Exception:
        return {
            "status": "unavailable",
            "model": MODEL,
            "backend": "vllm-mlx",
            "error": f"Cannot reach vllm-mlx at {VLLM_ENDPOINT}",
        }
