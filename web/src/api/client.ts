import type {
  Document,
  DocumentUpdate,
  GraphData,
  PaginatedResponse,
  SearchResponse,
  Stats,
} from "./types";

const API_BASE = "/api";

function getHeaders(): HeadersInit {
  const headers: HeadersInit = {};
  const key = localStorage.getItem("shrank_api_key");
  if (key) {
    headers["Authorization"] = `Bearer ${key}`;
  }
  return headers;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers: { ...getHeaders(), ...init?.headers },
  });
  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(`${res.status} ${res.statusText}: ${body}`);
  }
  return res.json();
}

// --- Documents ---

export function listDocuments(params: {
  limit?: number;
  offset?: number;
  status?: string;
  sender?: string;
  type?: string;
  tag?: string;
  sort?: string;
}): Promise<PaginatedResponse<Document>> {
  const q = new URLSearchParams();
  for (const [k, v] of Object.entries(params)) {
    if (v != null) q.set(k, String(v));
  }
  return request(`/documents?${q}`);
}

export function getDocument(id: string): Promise<Document> {
  return request(`/documents/${id}`);
}

export function updateDocument(
  id: string,
  data: DocumentUpdate,
): Promise<Document> {
  return request(`/documents/${id}`, {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
}

export function deleteDocument(id: string): Promise<void> {
  return request(`/documents/${id}`, { method: "DELETE" });
}

export function reprocessDocument(id: string): Promise<Document> {
  return request(`/documents/${id}/reprocess`, { method: "POST" });
}

export async function uploadDocument(
  file: File,
  onProgress?: (pct: number) => void,
): Promise<Document> {
  const form = new FormData();
  form.append("image", file);
  form.append("captured_at", new Date().toISOString());

  // Use XMLHttpRequest for progress tracking
  return new Promise((resolve, reject) => {
    const xhr = new XMLHttpRequest();
    xhr.open("POST", `${API_BASE}/documents`);

    const key = localStorage.getItem("shrank_api_key");
    if (key) xhr.setRequestHeader("Authorization", `Bearer ${key}`);

    xhr.upload.addEventListener("progress", (e) => {
      if (e.lengthComputable && onProgress) {
        onProgress(Math.round((e.loaded / e.total) * 100));
      }
    });
    xhr.addEventListener("load", () => {
      if (xhr.status >= 200 && xhr.status < 300) {
        resolve(JSON.parse(xhr.responseText));
      } else {
        reject(new Error(`${xhr.status}: ${xhr.responseText}`));
      }
    });
    xhr.addEventListener("error", () => reject(new Error("Upload failed")));
    xhr.send(form);
  });
}

// --- Search ---

export function search(params: {
  q: string;
  limit?: number;
  offset?: number;
}): Promise<SearchResponse> {
  const q = new URLSearchParams();
  for (const [k, v] of Object.entries(params)) {
    if (v != null) q.set(k, String(v));
  }
  return request(`/search?${q}`);
}

// --- Graph ---

export function getRelatedDocuments(
  id: string,
  depth?: number,
): Promise<GraphData> {
  const q = depth != null ? `?depth=${depth}` : "";
  return request(`/documents/${id}/related${q}`);
}

// --- Stats ---

export function getStats(): Promise<Stats> {
  return request("/stats");
}

// --- Images ---

export function originalUrl(id: string): string {
  return `${API_BASE}/images/original/${id}`;
}

export function thumbnailUrl(id: string): string {
  return `${API_BASE}/images/thumbnail/${id}`;
}
