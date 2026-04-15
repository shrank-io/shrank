import { Link } from "react-router-dom";
import {
  FileText,
  HardDrive,
  Loader2,
  AlertTriangle,
  TrendingUp,
  RefreshCw,
} from "lucide-react";
import { useStats, useDocuments, useReprocessDocument } from "../api/hooks";
import { thumbnailUrl } from "../api/client";
import StatusBadge from "../components/common/StatusBadge";
import TagBadge from "../components/common/TagBadge";
import AuthImage from "../components/common/AuthImage";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

export default function Dashboard() {
  const { data: stats, isLoading } = useStats();
  const { data: recentData } = useDocuments({ limit: 10, sort: "created_at" });
  const recentDocs = recentData?.pages.flatMap((p) => p.documents) ?? [];
  const reprocess = useReprocessDocument();

  if (isLoading) {
    return (
      <div className="space-y-6 p-6">
        <div className="grid grid-cols-4 gap-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="skeleton h-24 rounded-xl" />
          ))}
        </div>
        <div className="skeleton h-64 rounded-xl" />
      </div>
    );
  }

  if (!stats) {
    return (
      <div className="flex flex-col items-center justify-center py-24 text-ink-faint">
        <p>Cannot reach the backend</p>
        <p className="text-sm">Make sure the server is running on :3420</p>
      </div>
    );
  }

  const pending = stats.by_status["pending"] ?? 0;
  const processing = stats.by_status["processing"] ?? 0;
  const errors = stats.by_status["error"] ?? 0;

  const statCards = [
    {
      label: "Total documents",
      value: stats.total_documents,
      icon: FileText,
      color: "text-accent",
    },
    {
      label: "Processing",
      value: pending + processing,
      icon: Loader2,
      color: "text-warning",
    },
    {
      label: "Errors",
      value: errors,
      icon: AlertTriangle,
      color: "text-danger",
    },
    {
      label: "Storage used",
      value: formatBytes(stats.storage_bytes),
      icon: HardDrive,
      color: "text-ink-muted",
    },
  ];

  return (
    <div className="space-y-6 p-6">
      <h1 className="text-xl font-semibold text-ink">Dashboard</h1>

      {/* Stat cards */}
      <div className="grid grid-cols-2 gap-4 lg:grid-cols-4">
        {statCards.map((card) => (
          <div
            key={card.label}
            className="rounded-xl border border-edge bg-surface p-4"
          >
            <div className="flex items-center gap-2">
              <card.icon size={16} className={card.color} />
              <span className="text-xs text-ink-faint">{card.label}</span>
            </div>
            <p className="mt-2 text-2xl font-semibold tabular-nums text-ink">
              {card.value}
            </p>
          </div>
        ))}
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        {/* Recent documents */}
        <div className="lg:col-span-2">
          <div className="mb-3 flex items-center justify-between">
            <h2 className="flex items-center gap-2 text-sm font-medium text-ink">
              <TrendingUp size={14} className="text-ink-faint" />
              Recent documents
            </h2>
            <Link
              to="/browse"
              className="text-xs text-accent hover:underline"
            >
              View all
            </Link>
          </div>
          <div className="space-y-2">
            {recentDocs.map((doc) => (
              <Link
                key={doc.id}
                to={`/documents/${doc.id}`}
                className="flex items-center gap-3 rounded-lg border border-edge bg-surface p-3 transition-colors hover:border-accent/30"
              >
                <div className="h-10 w-8 flex-shrink-0 overflow-hidden rounded bg-surface-raised">
                  <AuthImage
                    src={thumbnailUrl(doc.id)}
                    alt=""
                    className="h-full w-full object-cover"
                  />
                </div>
                <div className="flex-1 min-w-0">
                  <p className="truncate text-sm font-medium text-ink">
                    {doc.subject ?? doc.sender ?? "Processing..."}
                  </p>
                  <p className="text-xs text-ink-faint">
                    {doc.sender && `${doc.sender} · `}
                    {doc.document_date ?? doc.captured_at.slice(0, 10)}
                  </p>
                </div>
                <StatusBadge status={doc.status} />
                {doc.status === "error" && (
                  <button
                    onClick={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      reprocess.mutate(doc.id);
                    }}
                    className="rounded-md p-1 text-ink-faint transition-colors hover:bg-surface-raised hover:text-ink"
                    title="Retry processing"
                  >
                    <RefreshCw size={14} />
                  </button>
                )}
              </Link>
            ))}
            {recentDocs.length === 0 && (
              <p className="py-8 text-center text-sm text-ink-faint">
                No documents yet.{" "}
                <Link to="/upload" className="text-accent hover:underline">
                  Upload one
                </Link>
              </p>
            )}
          </div>
        </div>

        {/* Tag cloud */}
        <div>
          <h2 className="mb-3 text-sm font-medium text-ink">Tag cloud</h2>
          <div className="flex flex-wrap gap-1.5">
            {stats.tags.map((t) => (
              <Link key={t.name} to={`/search?q=tag:${t.name}`}>
                <TagBadge tag={`${t.name} (${t.count})`} />
              </Link>
            ))}
            {stats.tags.length === 0 && (
              <p className="text-xs text-ink-faint">
                Tags appear as documents are processed
              </p>
            )}
          </div>

          {/* Senders count */}
          <h2 className="mb-3 mt-6 text-sm font-medium text-ink">
            Senders
          </h2>
          <p className="text-sm text-ink-muted">
            {stats.unique_senders} unique sender{stats.unique_senders !== 1 ? "s" : ""}
          </p>
        </div>
      </div>
    </div>
  );
}
