import type { Facets } from "../../api/types";

interface Props {
  facets: Facets | undefined;
  activeFilters: Record<string, string>;
  onToggle: (key: string, value: string) => void;
}

export default function FacetFilters({ facets, activeFilters, onToggle }: Props) {
  if (!facets) return null;

  const sections: { key: string; label: string; items: { name: string; count: number }[] }[] = [
    { key: "sender", label: "Sender", items: facets.senders },
    { key: "type", label: "Type", items: facets.types },
    { key: "tag", label: "Tags", items: facets.tags },
    { key: "year", label: "Year", items: facets.years },
  ];

  return (
    <div className="space-y-5">
      {sections.map(
        (section) =>
          section.items.length > 0 && (
            <div key={section.key}>
              <h3 className="mb-2 text-xs font-medium uppercase tracking-wider text-ink-faint">
                {section.label}
              </h3>
              <div className="space-y-0.5">
                {section.items.slice(0, 10).map((item) => {
                  const isActive = activeFilters[section.key] === item.name;
                  return (
                    <button
                      key={item.name}
                      onClick={() => onToggle(section.key, item.name)}
                      className={`flex w-full items-center justify-between rounded-md px-2 py-1.5 text-left text-sm transition-colors ${
                        isActive
                          ? "bg-accent/10 text-accent"
                          : "text-ink-muted hover:bg-surface-raised hover:text-ink"
                      }`}
                    >
                      <span className="truncate">{item.name}</span>
                      <span className="ml-2 text-xs tabular-nums text-ink-faint">
                        {item.count}
                      </span>
                    </button>
                  );
                })}
              </div>
            </div>
          ),
      )}

      {Object.keys(activeFilters).length > 0 && (
        <button
          onClick={() => {
            for (const k of Object.keys(activeFilters)) {
              onToggle(k, activeFilters[k]);
            }
          }}
          className="w-full rounded-md py-1.5 text-center text-xs font-medium text-ink-faint transition-colors hover:text-ink"
        >
          Clear all filters
        </button>
      )}
    </div>
  );
}
