import { Link } from "react-router-dom";
import { Calendar, Building2 } from "lucide-react";
import { thumbnailUrl } from "../../api/client";
import StatusBadge from "../common/StatusBadge";
import TagBadge from "../common/TagBadge";
import AuthImage from "../common/AuthImage";
import type { SearchResult } from "../../api/types";

function formatDate(iso: string | null): string {
  if (!iso) return "";
  return new Date(iso).toLocaleDateString("de-DE", {
    day: "2-digit",
    month: "short",
    year: "numeric",
  });
}

export default function SearchResults({
  results,
}: {
  results: SearchResult[];
}) {
  if (results.length === 0) {
    return (
      <div className="flex flex-col items-center py-16 text-ink-faint">
        <p className="text-lg">No results</p>
        <p className="text-sm">Try different keywords or broaden your filters</p>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {results.map(({ document: doc, score, match_sources, highlights }) => (
        <Link
          key={doc.id}
          to={`/documents/${doc.id}`}
          className="flex gap-4 rounded-xl border border-edge bg-surface p-4 transition-all hover:border-accent/30 hover:shadow-lg hover:shadow-accent/5"
        >
          {/* Thumbnail */}
          <div className="hidden h-24 w-18 flex-shrink-0 overflow-hidden rounded-lg bg-surface-raised sm:block">
            <AuthImage
              src={thumbnailUrl(doc.id)}
              alt=""
              className="h-full w-full object-cover"
            />
          </div>

          {/* Content */}
          <div className="flex-1 min-w-0">
            <div className="flex items-start gap-2">
              <h3 className="truncate text-sm font-medium text-ink">
                {doc.subject ?? "Untitled"}
              </h3>
              <StatusBadge status={doc.status} />
            </div>

            {/* Highlight */}
            {highlights.extracted_text && (
              <p
                className="mt-1 line-clamp-2 text-xs text-ink-muted"
                dangerouslySetInnerHTML={{
                  __html: highlights.extracted_text,
                }}
              />
            )}

            <div className="mt-2 flex flex-wrap items-center gap-3 text-xs text-ink-faint">
              {doc.sender && (
                <span className="flex items-center gap-1">
                  <Building2 size={11} />
                  {doc.sender}
                </span>
              )}
              {doc.document_date && (
                <span className="flex items-center gap-1">
                  <Calendar size={11} />
                  {formatDate(doc.document_date)}
                </span>
              )}
              <span className="ml-auto tabular-nums text-ink-faint/60">
                {(score * 100).toFixed(0)}% — {match_sources.join(", ")}
              </span>
            </div>

            {doc.tags && doc.tags.length > 0 && (
              <div className="mt-2 flex flex-wrap gap-1">
                {doc.tags.slice(0, 5).map((t) => (
                  <TagBadge key={t} tag={t} />
                ))}
              </div>
            )}
          </div>
        </Link>
      ))}
    </div>
  );
}
