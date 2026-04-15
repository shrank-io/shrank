import { NavLink } from "react-router-dom";
import {
  LayoutDashboard,
  FolderOpen,
  Search,
  Share2,
  Upload,
} from "lucide-react";

const links = [
  { to: "/", icon: LayoutDashboard, label: "Dashboard" },
  { to: "/browse", icon: FolderOpen, label: "Documents" },
  { to: "/search", icon: Search, label: "Search" },
  { to: "/graph", icon: Share2, label: "Graph" },
  { to: "/upload", icon: Upload, label: "Upload" },
] as const;

export default function Sidebar() {
  return (
    <aside className="fixed top-0 left-0 z-30 flex h-dvh w-56 flex-col border-r border-edge bg-surface">
      {/* Logo */}
      <NavLink
        to="/"
        className="flex items-center gap-2.5 border-b border-edge px-5 py-4"
      >
        <img src="/favicon.svg" alt="" className="h-7 w-7" />
        <span className="text-lg font-semibold tracking-tight text-ink">
          Shrank
        </span>
      </NavLink>

      {/* Nav */}
      <nav className="flex-1 space-y-0.5 px-3 py-4">
        {links.map(({ to, icon: Icon, label }) => (
          <NavLink
            key={to}
            to={to}
            end={to === "/"}
            className={({ isActive }) =>
              `flex items-center gap-2.5 rounded-lg px-3 py-2 text-sm font-medium transition-colors ${
                isActive
                  ? "bg-accent/10 text-accent"
                  : "text-ink-muted hover:bg-surface-raised hover:text-ink"
              }`
            }
          >
            <Icon size={18} />
            {label}
          </NavLink>
        ))}
      </nav>

      {/* Footer */}
      <div className="border-t border-edge px-5 py-3">
        <p className="text-[11px] text-ink-faint">Local archive</p>
      </div>
    </aside>
  );
}
