import { useState, type ElementType } from "react";
import {
  CheckCircle2,
  XCircle,
  SkipForward,
  AlertTriangle,
  Download,
  RotateCcw,
} from "lucide-react";
import { useAppStore } from "@/store/appStore";
import { StatusBadge } from "@/components/ui/index";
import { pluralize, groupBy } from "@/utils";
import type { MigrationItemResult } from "@/types";

const STATUS_ICONS: Record<string, ElementType> = {
  migrated: CheckCircle2,
  failed: XCircle,
  skipped: SkipForward,
  incompatible: AlertTriangle,
};
const STATUS_COLORS: Record<string, string> = {
  migrated: "text-green-400",
  failed: "text-red-400",
  skipped: "text-amber-400",
  incompatible: "text-amber-400",
};

export function MigrationReport() {
  const { activeStatus, resetWizard } = useAppStore((s) => ({
    activeStatus: s.wizard.activeStatus,
    resetWizard: s.resetWizard,
  }));

  const [filter, setFilter] = useState<string>("all");

  if (!activeStatus) return null;

  const { report, status } = activeStatus;

  const filtered =
    filter === "all" ? report : report.filter((r) => r.status === filter);
  const grouped = groupBy(filtered, (r) => r.category);

  const counts = {
    migrated: report.filter((r) => r.status === "migrated").length,
    failed: report.filter((r) => r.status === "failed").length,
    skipped: report.filter((r) => r.status === "skipped").length,
    incompatible: report.filter((r) => r.status === "incompatible").length,
  };

  const exportReport = () => {
    const content = JSON.stringify(
      {
        run_id: activeStatus.id,
        status: activeStatus.status,
        started_at: activeStatus.started_at,
        completed_at: activeStatus.completed_at,
        summary: counts,
        items: report,
      },
      null,
      2
    );
    const blob = new Blob([content], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `migration-report-${activeStatus.id.slice(0, 8)}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="space-y-6">
      {/* Summary cards */}
      <div className="grid grid-cols-4 gap-3">
        {(
          [
            { key: "migrated", label: "Migrated", color: "text-green-400", bg: "bg-green-500/10 border-green-500/20" },
            { key: "failed", label: "Failed", color: "text-red-400", bg: "bg-red-500/10 border-red-500/20" },
            { key: "skipped", label: "Skipped", color: "text-amber-400", bg: "bg-amber-500/10 border-amber-500/20" },
            { key: "incompatible", label: "Incompatible", color: "text-slate-400", bg: "bg-surface-700 border-white/8" },
          ] as const
        ).map(({ key, label, color, bg }) => (
          <button
            key={key}
            onClick={() => setFilter(filter === key ? "all" : key)}
            className={`p-3 rounded-xl border text-left transition-all ${bg} ${
              filter === key ? "ring-1 ring-white/20" : ""
            }`}
          >
            <p className={`text-2xl font-bold tabular-nums ${color}`}>
              {counts[key]}
            </p>
            <p className="text-xs text-slate-500 mt-0.5">{label}</p>
          </button>
        ))}
      </div>

      {/* Filter chips */}
      <div className="flex items-center gap-2">
        <span className="text-xs text-slate-500">Filter:</span>
        {["all", "migrated", "failed", "skipped", "incompatible"].map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`px-2.5 py-1 rounded-lg text-xs font-medium transition-all ${
              filter === f
                ? "bg-brand-600/20 text-brand-300 border border-brand-500/30"
                : "text-slate-500 hover:text-slate-300 hover:bg-white/5"
            }`}
          >
            {f.charAt(0).toUpperCase() + f.slice(1)}
          </button>
        ))}
        <div className="ml-auto flex items-center gap-2">
          <button
            onClick={exportReport}
            className="btn-ghost text-xs flex items-center gap-1.5"
          >
            <Download className="w-3.5 h-3.5" />
            Export JSON
          </button>
        </div>
      </div>

      {/* Items grouped by category */}
      <div className="space-y-4 max-h-96 overflow-y-auto pr-1">
        {Object.entries(grouped).map(([category, items]) => (
          <div key={category}>
            <p className="text-xs font-semibold text-slate-500 uppercase tracking-wider mb-2">
              {category}{" "}
              <span className="text-slate-600 font-normal">
                ({pluralize(items.length, "item")})
              </span>
            </p>
            <div className="card divide-y divide-white/5">
              {items.map((item) => (
                <ReportItem key={item.item_key} item={item} />
              ))}
            </div>
          </div>
        ))}
        {filtered.length === 0 && (
          <p className="text-sm text-slate-500 text-center py-8">
            No items match this filter
          </p>
        )}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-3 pt-2">
        <button
          onClick={resetWizard}
          className="btn-secondary flex items-center gap-2 text-sm"
        >
          <RotateCcw className="w-4 h-4" />
          New Migration
        </button>
        {status === "completed" && (
          <p className="text-xs text-slate-500">
            ✓ Files exported to destination profile folder
          </p>
        )}
      </div>
    </div>
  );
}

function ReportItem({ item }: { item: MigrationItemResult }) {
  const Icon = STATUS_ICONS[item.status] ?? CheckCircle2;
  const color = STATUS_COLORS[item.status] ?? "text-slate-400";

  return (
    <div className="flex items-start gap-3 px-3.5 py-2.5">
      <Icon className={`w-3.5 h-3.5 mt-0.5 shrink-0 ${color}`} />
      <div className="flex-1 min-w-0">
        <p className="text-xs text-slate-300 truncate">{item.label}</p>
        {item.message && (
          <p className="text-xs text-slate-600 mt-0.5 leading-relaxed">
            {item.message}
          </p>
        )}
      </div>
      <StatusBadge status={item.status} />
    </div>
  );
}
