import { useEffect, useState } from "react";
import { Save, Moon, Sun, Shield, Clock, Database } from "lucide-react";
import { useAppStore } from "@/store/appStore";
import { Spinner, SectionHeader, Alert } from "@/components/ui/index";
import type { AppSettings } from "@/types";

export function SettingsPanel() {
  const { settings, settingsLoading, loadSettings, saveSettings } = useAppStore(
    (s) => ({
      settings: s.settings,
      settingsLoading: s.settingsLoading,
      loadSettings: s.loadSettings,
      saveSettings: s.saveSettings,
    })
  );

  const [local, setLocal] = useState<AppSettings>(settings);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadSettings();
  }, []);

  useEffect(() => {
    setLocal(settings);
  }, [settings]);

  const handleSave = async () => {
    setSaving(true);
    try {
      await saveSettings(local);
    } finally {
      setSaving(false);
    }
  };

  if (settingsLoading) {
    return (
      <div className="flex justify-center py-20">
        <Spinner size="lg" />
      </div>
    );
  }

  return (
    <div className="p-8 max-w-2xl mx-auto">
      <SectionHeader
        title="Settings"
        description="Application preferences and data handling options"
      />

      <div className="space-y-4">
        {/* Appearance */}
        <div className="card p-5">
          <h3 className="text-sm font-semibold text-white mb-4 flex items-center gap-2">
            <Moon className="w-4 h-4 text-brand-400" />
            Appearance
          </h3>
          <div>
            <label className="label">Theme</label>
            <div className="flex gap-2">
              {["dark", "light"].map((t) => (
                <button
                  key={t}
                  onClick={() => setLocal((s) => ({ ...s, theme: t }))}
                  className={`flex items-center gap-2 px-4 py-2 rounded-lg border text-sm transition-all ${
                    local.theme === t
                      ? "bg-brand-600/20 border-brand-500/30 text-brand-300"
                      : "border-white/8 text-slate-400 hover:border-white/16"
                  }`}
                >
                  {t === "dark" ? (
                    <Moon className="w-3.5 h-3.5" />
                  ) : (
                    <Sun className="w-3.5 h-3.5" />
                  )}
                  {t.charAt(0).toUpperCase() + t.slice(1)}
                </button>
              ))}
            </div>
          </div>
        </div>

        {/* Security */}
        <div className="card p-5">
          <h3 className="text-sm font-semibold text-white mb-4 flex items-center gap-2">
            <Shield className="w-4 h-4 text-brand-400" />
            Security
          </h3>
          <div className="space-y-3">
            <label className="flex items-start justify-between gap-4 cursor-pointer">
              <div>
                <p className="text-sm text-slate-200">Encrypt backups</p>
                <p className="text-xs text-slate-500 mt-0.5">
                  Use AES-256-GCM encryption for all profile backups. Key stored in OS keychain.
                </p>
              </div>
              <input
                type="checkbox"
                checked={local.backup_encryption}
                onChange={(e) =>
                  setLocal((s) => ({ ...s, backup_encryption: e.target.checked }))
                }
                className="mt-1 accent-brand-500 shrink-0"
              />
            </label>

            <div className="divider" />

            <label className="flex items-start justify-between gap-4 cursor-pointer">
              <div>
                <p className="text-sm text-slate-200">Auto-backup before migration</p>
                <p className="text-xs text-slate-500 mt-0.5">
                  Automatically create a backup of the destination profile before
                  writing any migration data.
                </p>
              </div>
              <input
                type="checkbox"
                checked={local.auto_backup}
                onChange={(e) =>
                  setLocal((s) => ({ ...s, auto_backup: e.target.checked }))
                }
                className="mt-1 accent-brand-500 shrink-0"
              />
            </label>
          </div>
        </div>

        {/* History */}
        <div className="card p-5">
          <h3 className="text-sm font-semibold text-white mb-4 flex items-center gap-2">
            <Clock className="w-4 h-4 text-brand-400" />
            History Export
          </h3>
          <div>
            <label className="label">Max history days to export</label>
            <div className="flex items-center gap-3">
              <input
                type="range"
                min={7}
                max={365}
                step={7}
                value={local.max_history_days}
                onChange={(e) =>
                  setLocal((s) => ({
                    ...s,
                    max_history_days: Number(e.target.value),
                  }))
                }
                className="flex-1 accent-brand-500"
              />
              <span className="text-sm text-slate-300 tabular-nums w-16 text-right">
                {local.max_history_days} days
              </span>
            </div>
            <p className="text-xs text-slate-500 mt-1">
              Only history entries within this window will be exported.
            </p>
          </div>
        </div>

        {/* Data */}
        <div className="card p-5">
          <h3 className="text-sm font-semibold text-white mb-4 flex items-center gap-2">
            <Database className="w-4 h-4 text-brand-400" />
            Privacy
          </h3>
          <Alert type="info">
            Universal Browser Sync is entirely local-first. No data is ever sent to
            external servers. All processing, encryption, and storage happens on
            your device.
          </Alert>
        </div>

        {/* Save */}
        <div className="flex justify-end">
          <button
            onClick={handleSave}
            disabled={saving}
            className="btn-primary flex items-center gap-2"
          >
            {saving ? <Spinner size="sm" /> : <Save className="w-4 h-4" />}
            {saving ? "Saving…" : "Save Settings"}
          </button>
        </div>
      </div>
    </div>
  );
}
