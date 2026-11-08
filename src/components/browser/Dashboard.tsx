import { useEffect } from "react";
import {
  RefreshCw,
  ArrowRightLeft,
  History,
  Archive,
  AlertCircle,
} from "lucide-react";
import { useAppStore } from "@/store/appStore";
import { BrowserCard } from "./BrowserCard";
import { Spinner, Empty, SectionHeader } from "@/components/ui/index";
import { pluralize } from "@/utils";

export function Dashboard() {
  const {
    browsers,
    browsersLoading,
    browsersError,
    loadBrowsers,
    setView,
    migrationHistory,
    loadHistory,
  } = useAppStore((s) => ({
    browsers: s.browsers,
    browsersLoading: s.browsersLoading,
    browsersError: s.browsersError,
    loadBrowsers: s.loadBrowsers,
    setView: s.setView,
    migrationHistory: s.migrationHistory,
    loadHistory: s.loadHistory,
  }));

  useEffect(() => {
    loadBrowsers();
    loadHistory();
  }, []);

  const totalProfiles = browsers.reduce((n, b) => n + b.profiles.length, 0);
  const runningBrowsers = browsers.filter((b) => b.is_running);

  return (
    <div className="p-8 max-w-5xl mx-auto">
      {/* Page header */}
      <SectionHeader
        title="Installed Browsers"
        description={
          browsersLoading
            ? "Scanning your system…"
            : browsers.length > 0
            ? `${pluralize(browsers.length, "browser")} detected · ${pluralize(totalProfiles, "profile")}`
            : "No browsers detected yet"
        }
        action={
          <button
            onClick={loadBrowsers}
            disabled={browsersLoading}
            className="btn-secondary flex items-center gap-2 text-sm"
          >
            <RefreshCw
              className={`w-3.5 h-3.5 ${browsersLoading ? "animate-spin" : ""}`}
            />
            Refresh
          </button>
        }
      />

      {/* Running browsers warning */}
      {runningBrowsers.length > 0 && (
        <div className="mb-6 flex items-start gap-3 p-3 rounded-xl border border-amber-500/20 bg-amber-500/8 text-sm">
          <AlertCircle className="w-4 h-4 text-amber-400 shrink-0 mt-0.5" />
          <p className="text-amber-200/80">
            <strong className="text-amber-300">
              {runningBrowsers.map((b) => b.display_name).join(", ")}
            </strong>{" "}
            {runningBrowsers.length === 1 ? "is" : "are"} currently running.
            Close {runningBrowsers.length === 1 ? "it" : "them"} before
            migrating to avoid database lock errors.
          </p>
        </div>
      )}

      {/* States */}
      {browsersLoading ? (
        <div className="flex items-center justify-center py-20">
          <Spinner size="lg" />
        </div>
      ) : browsersError ? (
        <div className="card p-8 text-center">
          <AlertCircle className="w-10 h-10 text-red-400 mx-auto mb-3" />
          <p className="text-sm font-medium text-white mb-1">Detection failed</p>
          <p className="text-xs text-slate-400 mb-4">{browsersError}</p>
          <button onClick={loadBrowsers} className="btn-secondary text-sm">
            Try again
          </button>
        </div>
      ) : browsers.length === 0 ? (
        <Empty
          title="No browsers detected"
          description="Install Chrome, Firefox, Edge, Brave, or Opera to get started."
          action={
            <button onClick={loadBrowsers} className="btn-secondary text-sm">
              Scan again
            </button>
          }
        />
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          {browsers.map((browser) => (
            <BrowserCard key={browser.kind} browser={browser} />
          ))}
        </div>
      )}

      {/* Quick actions */}
      {browsers.length > 0 && (
        <div className="mt-10">
          <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-widest mb-4">
            Quick Actions
          </h3>
          <div className="grid grid-cols-3 gap-3">
            {[
              {
                icon: ArrowRightLeft,
                label: "Start Migration",
                desc: "Move data between browsers",
                view: "migrate" as const,
                primary: true,
              },
              {
                icon: History,
                label: "View History",
                desc: "Past migration runs",
                view: "history" as const,
                primary: false,
              },
              {
                icon: Archive,
                label: "Backups",
                desc: "Encrypted profile backups",
                view: "backups" as const,
                primary: false,
              },
            ].map(({ icon: Icon, label, desc, view, primary }) => (
              <button
                key={view}
                onClick={() => setView(view)}
                className={`card-hover p-4 text-left flex flex-col gap-2 ${
                  primary ? "border-brand-500/25" : ""
                }`}
              >
                <div
                  className={`w-8 h-8 rounded-lg flex items-center justify-center ${
                    primary ? "bg-brand-600/20" : "bg-surface-700"
                  }`}
                >
                  <Icon
                    className={`w-4 h-4 ${
                      primary ? "text-brand-400" : "text-slate-400"
                    }`}
                  />
                </div>
                <div>
                  <p className="text-sm font-medium text-white">{label}</p>
                  <p className="text-xs text-slate-500 mt-0.5">{desc}</p>
                </div>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Recent activity */}
      {migrationHistory.length > 0 && (
        <div className="mt-8">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-widest">
              Recent Migrations
            </h3>
            <button
              onClick={() => setView("history")}
              className="text-xs text-brand-400 hover:text-brand-300"
            >
              View all →
            </button>
          </div>
          <div className="card divide-y divide-white/6">
            {migrationHistory.slice(0, 3).map((run) => (
              <div
                key={run.id}
                className="flex items-center gap-3 px-4 py-3 text-sm"
              >
                <div
                  className={`w-2 h-2 rounded-full shrink-0 ${
                    run.status === "completed"
                      ? "bg-green-400"
                      : run.status === "failed"
                      ? "bg-red-400"
                      : run.status === "partial"
                      ? "bg-amber-400"
                      : "bg-amber-400"
                  }`}
                />
                <span className="text-slate-300 capitalize">
                  {run.source_browser} → {run.dest_browser}
                </span>
                <span className="text-slate-500 text-xs ml-auto">
                  {run.items_migrated} items
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
