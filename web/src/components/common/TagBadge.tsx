export default function TagBadge({
  tag,
  onClick,
}: {
  tag: string;
  onClick?: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="inline-flex items-center rounded-md bg-surface-raised px-2 py-0.5 text-xs text-ink-muted transition-colors hover:bg-surface-overlay hover:text-ink"
    >
      {tag}
    </button>
  );
}
