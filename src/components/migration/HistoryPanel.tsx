import { useEffect } from "react";
import { History, CheckCircle2, XCircle, Ban, Loader } from "lucide-react";
import { useAppStore } from "@/store/appStore";
import { Empty, Spinner, SectionHeader, StatusBadge } from "@/components/ui/index";
import { formatRelativeTime, formatDateTime, pluralize } from "@/utils";
import { BrowserIcon } from "@/components/ui/BrowserIcon";
import type { BrowserKind } from "@/types";

export function HistoryPanel() {
  const { migrationHistory, historyLoading, loadHistory } = useAppStore((s) => ({
    migrationHistory: s.migrationHistory,
    historyLoading: s.historyLoading,
    loadHistory: s.loadHistory,
  }));

  useEffect(() => {
    loadHistory();
  }, []);

  return (
    <div className="p-8 max-w-4xl mx-auto">
      <SectionHeader
        title="Migration History"
        description="All past migration runs and their results"
        action={
          <button
            onClick={loadHistory}
            className="btn-secondary text-sm"
            disabled={historyLoading}
          >
            Refresh
          </button>
        }
      />

      {historyLoading ? (
        <div className="flex justify-center py-20">
          <Spinner size="lg" />
        </div>
      ) : migrationHistory.length === 0 ? (
        <Empty
          icon={History}
          title="No migrations yet"
          description="Start a migration from the Migrate tab to see history here."
        />
      ) : (
        <div className="space-y-3">
          {migrationHistory.map((run) => {
            const categories = (() => {
              try {
                return JSON.parse(run.data_types) as string[];
              } catch {
                return [run.data_types];
              }
            })();

            return (
              <div key={run.id} className="card p-4">
                <div className="flex items-start gap-4">
                  {/* Status icon */}
                  <div className="mt-0.5">
                    {run.status === "completed" ? (
                      <CheckCircle2 className="w-5 h-5 text-green-400" />
                    ) : run.status === "failed" ? (
                      <XCircle className="w-5 h-5 text-red-400" />
                    ) : run.status === "cancelled" ? (
                      <Ban className="w-5 h-5 text-amber-400" />
                    ) : (
                      <Loader className="w-5 h-5 text-slate-400" />
                    )}
                  </div>

                  {/* Browser pair */}
                  <div className="flex items-center gap-2 shrink-0">
                    <BrowserIcon
                      kind={run.source_browser as BrowserKind}
                      size={20}
                    />
                    <span className="text-xs text-slate-600">→</span>
                    <BrowserIcon
                      kind={run.dest_browser as BrowserKind}
                      size={20}
                    />
                  </div>

                  {/* Details */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap">
                      <span className="text-sm font-medium text-white capitalize">
                        {run.source_browser} → {run.dest_browser}
                      </span>
                      <StatusBadge status={run.status} />
                    </div>
                    <p className="text-xs text-slate-500 mt-0.5">
                      {run.source_profile} → {run.dest_profile}
                    </p>
                    <div className="flex flex-wrap gap-1 mt-2">
                      {categories.map((cat) => (
                        <span
                          key={cat}
                          className="px-1.5 py-0.5 rounded text-xs bg-surface-700 text-slate-400"
                        >
                          {cat}
                        </span>
                      ))}
                    </div>
                  </div>

                  {/* Stats + time */}
                  <div className="text-right shrink-0">
                    <p className="text-sm font-medium text-white">
                      {pluralize(run.items_migrated, "item")}
                    </p>
                    {run.items_failed > 0 && (
                      <p className="text-xs text-red-400">
                        {run.items_failed} failed
                      </p>
                    )}
                    <p
                      className="text-xs text-slate-500 mt-1"
                      title={formatDateTime(run.started_at)}
                    >
                      {formatRelativeTime(run.started_at)}
                    </p>
                  </div>
                </div>

                {/* Error message */}
                {run.error_message && (
                  <div className="mt-3 px-3 py-2 rounded-lg bg-red-500/8 border border-red-500/20 text-xs text-red-300">
                    {run.error_message}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
