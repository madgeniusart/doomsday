import {
  Bookmark,
  History,
  Cookie,
  Puzzle,
  Settings,
  KeyRound,
  Search,
  ShieldCheck,
  Database,
  CheckSquare,
  Square,
} from "lucide-react";
import type { ElementType } from "react";
import { CATEGORY_META, type DataCategory } from "@/types";
import { RiskBadge } from "@/components/ui/index";
import { cn } from "@/utils";

const ICON_MAP: Record<string, ElementType> = {
  Bookmark,
  History,
  Cookie,
  Puzzle,
  Settings,
  KeyRound,
  Search,
  ShieldCheck,
  Database,
};

// Categories available in MVP
const AVAILABLE: DataCategory[] = [
  "bookmarks",
  "history",
  "cookies",
  "extensions",
  "preferences",
];

const COMING_SOON: DataCategory[] = [
  "passwords",
  "search_engines",
  "permissions",
  "local_storage",
];

interface CategorySelectorProps {
  selected: DataCategory[];
  onToggle: (cat: DataCategory) => void;
  onSelectAll: (cats: DataCategory[]) => void;
}

export function CategorySelector({
  selected,
  onToggle,
  onSelectAll,
}: CategorySelectorProps) {
  const allSelected = AVAILABLE.every((c) => selected.includes(c));

  return (
    <div className="space-y-6">
      {/* Select all */}
      <div className="flex items-center justify-between">
        <p className="text-sm text-slate-400">
          Choose which data to migrate
        </p>
        <button
          onClick={() =>
            allSelected ? onSelectAll([]) : onSelectAll(AVAILABLE)
          }
          className="text-xs text-brand-400 hover:text-brand-300 transition-colors"
        >
          {allSelected ? "Deselect all" : "Select all"}
        </button>
      </div>

      {/* Available categories */}
      <div className="space-y-2">
        {AVAILABLE.map((cat) => {
          const meta = CATEGORY_META[cat];
          const Icon = ICON_MAP[meta.icon] ?? Bookmark;
          const isSelected = selected.includes(cat);

          return (
            <button
              key={cat}
              onClick={() => onToggle(cat)}
              className={cn(
                "w-full flex items-start gap-3 p-3.5 rounded-xl border text-left transition-all duration-150",
                isSelected
                  ? "bg-brand-500/8 border-brand-500/30"
                  : "bg-surface-900/50 border-white/6 hover:border-white/12 hover:bg-white/2"
              )}
            >
              {/* Checkbox */}
              <div className="mt-0.5 shrink-0">
                {isSelected ? (
                  <CheckSquare className="w-4 h-4 text-brand-400" />
                ) : (
                  <Square className="w-4 h-4 text-slate-600" />
                )}
              </div>

              {/* Icon */}
              <div
                className={cn(
                  "w-8 h-8 rounded-lg flex items-center justify-center shrink-0",
                  isSelected ? "bg-brand-500/15" : "bg-surface-700"
                )}
              >
                <Icon
                  className={cn(
                    "w-4 h-4",
                    isSelected ? "text-brand-400" : "text-slate-400"
                  )}
                />
              </div>

              {/* Text */}
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 flex-wrap">
                  <span
                    className={cn(
                      "text-sm font-medium",
                      isSelected ? "text-white" : "text-slate-300"
                    )}
                  >
                    {meta.label}
                  </span>
                  <RiskBadge level={meta.risk} />
                </div>
                <p className="text-xs text-slate-500 mt-0.5">
                  {meta.description}
                </p>
                {meta.riskNote && (
                  <p className="text-xs text-slate-600 mt-0.5 italic">
                    {meta.riskNote}
                  </p>
                )}
              </div>
            </button>
          );
        })}
      </div>

      {/* Coming soon */}
      {COMING_SOON.length > 0 && (
        <div>
          <p className="text-xs font-semibold text-slate-600 uppercase tracking-widest mb-2">
            Coming soon
          </p>
          <div className="space-y-1.5">
            {COMING_SOON.map((cat) => {
              const meta = CATEGORY_META[cat];
              const Icon = ICON_MAP[meta.icon] ?? Bookmark;
              return (
                <div
                  key={cat}
                  className="flex items-center gap-3 p-3 rounded-xl border border-white/4 opacity-40 cursor-not-allowed"
                >
                  <Square className="w-4 h-4 text-slate-600 shrink-0" />
                  <div className="w-8 h-8 rounded-lg bg-surface-700 flex items-center justify-center shrink-0">
                    <Icon className="w-4 h-4 text-slate-600" />
                  </div>
                  <div>
                    <span className="text-sm text-slate-500">{meta.label}</span>
                    <RiskBadge level={meta.risk} />
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
}
