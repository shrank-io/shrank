import { useNavigate } from "react-router-dom";
import { Search } from "lucide-react";
import { useState, type FormEvent } from "react";

export default function Header() {
  const [query, setQuery] = useState("");
  const navigate = useNavigate();

  function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (query.trim()) {
      navigate(`/search?q=${encodeURIComponent(query.trim())}`);
    }
  }

  return (
    <header className="sticky top-0 z-20 flex h-14 items-center gap-4 border-b border-edge bg-surface/80 px-6 backdrop-blur">
      <form onSubmit={handleSubmit} className="relative flex-1 max-w-lg">
        <Search
          size={16}
          className="absolute left-3 top-1/2 -translate-y-1/2 text-ink-faint"
        />
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search documents..."
          className="h-9 w-full rounded-lg border border-edge bg-surface-raised pl-9 pr-3 text-sm text-ink placeholder:text-ink-faint outline-none transition-colors focus:border-accent/50 focus:ring-1 focus:ring-accent/25"
        />
      </form>
    </header>
  );
}
