import {
  LayoutDashboard,
  ArrowRightLeft,
  History,
  Archive,
  Settings,
  Github,
} from "lucide-react";
import type { ElementType } from "react";
import { cn } from "@/utils";
import { useAppStore, type AppView } from "@/store/appStore";

const NAV_ITEMS: { id: AppView; label: string; icon: ElementType }[] = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { id: "migrate", label: "Migrate", icon: ArrowRightLeft },
  { id: "history", label: "History", icon: History },
  { id: "backups", label: "Backups", icon: Archive },
  { id: "settings", label: "Settings", icon: Settings },
];

export function Sidebar() {
  const { view, setView } = useAppStore((s) => ({
    view: s.view,
    setView: s.setView,
  }));

  return (
    <aside className="w-56 shrink-0 flex flex-col bg-surface-900 border-r border-white/8 h-full">
      {/* Logo */}
      <div
        className="h-14 flex items-center gap-2.5 px-5 border-b border-white/8"
        data-tauri-drag-region
      >
        <div className="w-7 h-7 rounded-lg bg-brand-600 flex items-center justify-center shrink-0">
          <ArrowRightLeft className="w-3.5 h-3.5 text-white" />
        </div>
        <div className="leading-tight">
          <p className="text-xs font-semibold text-white">Universal</p>
          <p className="text-xs text-slate-400">Browser Sync</p>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 px-3 py-4 space-y-0.5">
        {NAV_ITEMS.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => setView(id)}
            className={cn(
              "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-all duration-150",
              view === id
                ? "bg-brand-600/20 text-brand-300 border border-brand-500/20"
                : "text-slate-400 hover:text-slate-200 hover:bg-white/5"
            )}
          >
            <Icon className="w-4 h-4 shrink-0" />
            {label}
          </button>
        ))}
      </nav>

      {/* Footer */}
      <div className="px-3 py-4 border-t border-white/8">
        <a
          href="https://github.com/your-org/universal-browser-sync"
          target="_blank"
          rel="noreferrer"
          className="flex items-center gap-2 px-3 py-2 rounded-lg text-xs text-slate-500 hover:text-slate-300 hover:bg-white/5 transition-all"
        >
          <Github className="w-3.5 h-3.5" />
          View on GitHub
        </a>
        <p className="px-3 mt-1 text-xs text-slate-600">v1.0.0</p>
      </div>
    </aside>
  );
}
