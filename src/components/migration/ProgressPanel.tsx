import { XCircle, CheckCircle2, AlertCircle, Ban } from "lucide-react";
import { Spinner, ProgressBar, Alert } from "@/components/ui/index";
import { useAppStore } from "@/store/appStore";
import { pluralize } from "@/utils";
import type { MigrationStatus } from "@/types";

export function ProgressPanel() {
  const { activeStatus, cancelMigration } = useAppStore((s) => ({
    activeStatus: s.wizard.activeStatus,
    cancelMigration: s.cancelMigration,
  }));

  if (!activeStatus) {
    return (
      <div className="flex flex-col items-center justify-center py-16 gap-4">
        <Spinner size="lg" />
        <p className="text-sm text-slate-400">Starting migration…</p>
      </div>
    );
  }

  const { status, progress, current_category, items_processed, items_total, items_failed, errors } =
    activeStatus;

  const isActive = status === "running" || status === "pending";
  const isDone = status === "completed" || status === "failed" || status === "cancelled";

  return (
    <div className="space-y-6">
      {/* Status header */}
      <div className="flex items-center gap-3">
        {isActive && <Spinner size="md" />}
        {status === "completed" && (
          <CheckCircle2 className="w-6 h-6 text-green-400" />
        )}
        {status === "failed" && (
          <XCircle className="w-6 h-6 text-red-400" />
        )}
        {status === "cancelled" && (
          <Ban className="w-6 h-6 text-amber-400" />
        )}

        <div>
          <p className="text-base font-semibold text-white capitalize">
            {status === "running" && current_category
              ? `Migrating ${current_category}…`
              : status === "pending"
              ? "Starting…"
              : status === "completed"
              ? "Migration complete"
              : status === "failed"
              ? "Migration failed"
              : "Migration cancelled"}
          </p>
          <p className="text-xs text-slate-400 mt-0.5">
            {items_processed} of {items_total}{" "}
            {pluralize(items_total, "category", "categories")} processed
            {items_failed > 0 && ` · ${items_failed} failed`}
          </p>
        </div>

        {isActive && (
          <button
            onClick={cancelMigration}
            className="ml-auto btn-ghost text-xs flex items-center gap-1.5 text-red-400 hover:text-red-300"
          >
            <XCircle className="w-3.5 h-3.5" />
            Cancel
          </button>
        )}
      </div>

      {/* Progress bar */}
      <ProgressBar value={progress} showPercent />

      {/* Category steps */}
      <CategorySteps status={activeStatus} />

      {/* Errors */}
      {errors.length > 0 && (
        <div className="space-y-2">
          {errors.map((err, i) => (
            <Alert key={i} type="error">
              {err}
            </Alert>
          ))}
        </div>
      )}

      {/* Dry run note */}
      {isDone && status === "completed" && (
        <Alert type="info">
          Migration complete. Check the report on the next screen for per-item
          details.
        </Alert>
      )}
    </div>
  );
}

function CategorySteps({ status }: { status: MigrationStatus }) {
  const categories = status.report
    .reduce<string[]>((acc, item) => {
      if (!acc.includes(item.category)) acc.push(item.category);
      return acc;
    }, [])
    .concat(
      status.current_category &&
        !status.report.find((r) => r.category === status.current_category)
        ? [status.current_category]
        : []
    );

  if (categories.length === 0) return null;

  return (
    <div className="space-y-2">
      {categories.map((cat) => {
        const items = status.report.filter((r) => r.category === cat);
        const failed = items.filter((r) => r.status === "failed").length;
        const isCurrent = cat === status.current_category;

        return (
          <div
            key={cat}
            className="flex items-center gap-3 p-3 rounded-lg bg-surface-900/60 border border-white/5"
          >
            {isCurrent && status.status === "running" ? (
              <Spinner size="sm" />
            ) : failed > 0 ? (
              <AlertCircle className="w-4 h-4 text-amber-400 shrink-0" />
            ) : items.length > 0 ? (
              <CheckCircle2 className="w-4 h-4 text-green-400 shrink-0" />
            ) : (
              <div className="w-4 h-4 rounded-full border border-slate-600 shrink-0" />
            )}
            <span
              className={`text-sm flex-1 ${
                isCurrent ? "text-white font-medium" : "text-slate-400"
              }`}
            >
              {cat}
            </span>
            {items.length > 0 && (
              <span className="text-xs text-slate-500">
                {items.length} items
              </span>
            )}
            {failed > 0 && (
              <span className="text-xs text-amber-400">{failed} failed</span>
            )}
          </div>
        );
      })}
    </div>
  );
}
