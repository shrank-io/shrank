import { useState } from "react";
import { SlidersHorizontal } from "lucide-react";
import { useDocuments } from "../api/hooks";
import DocumentGrid from "../components/Documents/DocumentGrid";

const DOC_TYPES = [
  "invoice",
  "letter",
  "policy",
  "statement",
  "contract",
  "receipt",
  "notification",
  "certificate",
];

export default function Browse() {
  const [status, setStatus] = useState<string | undefined>();
  const [docType, setDocType] = useState<string | undefined>();
  const [sort, setSort] = useState("created_at");

  const { data, hasNextPage, isFetchingNextPage, fetchNextPage, isLoading } =
    useDocuments({ status, type: docType, sort, limit: 30 });

  const documents = data?.pages.flatMap((p) => p.documents) ?? [];

  return (
    <div className="p-6">
      {/* Toolbar */}
      <div className="mb-5 flex flex-wrap items-center gap-3">
        <h1 className="mr-auto text-xl font-semibold text-ink">Documents</h1>

        <div className="flex items-center gap-2">
          <SlidersHorizontal size={14} className="text-ink-faint" />
          <select
            value={status ?? ""}
            onChange={(e) => setStatus(e.target.value || undefined)}
            className="h-8 rounded-md border border-edge bg-surface-raised px-2 text-xs text-ink outline-none"
          >
            <option value="">All statuses</option>
            <option value="pending">Pending</option>
            <option value="processing">Processing</option>
            <option value="complete">Complete</option>
            <option value="error">Error</option>
          </select>
          <select
            value={docType ?? ""}
            onChange={(e) => setDocType(e.target.value || undefined)}
            className="h-8 rounded-md border border-edge bg-surface-raised px-2 text-xs text-ink outline-none"
          >
            <option value="">All types</option>
            {DOC_TYPES.map((t) => (
              <option key={t} value={t}>
                {t.charAt(0).toUpperCase() + t.slice(1)}
              </option>
            ))}
          </select>
          <select
            value={sort}
            onChange={(e) => setSort(e.target.value)}
            className="h-8 rounded-md border border-edge bg-surface-raised px-2 text-xs text-ink outline-none"
          >
            <option value="created_at">Newest first</option>
            <option value="document_date">Document date</option>
            <option value="sender">Sender A-Z</option>
          </select>
        </div>
      </div>

      {/* Grid */}
      {isLoading ? (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-4">
          {Array.from({ length: 12 }).map((_, i) => (
            <div key={i} className="skeleton aspect-[3/5] rounded-xl" />
          ))}
        </div>
      ) : (
        <DocumentGrid
          documents={documents}
          hasNextPage={!!hasNextPage}
          isFetchingNextPage={isFetchingNextPage}
          fetchNextPage={fetchNextPage}
        />
      )}
    </div>
  );
}
