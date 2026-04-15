import { useState } from "react";
import {
  ArrowLeft,
  Calendar,
  Building2,
  FileType,
  Hash,
  RefreshCw,
  Trash2,
  Save,
  X,
} from "lucide-react";
import { Link, useNavigate } from "react-router-dom";
import { originalUrl } from "../../api/client";
import {
  useDocument,
  useUpdateDocument,
  useDeleteDocument,
  useReprocessDocument,
  useRelatedDocuments,
} from "../../api/hooks";
import type { DocumentUpdate } from "../../api/types";
import ImageViewer from "../common/ImageViewer";
import StatusBadge from "../common/StatusBadge";
import TagBadge from "../common/TagBadge";

function formatDate(iso: string | null): string {
  if (!iso) return "Unknown";
  return new Date(iso).toLocaleDateString("de-DE", {
    day: "2-digit",
    month: "long",
    year: "numeric",
  });
}

export default function DocumentDetailView({ id }: { id: string }) {
  const navigate = useNavigate();
  const { data: doc, isLoading } = useDocument(id);
  const { data: graph } = useRelatedDocuments(id, 1);
  const updateMut = useUpdateDocument();
  const deleteMut = useDeleteDocument();
  const reprocessMut = useReprocessDocument();

  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState<DocumentUpdate>({});

  if (isLoading) {
    return (
      <div className="flex gap-6 p-6">
        <div className="skeleton h-[70vh] flex-1 rounded-xl" />
        <div className="w-96 space-y-4">
          <div className="skeleton h-8 w-3/4 rounded" />
          <div className="skeleton h-4 w-1/2 rounded" />
          <div className="skeleton h-32 w-full rounded" />
        </div>
      </div>
    );
  }

  if (!doc) {
    return (
      <div className="flex flex-col items-center justify-center py-24 text-ink-faint">
        <p>Document not found</p>
        <Link to="/browse" className="mt-2 text-accent hover:underline">
          Back to documents
        </Link>
      </div>
    );
  }

  function startEditing() {
    if (!doc) return;
    setDraft({
      sender: doc.sender ?? "",
      document_date: doc.document_date ?? "",
      document_type: doc.document_type ?? "",
      subject: doc.subject ?? "",
      tags: doc.tags ?? [],
    });
    setEditing(true);
  }

  function saveEdits() {
    updateMut.mutate(
      { id, data: draft },
      { onSuccess: () => setEditing(false) },
    );
  }

  function handleDelete() {
    if (confirm("Delete this document permanently?")) {
      deleteMut.mutate(id, { onSuccess: () => navigate("/browse") });
    }
  }

  const relatedDocs = graph?.nodes?.filter(
    (n) => n.type === "document" && n.id !== id,
  );

  return (
    <div className="flex h-full flex-col lg:flex-row">
      {/* Image panel */}
      <div className="flex-1 p-4">
        <div className="mb-3 flex items-center gap-2">
          <Link
            to="/browse"
            className="rounded-md p-1.5 text-ink-muted transition-colors hover:bg-surface-raised hover:text-ink"
          >
            <ArrowLeft size={18} />
          </Link>
          <h1 className="truncate text-lg font-semibold text-ink">
            {doc.subject ?? "Untitled document"}
          </h1>
        </div>
        <div className="h-[calc(100%-48px)]">
          <ImageViewer
            src={originalUrl(doc.id)}
            alt={doc.subject ?? "Document"}
          />
        </div>
      </div>

      {/* Metadata panel */}
      <div className="w-full border-l border-edge lg:w-[400px]">
        <div className="flex flex-col gap-5 overflow-y-auto p-5">
          {/* Actions */}
          <div className="flex items-center gap-2">
            <StatusBadge status={doc.status} />
            <div className="flex-1" />
            {!editing ? (
              <button
                onClick={startEditing}
                className="rounded-md bg-surface-raised px-3 py-1.5 text-xs font-medium text-ink-muted transition-colors hover:bg-surface-overlay hover:text-ink"
              >
                Edit
              </button>
            ) : (
              <>
                <button
                  onClick={saveEdits}
                  disabled={updateMut.isPending}
                  className="flex items-center gap-1 rounded-md bg-accent/15 px-3 py-1.5 text-xs font-medium text-accent transition-colors hover:bg-accent/25"
                >
                  <Save size={12} />
                  Save
                </button>
                <button
                  onClick={() => setEditing(false)}
                  className="rounded-md p-1.5 text-ink-faint hover:text-ink"
                >
                  <X size={14} />
                </button>
              </>
            )}
            <button
              onClick={() => reprocessMut.mutate(id)}
              disabled={reprocessMut.isPending}
              className="rounded-md p-1.5 text-ink-faint transition-colors hover:bg-surface-raised hover:text-ink"
              title="Reprocess"
            >
              <RefreshCw
                size={14}
                className={reprocessMut.isPending ? "animate-spin" : ""}
              />
            </button>
            <button
              onClick={handleDelete}
              disabled={deleteMut.isPending}
              className="rounded-md p-1.5 text-ink-faint transition-colors hover:bg-danger/15 hover:text-danger"
              title="Delete"
            >
              <Trash2 size={14} />
            </button>
          </div>

          {/* Fields */}
          <div className="space-y-4">
            <Field
              icon={Building2}
              label="Sender"
              value={editing ? draft.sender ?? "" : doc.sender}
              editing={editing}
              onChange={(v) => setDraft({ ...draft, sender: v })}
            />
            <Field
              icon={Calendar}
              label="Date"
              value={
                editing
                  ? draft.document_date ?? ""
                  : formatDate(doc.document_date)
              }
              editing={editing}
              onChange={(v) => setDraft({ ...draft, document_date: v })}
            />
            <Field
              icon={FileType}
              label="Type"
              value={editing ? draft.document_type ?? "" : doc.document_type}
              editing={editing}
              onChange={(v) => setDraft({ ...draft, document_type: v })}
            />
          </div>

          {/* Tags */}
          <div>
            <div className="mb-2 flex items-center gap-1.5 text-xs font-medium text-ink-faint">
              <Hash size={12} />
              Tags
            </div>
            {editing ? (
              <input
                value={(draft.tags ?? []).join(", ")}
                onChange={(e) =>
                  setDraft({
                    ...draft,
                    tags: e.target.value
                      .split(",")
                      .map((t) => t.trim())
                      .filter(Boolean),
                  })
                }
                className="w-full rounded-md border border-edge bg-surface-raised px-2 py-1.5 text-sm text-ink outline-none focus:border-accent/50"
                placeholder="tag1, tag2, ..."
              />
            ) : (
              <div className="flex flex-wrap gap-1.5">
                {doc.tags?.map((t) => <TagBadge key={t} tag={t} />) ?? (
                  <span className="text-xs text-ink-faint">No tags</span>
                )}
              </div>
            )}
          </div>

          {/* Amounts */}
          {doc.amounts && doc.amounts.length > 0 && (
            <div>
              <h3 className="mb-2 text-xs font-medium text-ink-faint">
                Amounts
              </h3>
              <div className="space-y-1">
                {doc.amounts.map((a, i) => (
                  <div
                    key={i}
                    className="flex items-baseline justify-between text-sm"
                  >
                    <span className="text-ink-muted">{a.label}</span>
                    <span className="font-medium tabular-nums text-ink">
                      {a.value.toLocaleString("de-DE", {
                        style: "currency",
                        currency: a.currency,
                      })}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Dates */}
          {doc.dates && doc.dates.length > 0 && (
            <div>
              <h3 className="mb-2 text-xs font-medium text-ink-faint">
                Important dates
              </h3>
              <div className="space-y-1">
                {doc.dates.map((d, i) => (
                  <div
                    key={i}
                    className="flex items-baseline justify-between text-sm"
                  >
                    <span className="text-ink-muted">{d.label}</span>
                    <span className="text-ink">{formatDate(d.date)}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Reference IDs */}
          {doc.reference_ids && doc.reference_ids.length > 0 && (
            <div>
              <h3 className="mb-2 text-xs font-medium text-ink-faint">
                References
              </h3>
              <div className="space-y-1">
                {doc.reference_ids.map((r, i) => (
                  <div key={i} className="flex items-baseline gap-2 text-sm">
                    <span className="rounded bg-surface-raised px-1.5 py-0.5 text-[10px] font-medium uppercase text-ink-faint">
                      {r.type}
                    </span>
                    <span className="font-mono text-xs text-ink-muted">
                      {r.value}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Confidence */}
          {doc.confidence != null && (
            <div>
              <h3 className="mb-1 text-xs font-medium text-ink-faint">
                Confidence
              </h3>
              <div className="flex items-center gap-2">
                <div className="h-1.5 flex-1 rounded-full bg-surface-raised">
                  <div
                    className="h-full rounded-full transition-all"
                    style={{
                      width: `${doc.confidence * 100}%`,
                      background:
                        doc.confidence >= 0.7
                          ? "var(--color-success)"
                          : doc.confidence >= 0.4
                            ? "var(--color-warning)"
                            : "var(--color-danger)",
                    }}
                  />
                </div>
                <span className="text-xs tabular-nums text-ink-muted">
                  {Math.round(doc.confidence * 100)}%
                </span>
              </div>
            </div>
          )}

          {/* Extracted text */}
          {doc.extracted_text && (
            <div>
              <h3 className="mb-2 text-xs font-medium text-ink-faint">
                Extracted text
              </h3>
              <pre className="max-h-60 overflow-y-auto whitespace-pre-wrap rounded-lg bg-surface-raised p-3 text-xs leading-relaxed text-ink-muted">
                {doc.extracted_text}
              </pre>
            </div>
          )}

          {/* Related documents */}
          {relatedDocs && relatedDocs.length > 0 && (
            <div>
              <h3 className="mb-2 text-xs font-medium text-ink-faint">
                Related documents
              </h3>
              <div className="space-y-1.5">
                {relatedDocs.map((n) => (
                  <Link
                    key={n.id}
                    to={`/documents/${n.id}`}
                    className="block rounded-lg border border-edge p-2.5 text-sm transition-colors hover:border-accent/30 hover:bg-surface-raised"
                  >
                    <p className="truncate font-medium text-ink">{n.label}</p>
                    {n.document?.document_date && (
                      <p className="mt-0.5 text-xs text-ink-faint">
                        {formatDate(n.document.document_date)}
                      </p>
                    )}
                  </Link>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function Field({
  icon: Icon,
  label,
  value,
  editing,
  onChange,
}: {
  icon: typeof Calendar;
  label: string;
  value: string | null | undefined;
  editing: boolean;
  onChange: (v: string) => void;
}) {
  return (
    <div>
      <div className="mb-1 flex items-center gap-1.5 text-xs font-medium text-ink-faint">
        <Icon size={12} />
        {label}
      </div>
      {editing ? (
        <input
          value={value ?? ""}
          onChange={(e) => onChange(e.target.value)}
          className="w-full rounded-md border border-edge bg-surface-raised px-2 py-1.5 text-sm text-ink outline-none focus:border-accent/50"
        />
      ) : (
        <p className="text-sm text-ink">{value || "—"}</p>
      )}
    </div>
  );
}
