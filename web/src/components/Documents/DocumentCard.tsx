import { Link } from "react-router-dom";
import { FileText, Calendar } from "lucide-react";
import { thumbnailUrl } from "../../api/client";
import StatusBadge from "../common/StatusBadge";
import TagBadge from "../common/TagBadge";
import AuthImage from "../common/AuthImage";
import type { Document } from "../../api/types";

function formatDate(iso: string | null): string {
  if (!iso) return "";
  return new Date(iso).toLocaleDateString("de-DE", {
    day: "2-digit",
    month: "short",
    year: "numeric",
  });
}

export default function DocumentCard({ doc }: { doc: Document }) {
  const displayDate = doc.document_date ?? doc.captured_at;

  return (
    <Link
      to={`/documents/${doc.id}`}
      className="group flex flex-col overflow-hidden rounded-xl border border-edge bg-surface transition-all hover:border-accent/30 hover:shadow-lg hover:shadow-accent/5"
    >
      {/* Thumbnail */}
      <div className="relative aspect-[3/4] overflow-hidden bg-surface-raised">
        <AuthImage
          src={thumbnailUrl(doc.id)}
          alt={doc.subject ?? "Document"}
          className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-[1.03]"
        />
        {/* Fallback when no thumbnail */}
        <div className="absolute inset-0 flex items-center justify-center text-ink-faint">
          <FileText size={40} strokeWidth={1} />
        </div>
        {/* Status overlay */}
        <div className="absolute top-2 right-2">
          <StatusBadge status={doc.status} />
        </div>
      </div>

      {/* Info */}
      <div className="flex flex-1 flex-col gap-1.5 p-3">
        {doc.sender && (
          <p className="truncate text-sm font-medium text-ink">
            {doc.sender}
          </p>
        )}
        {doc.subject && (
          <p className="line-clamp-2 text-xs text-ink-muted">{doc.subject}</p>
        )}
        <div className="mt-auto flex items-center gap-1.5 pt-2 text-xs text-ink-faint">
          <Calendar size={12} />
          {formatDate(displayDate)}
          {doc.document_type && (
            <>
              <span className="text-edge">|</span>
              <span className="capitalize">{doc.document_type}</span>
            </>
          )}
        </div>
        {doc.tags && doc.tags.length > 0 && (
          <div className="flex flex-wrap gap-1 pt-1">
            {doc.tags.slice(0, 3).map((t) => (
              <TagBadge key={t} tag={t} />
            ))}
            {doc.tags.length > 3 && (
              <span className="text-[10px] text-ink-faint">
                +{doc.tags.length - 3}
              </span>
            )}
          </div>
        )}
      </div>
    </Link>
  );
}
