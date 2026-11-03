import { useEffect, useState } from "react";
import {
  Archive,
  Plus,
  Trash2,
  Lock,
  Unlock,
  RefreshCw,
} from "lucide-react";
import { useAppStore } from "@/store/appStore";
import { createBackup } from "@/services/tauri";
import { Empty, Spinner, SectionHeader, Badge, Alert } from "@/components/ui/index";
import { formatBytes, formatRelativeTime, formatDateTime } from "@/utils";
import { open } from "@tauri-apps/plugin-dialog";
import type { InstalledBrowser, BrowserProfile } from "@/types";

export function BackupPanel() {
  const {
    backups,
    backupsLoading,
    loadBackups,
    deleteBackup,
    browsers,
    loadBrowsers,
    notify,
    settings,
  } = useAppStore((s) => ({
    backups: s.backups,
    backupsLoading: s.backupsLoading,
    loadBackups: s.loadBackups,
    deleteBackup: s.deleteBackup,
    browsers: s.browsers,
    loadBrowsers: s.loadBrowsers,
    notify: s.notify,
    settings: s.settings,
  }));

  const [creating, setCreating] = useState(false);
  const [selectedBrowser, setSelectedBrowser] = useState<InstalledBrowser | null>(null);
  const [selectedProfile, setSelectedProfile] = useState<BrowserProfile | null>(null);
  const [encrypt, setEncrypt] = useState(settings.backup_encryption);

  useEffect(() => {
    loadBackups();
    if (browsers.length === 0) loadBrowsers();
  }, []);

  const handleCreate = async () => {
    if (!selectedBrowser || !selectedProfile) {
      notify("error", "Select a browser profile to back up");
      return;
    }

    // Ask for output directory. `open()` can resolve to a string, an array
    // of strings (if multiple selected is ever enabled), or null (cancelled).
    const result = await open({
      directory: true,
      multiple: false,
      title: "Choose backup destination folder",
    });

    const outputDir = Array.isArray(result) ? result[0] : result;
    if (!outputDir) return;

    setCreating(true);
    try {
      await createBackup({
        browser: selectedBrowser.kind,
        profile_path: selectedProfile.path,
        output_dir: outputDir,
        encrypt,
      });
      notify("success", "Backup created successfully");
      loadBackups();
    } catch (err) {
      notify("error", `Backup failed: ${err}`);
    } finally {
      setCreating(false);
    }
  };

  return (
    <div className="p-8 max-w-4xl mx-auto">
      <SectionHeader
        title="Backups"
        description="Create encrypted backups of browser profiles"
        action={
          <button
            onClick={loadBackups}
            disabled={backupsLoading}
            className="btn-secondary text-sm flex items-center gap-2"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${backupsLoading ? "animate-spin" : ""}`} />
            Refresh
          </button>
        }
      />

      {/* Create backup */}
      <div className="card p-5 mb-6">
        <h3 className="text-sm font-semibold text-white mb-4 flex items-center gap-2">
          <Plus className="w-4 h-4 text-brand-400" />
          Create New Backup
        </h3>

        <div className="grid grid-cols-2 gap-4 mb-4">
          {/* Browser selector */}
          <div>
            <label className="label">Browser</label>
            <select
              className="input"
              value={selectedBrowser?.kind ?? ""}
              onChange={(e) => {
                const b = browsers.find((br) => br.kind === e.target.value);
                setSelectedBrowser(b ?? null);
                setSelectedProfile(null);
              }}
            >
              <option value="">Select browser…</option>
              {browsers.map((b) => (
                <option key={b.kind} value={b.kind}>
                  {b.display_name}
                </option>
              ))}
            </select>
          </div>

          {/* Profile selector */}
          <div>
            <label className="label">Profile</label>
            <select
              className="input"
              value={selectedProfile?.id ?? ""}
              onChange={(e) => {
                const p = selectedBrowser?.profiles.find(
                  (pr) => pr.id === e.target.value
                );
                setSelectedProfile(p ?? null);
              }}
              disabled={!selectedBrowser}
            >
              <option value="">Select profile…</option>
              {selectedBrowser?.profiles.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.display_name}
                  {p.is_default ? " (Default)" : ""}
                </option>
              ))}
            </select>
          </div>
        </div>

        {/* Encryption toggle */}
        <label className="flex items-center gap-3 mb-4 cursor-pointer">
          <input
            type="checkbox"
            checked={encrypt}
            onChange={(e) => setEncrypt(e.target.checked)}
            className="accent-brand-500"
          />
          <span className="flex items-center gap-1.5 text-sm text-slate-300">
            {encrypt ? (
              <Lock className="w-4 h-4 text-green-400" />
            ) : (
              <Unlock className="w-4 h-4 text-amber-400" />
            )}
            {encrypt ? "AES-256-GCM encrypted backup" : "Unencrypted backup (not recommended)"}
          </span>
        </label>

        {encrypt && (
          <Alert type="info">
            The backup key is stored in your OS keychain. You will need the
            same machine to restore it.
          </Alert>
        )}

        <div className="mt-4">
          <button
            onClick={handleCreate}
            disabled={creating || !selectedProfile}
            className="btn-primary flex items-center gap-2"
          >
            {creating ? (
              <Spinner size="sm" />
            ) : (
              <Archive className="w-4 h-4" />
            )}
            {creating ? "Creating backup…" : "Choose Folder & Backup"}
          </button>
        </div>
      </div>

      {/* Existing backups */}
      <h3 className="text-xs font-semibold text-slate-500 uppercase tracking-widest mb-3">
        Saved Backups
      </h3>

      {backupsLoading ? (
        <div className="flex justify-center py-10">
          <Spinner />
        </div>
      ) : backups.length === 0 ? (
        <Empty
          icon={Archive}
          title="No backups yet"
          description="Create a backup above to protect your browser data."
        />
      ) : (
        <div className="space-y-2">
          {backups.map((backup) => (
            <div key={backup.id} className="card p-4 flex items-center gap-4">
              <div className="w-9 h-9 rounded-lg bg-surface-700 flex items-center justify-center shrink-0">
                {backup.encrypted ? (
                  <Lock className="w-4 h-4 text-green-400" />
                ) : (
                  <Archive className="w-4 h-4 text-slate-400" />
                )}
              </div>

              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium text-white capitalize">
                    {backup.browser}
                  </span>
                  <span className="text-slate-600 text-xs">·</span>
                  <span className="text-xs text-slate-400">{backup.profile}</span>
                  {backup.encrypted && (
                    <Badge variant="success">Encrypted</Badge>
                  )}
                </div>
                <p
                  className="text-xs text-slate-500 mt-0.5 truncate"
                  title={backup.path}
                >
                  {backup.path}
                </p>
              </div>

              <div className="text-right shrink-0">
                <p className="text-sm text-slate-300">{formatBytes(backup.size_bytes)}</p>
                <p
                  className="text-xs text-slate-500"
                  title={formatDateTime(backup.created_at)}
                >
                  {formatRelativeTime(backup.created_at)}
                </p>
              </div>

              <button
                onClick={() => deleteBackup(backup.id)}
                className="btn-ghost p-1.5 text-red-400 hover:text-red-300"
                title="Delete backup"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
