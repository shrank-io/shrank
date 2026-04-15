import { useState, useEffect } from "react";
import { useSearchParams } from "react-router-dom";
import { useSearch } from "../api/hooks";
import SearchBar from "../components/Search/SearchBar";
import SearchResults from "../components/Search/SearchResults";
import FacetFilters from "../components/Search/FacetFilters";

export default function SearchPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const initialQuery = searchParams.get("q") ?? "";
  const [query, setQuery] = useState(initialQuery);
  const [activeQuery, setActiveQuery] = useState(initialQuery);
  const [activeFilters, setActiveFilters] = useState<Record<string, string>>(
    {},
  );

  // Sync URL → state on navigation
  useEffect(() => {
    const q = searchParams.get("q") ?? "";
    setQuery(q);
    setActiveQuery(q);
  }, [searchParams]);

  const { data, isLoading } = useSearch(
    buildQuery(activeQuery, activeFilters),
    50,
  );

  const results = data?.pages.flatMap((p) => p.results) ?? [];
  const facets = data?.pages[0]?.facets;
  const total = data?.pages[0]?.total ?? 0;

  function handleSubmit() {
    setActiveQuery(query);
    setSearchParams(query ? { q: query } : {});
  }

  function toggleFilter(key: string, value: string) {
    setActiveFilters((prev) => {
      const next = { ...prev };
      if (next[key] === value) {
        delete next[key];
      } else {
        next[key] = value;
      }
      return next;
    });
  }

  return (
    <div className="flex h-full">
      {/* Sidebar */}
      <aside className="hidden w-56 flex-shrink-0 overflow-y-auto border-r border-edge p-4 lg:block">
        <FacetFilters
          facets={facets}
          activeFilters={activeFilters}
          onToggle={toggleFilter}
        />
      </aside>

      {/* Main */}
      <div className="flex-1 overflow-y-auto p-6">
        <SearchBar
          value={query}
          onChange={setQuery}
          onSubmit={handleSubmit}
        />

        {activeQuery && (
          <p className="mt-4 text-xs text-ink-faint">
            {isLoading ? "Searching..." : `${total} result${total !== 1 ? "s" : ""}`}
          </p>
        )}

        <div className="mt-4">
          {isLoading && activeQuery ? (
            <div className="space-y-3">
              {Array.from({ length: 5 }).map((_, i) => (
                <div key={i} className="skeleton h-28 rounded-xl" />
              ))}
            </div>
          ) : activeQuery ? (
            <SearchResults results={results} />
          ) : (
            <div className="flex flex-col items-center py-24 text-ink-faint">
              <p className="text-lg">Search your archive</p>
              <p className="mt-1 text-sm">
                Keywords, natural language, or structured queries like
                sender:"AOK" type:invoice
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function buildQuery(q: string, filters: Record<string, string>): string {
  let result = q;
  for (const [key, value] of Object.entries(filters)) {
    result += ` ${key}:"${value}"`;
  }
  return result.trim();
}
