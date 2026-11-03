import { create } from "zustand";
import { immer } from "zustand/middleware/immer";
import type {
  AppSettings,
  BackupRecord,
  BrowserKind,
  BrowserProfile,
  DataCategory,
  InstalledBrowser,
  MigrationRecord,
  MigrationStatus,
} from "@/types";
import * as api from "@/services/tauri";

export type AppView =
  | "dashboard"
  | "migrate"
  | "history"
  | "backups"
  | "settings";

export type WizardStep =
  | "source"
  | "destination"
  | "categories"
  | "review"
  | "progress"
  | "report";

export interface MigrationWizardState {
  step: WizardStep;
  sourceBrowser?: BrowserKind;
  sourceProfile?: BrowserProfile;
  destBrowser?: BrowserKind;
  destProfile?: BrowserProfile;
  selectedCategories: DataCategory[];
  dryRun: boolean;
  activeRunId?: string;
  activeStatus?: MigrationStatus;
}

interface AppState {
  // Navigation
  view: AppView;

  // Browser data
  browsers: InstalledBrowser[];
  browsersLoading: boolean;
  browsersError?: string;

  // Consent
  consentGiven: boolean;

  // Migration wizard
  wizard: MigrationWizardState;

  // History
  migrationHistory: MigrationRecord[];
  historyLoading: boolean;

  // Backups
  backups: BackupRecord[];
  backupsLoading: boolean;

  // Settings
  settings: AppSettings;
  settingsLoading: boolean;

  // Notification
  notification?: { type: "success" | "error" | "info"; message: string };
}

interface AppActions {
  setView: (view: AppView) => void;

  // Browser
  loadBrowsers: () => Promise<void>;

  // Consent
  giveConsent: () => Promise<void>;

  // Wizard
  setWizardStep: (step: WizardStep) => void;
  setSourceBrowser: (browser: BrowserKind, profile: BrowserProfile) => void;
  setDestBrowser: (browser: BrowserKind, profile: BrowserProfile) => void;
  toggleCategory: (category: DataCategory) => void;
  selectAllCategories: (categories: DataCategory[]) => void;
  setDryRun: (v: boolean) => void;
  startMigration: () => Promise<void>;
  pollMigrationStatus: () => Promise<void>;
  cancelMigration: () => Promise<void>;
  resetWizard: () => void;

  // History
  loadHistory: () => Promise<void>;

  // Backups
  loadBackups: () => Promise<void>;
  deleteBackup: (id: string) => Promise<void>;

  // Settings
  loadSettings: () => Promise<void>;
  saveSettings: (s: AppSettings) => Promise<void>;

  // Notifications
  notify: (type: "success" | "error" | "info", message: string) => void;
  clearNotification: () => void;
}

const defaultWizard: MigrationWizardState = {
  step: "source",
  selectedCategories: ["bookmarks", "history", "extensions"],
  dryRun: false,
};

export const useAppStore = create<AppState & AppActions>()(
  immer((set, get) => ({
    // Initial state
    view: "dashboard",
    browsers: [],
    browsersLoading: false,
    consentGiven: false,
    wizard: { ...defaultWizard },
    migrationHistory: [],
    historyLoading: false,
    backups: [],
    backupsLoading: false,
    settings: {
      theme: "dark",
      auto_backup: true,
      backup_encryption: true,
      consent_given: false,
      max_history_days: 90,
    },
    settingsLoading: false,

    setView: (view) => set((s) => { s.view = view; }),

    // ── Browsers ──────────────────────────────────────────────────────────────
    loadBrowsers: async () => {
      set((s) => { s.browsersLoading = true; s.browsersError = undefined; });
      try {
        const browsers = await api.detectBrowsers();
        set((s) => { s.browsers = browsers; s.browsersLoading = false; });
      } catch (err) {
        set((s) => {
          s.browsersError = String(err);
          s.browsersLoading = false;
        });
      }
    },

    // ── Consent ───────────────────────────────────────────────────────────────
    giveConsent: async () => {
      const current = get().settings;
      const updated = { ...current, consent_given: true };
      await api.saveSettings(updated);
      set((s) => {
        s.consentGiven = true;
        s.settings = updated;
      });
    },

    // ── Wizard ────────────────────────────────────────────────────────────────
    setWizardStep: (step) => set((s) => { s.wizard.step = step; }),

    setSourceBrowser: (browser, profile) =>
      set((s) => {
        s.wizard.sourceBrowser = browser;
        s.wizard.sourceProfile = profile;
      }),

    setDestBrowser: (browser, profile) =>
      set((s) => {
        s.wizard.destBrowser = browser;
        s.wizard.destProfile = profile;
      }),

    toggleCategory: (category) =>
      set((s) => {
        const idx = s.wizard.selectedCategories.indexOf(category);
        if (idx >= 0) {
          s.wizard.selectedCategories.splice(idx, 1);
        } else {
          s.wizard.selectedCategories.push(category);
        }
      }),

    selectAllCategories: (categories) =>
      set((s) => { s.wizard.selectedCategories = [...categories]; }),

    setDryRun: (v) => set((s) => { s.wizard.dryRun = v; }),

    startMigration: async () => {
      const { wizard } = get();
      if (
        !wizard.sourceBrowser ||
        !wizard.sourceProfile ||
        !wizard.destBrowser ||
        !wizard.destProfile
      ) {
        get().notify("error", "Please select source and destination profiles");
        return;
      }

      try {
        const runId = await api.startMigration({
          source_browser: wizard.sourceBrowser,
          source_profile_path: wizard.sourceProfile.path,
          dest_browser: wizard.destBrowser,
          dest_profile_path: wizard.destProfile.path,
          data_categories: wizard.selectedCategories,
          dry_run: wizard.dryRun,
        });

        set((s) => {
          s.wizard.activeRunId = runId;
          s.wizard.step = "progress";
        });

        // Start polling
        get().pollMigrationStatus();
      } catch (err) {
        get().notify("error", `Migration failed to start: ${err}`);
      }
    },

    pollMigrationStatus: async () => {
      const poll = async () => {
        const { wizard } = get();
        if (!wizard.activeRunId) return;

        try {
          const status = await api.getMigrationStatus(wizard.activeRunId);
          if (!status) return;

          set((s) => { s.wizard.activeStatus = status; });

          if (
            status.status === "running" ||
            status.status === "pending"
          ) {
            setTimeout(poll, 500);
          } else {
            // Done
            set((s) => { s.wizard.step = "report"; });
            get().loadHistory();
          }
        } catch {
          setTimeout(poll, 1000);
        }
      };
      poll();
    },

    cancelMigration: async () => {
      const { wizard } = get();
      if (!wizard.activeRunId) return;
      await api.cancelMigration(wizard.activeRunId);
      set((s) => {
        if (s.wizard.activeStatus) {
          s.wizard.activeStatus.status = "cancelled";
        }
      });
    },

    resetWizard: () =>
      set((s) => {
        s.wizard = { ...defaultWizard };
        s.view = "dashboard";
      }),

    // ── History ───────────────────────────────────────────────────────────────
    loadHistory: async () => {
      set((s) => { s.historyLoading = true; });
      try {
        const history = await api.getMigrationHistory(50);
        set((s) => { s.migrationHistory = history; s.historyLoading = false; });
      } catch {
        set((s) => { s.historyLoading = false; });
      }
    },

    // ── Backups ───────────────────────────────────────────────────────────────
    loadBackups: async () => {
      set((s) => { s.backupsLoading = true; });
      try {
        const backups = await api.listBackups();
        set((s) => { s.backups = backups; s.backupsLoading = false; });
      } catch {
        set((s) => { s.backupsLoading = false; });
      }
    },

    deleteBackup: async (id) => {
      await api.deleteBackup(id);
      set((s) => {
        s.backups = s.backups.filter((b) => b.id !== id);
      });
      get().notify("success", "Backup deleted");
    },

    // ── Settings ──────────────────────────────────────────────────────────────
    loadSettings: async () => {
      set((s) => { s.settingsLoading = true; });
      try {
        const settings = await api.getSettings();
        set((s) => {
          s.settings = settings;
          s.consentGiven = settings.consent_given;
          s.settingsLoading = false;
        });
      } catch {
        set((s) => { s.settingsLoading = false; });
      }
    },

    saveSettings: async (settings) => {
      await api.saveSettings(settings);
      set((s) => { s.settings = settings; });
      get().notify("success", "Settings saved");
    },

    // ── Notifications ─────────────────────────────────────────────────────────
    notify: (type, message) => {
      set((s) => { s.notification = { type, message }; });
      setTimeout(() => get().clearNotification(), 4000);
    },

    clearNotification: () =>
      set((s) => { s.notification = undefined; }),
  }))
);
