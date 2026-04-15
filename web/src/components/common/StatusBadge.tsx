import { Loader2, CheckCircle2, AlertTriangle, Clock } from "lucide-react";
import type { Document } from "../../api/types";

const config: Record<
  Document["status"],
  { icon: typeof Clock; label: string; class: string }
> = {
  pending: {
    icon: Clock,
    label: "Pending",
    class: "bg-warning/15 text-warning",
  },
  processing: {
    icon: Loader2,
    label: "Processing",
    class: "bg-accent-dim text-accent",
  },
  complete: {
    icon: CheckCircle2,
    label: "Complete",
    class: "bg-success/15 text-success",
  },
  error: {
    icon: AlertTriangle,
    label: "Error",
    class: "bg-danger/15 text-danger",
  },
};

export default function StatusBadge({ status }: { status: Document["status"] }) {
  const c = config[status];
  const Icon = c.icon;
  return (
    <span
      className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium ${c.class}`}
    >
      <Icon
        size={12}
        className={status === "processing" ? "animate-spin" : ""}
      />
      {c.label}
    </span>
  );
}
