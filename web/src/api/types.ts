/** Matches the server's documents table schema */
export interface Document {
  id: string;
  created_at: string;
  updated_at: string;
  captured_at: string;
  synced_at: string | null;
  original_path: string;
  thumbnail_path: string;
  status: "pending" | "processing" | "complete" | "error";
  processing_error: string | null;
  raw_llm_response: string | null;
  language: string | null;
  sender: string | null;
  sender_normalized: string | null;
  document_date: string | null;
  document_type: string | null;
  subject: string | null;
  extracted_text: string | null;
  amounts: Amount[] | null;
  dates: DateEntry[] | null;
  reference_ids: ReferenceId[] | null;
  tags: string[] | null;
  confidence: number | null;
}

export interface Amount {
  value: number;
  currency: string;
  label: string;
}

export interface DateEntry {
  date: string;
  label: string;
}

export interface ReferenceId {
  type: string;
  value: string;
}

export interface Entity {
  id: string;
  entity_type: string;
  value: string;
  display_name: string | null;
  metadata: Record<string, unknown> | null;
  created_at: string;
}

export interface DocumentEntity {
  document_id: string;
  entity_id: string;
  role: string;
  confidence: number;
}

export interface DocumentEdge {
  source_id: string;
  target_id: string;
  relation_type: string;
  confidence: number;
  inferred_by: string;
  created_at: string;
}

/** Search API response */
export interface SearchResult {
  document: Document;
  score: number;
  match_sources: string[];
  highlights: Record<string, string>;
}

export interface Facets {
  senders: FacetEntry[];
  tags: FacetEntry[];
  types: FacetEntry[];
  years: FacetEntry[];
}

export interface FacetEntry {
  name: string;
  count: number;
}

export interface SearchResponse {
  results: SearchResult[];
  facets: Facets;
  total: number;
  query_intent: string;
}

/** Stats API response (matches actual backend) */
export interface Stats {
  total_documents: number;
  unique_senders: number;
  unique_tags: number;
  by_status: Record<string, number>;
  storage_bytes: number;
  tags: FacetEntry[];
}

/** Graph traversal response */
export interface GraphNode {
  id: string;
  label: string;
  type: "document" | "entity";
  document?: Document;
  entity?: Entity;
}

export interface GraphLink {
  source: string;
  target: string;
  relation_type: string;
  confidence: number;
}

export interface GraphData {
  nodes: GraphNode[];
  links: GraphLink[];
}

/** Paginated list response (matches actual backend) */
export interface PaginatedResponse<T> {
  documents: T[];
  total: number;
}

/** Document update payload */
export interface DocumentUpdate {
  sender?: string;
  sender_normalized?: string;
  document_date?: string;
  document_type?: string;
  subject?: string;
  tags?: string[];
}
