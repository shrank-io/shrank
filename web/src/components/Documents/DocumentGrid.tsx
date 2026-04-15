import { useEffect, useRef } from "react";
import DocumentCard from "./DocumentCard";
import type { Document } from "../../api/types";

interface Props {
  documents: Document[];
  hasNextPage: boolean;
  isFetchingNextPage: boolean;
  fetchNextPage: () => void;
}

export default function DocumentGrid({
  documents,
  hasNextPage,
  isFetchingNextPage,
  fetchNextPage,
}: Props) {
  const sentinel = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!sentinel.current || !hasNextPage) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting && !isFetchingNextPage) fetchNextPage();
      },
      { rootMargin: "400px" },
    );
    observer.observe(sentinel.current);
    return () => observer.disconnect();
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

  if (documents.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-24 text-ink-faint">
        <p className="text-lg">No documents yet</p>
        <p className="text-sm">Upload some documents to get started</p>
      </div>
    );
  }

  return (
    <>
      <div className="grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-4">
        {documents.map((doc) => (
          <DocumentCard key={doc.id} doc={doc} />
        ))}
        {isFetchingNextPage &&
          Array.from({ length: 6 }).map((_, i) => (
            <div key={`skel-${i}`} className="skeleton aspect-[3/5] rounded-xl" />
          ))}
      </div>
      <div ref={sentinel} className="h-1" />
    </>
  );
}
