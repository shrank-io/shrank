import { Search, X } from "lucide-react";
import { useRef } from "react";

export default function SearchBar({
  value,
  onChange,
  onSubmit,
}: {
  value: string;
  onChange: (v: string) => void;
  onSubmit: () => void;
}) {
  const inputRef = useRef<HTMLInputElement>(null);

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        onSubmit();
      }}
      className="relative"
    >
      <Search
        size={18}
        className="absolute left-4 top-1/2 -translate-y-1/2 text-ink-faint"
      />
      <input
        ref={inputRef}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder='Search documents... Try "Krankenkasse 2026" or "sender:AOK type:invoice"'
        className="h-12 w-full rounded-xl border border-edge bg-surface pl-11 pr-10 text-sm text-ink placeholder:text-ink-faint outline-none transition-colors focus:border-accent/50 focus:ring-2 focus:ring-accent/15"
      />
      {value && (
        <button
          type="button"
          onClick={() => {
            onChange("");
            inputRef.current?.focus();
          }}
          className="absolute right-3 top-1/2 -translate-y-1/2 rounded-md p-1 text-ink-faint hover:text-ink"
        >
          <X size={16} />
        </button>
      )}
    </form>
  );
}
