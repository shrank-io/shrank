import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { Share2 } from "lucide-react";
import { useRelatedDocuments, useDocuments } from "../api/hooks";
import GraphExplorer from "../components/Graph/GraphExplorer";

export default function GraphPage() {
  const [searchParams] = useSearchParams();
  const initialDocId = searchParams.get("doc") ?? "";
  const [docId, setDocId] = useState(initialDocId);
  const [depth, setDepth] = useState(2);

  const { data: graph, isLoading } = useRelatedDocuments(
    docId || undefined,
    depth,
  );

  // For the document picker, show recent completed documents
  const { data: recentDocs } = useDocuments({
    limit: 20,
    status: "complete",
    sort: "created_at",
  });

  const documents = recentDocs?.pages.flatMap((p) => p.documents) ?? [];

  return (
    <div className="flex h-full flex-col p-6">
      {/* Controls */}
      <div className="mb-4 flex flex-wrap items-center gap-3">
        <Share2 size={18} className="text-ink-faint" />
        <h1 className="text-xl font-semibold text-ink">Graph Explorer</h1>

        <div className="ml-auto flex items-center gap-2">
          <select
            value={docId}
            onChange={(e) => setDocId(e.target.value)}
            className="h-8 max-w-xs rounded-md border border-edge bg-surface-raised px-2 text-xs text-ink outline-none"
          >
            <option value="">Select a document...</option>
            {documents.map((doc) => (
              <option key={doc.id} value={doc.id}>
                {doc.sender
                  ? `${doc.sender} — ${doc.subject ?? doc.id.slice(0, 8)}`
                  : doc.id.slice(0, 12)}
              </option>
            ))}
          </select>
          <select
            value={depth}
            onChange={(e) => setDepth(Number(e.target.value))}
            className="h-8 rounded-md border border-edge bg-surface-raised px-2 text-xs text-ink outline-none"
          >
            <option value={1}>Depth 1</option>
            <option value={2}>Depth 2</option>
            <option value={3}>Depth 3</option>
          </select>
        </div>
      </div>

      {/* Graph */}
      <div className="flex-1 rounded-xl border border-edge bg-surface">
        {!docId ? (
          <div className="flex h-full items-center justify-center text-ink-faint">
            <div className="text-center">
              <Share2
                size={48}
                strokeWidth={1}
                className="mx-auto mb-3 text-ink-faint/50"
              />
              <p>Select a document to explore its relationships</p>
              <p className="mt-1 text-xs">
                Documents, entities, and references form a knowledge graph
              </p>
            </div>
          </div>
        ) : isLoading ? (
          <div className="flex h-full items-center justify-center text-ink-faint">
            <p>Loading graph...</p>
          </div>
        ) : graph && graph.nodes.length > 0 ? (
          <GraphExplorer data={graph} focusId={docId} />
        ) : (
          <div className="flex h-full items-center justify-center text-ink-faint">
            <p>No relationships found for this document</p>
          </div>
        )}
      </div>
    </div>
  );
}
