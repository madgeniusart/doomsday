import { describe, it, expect, vi, beforeEach } from "vitest";
import { useAppStore } from "@/store/appStore";
import { invoke } from "@tauri-apps/api/core";
import type { InstalledBrowser, AppSettings } from "@/types";

const mockInvoke = vi.mocked(invoke);

const MOCK_BROWSERS: InstalledBrowser[] = [
  {
    kind: "chrome",
    display_name: "Google Chrome",
    version: "121.0.0.0",
    profiles: [
      {
        id: "profile-1",
        display_name: "Default",
        path: "/home/user/.config/google-chrome/Default",
        is_default: true,
      },
      {
        id: "profile-2",
        display_name: "Work",
        path: "/home/user/.config/google-chrome/Profile 1",
        is_default: false,
        email: "work@example.com",
      },
    ],
    is_running: false,
  },
  {
    kind: "firefox",
    display_name: "Mozilla Firefox",
    profiles: [
      {
        id: "ff-profile-1",
        display_name: "default-release",
        path: "/home/user/.mozilla/firefox/abc123.default-release",
        is_default: true,
      },
    ],
    is_running: true,
  },
];

const MOCK_SETTINGS: AppSettings = {
  theme: "dark",
  auto_backup: true,
  backup_encryption: true,
  consent_given: true,
  max_history_days: 90,
};

beforeEach(() => {
  // Reset store to initial state
  useAppStore.setState({
    view: "dashboard",
    browsers: [],
    browsersLoading: false,
    consentGiven: false,
    wizard: {
      step: "source",
      selectedCategories: ["bookmarks", "history", "extensions"],
      dryRun: false,
    },
    migrationHistory: [],
    historyLoading: false,
    backups: [],
    backupsLoading: false,
    settings: MOCK_SETTINGS,
    settingsLoading: false,
    notification: undefined,
  });
  vi.clearAllMocks();
});

describe("Navigation", () => {
  it("changes view", () => {
    const { setView } = useAppStore.getState();
    setView("migrate");
    expect(useAppStore.getState().view).toBe("migrate");
  });

  it("cycles through all views", () => {
    const { setView } = useAppStore.getState();
    const views = ["dashboard", "migrate", "history", "backups", "settings"] as const;
    for (const v of views) {
      setView(v);
      expect(useAppStore.getState().view).toBe(v);
    }
  });
});

describe("Browser loading", () => {
  it("sets loading state while fetching", async () => {
    mockInvoke.mockResolvedValueOnce(MOCK_BROWSERS);
    const promise = useAppStore.getState().loadBrowsers();
    expect(useAppStore.getState().browsersLoading).toBe(true);
    await promise;
    expect(useAppStore.getState().browsersLoading).toBe(false);
    expect(useAppStore.getState().browsers).toHaveLength(2);
  });

  it("sets error on failure", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("Permission denied"));
    await useAppStore.getState().loadBrowsers();
    expect(useAppStore.getState().browsersError).toMatch(/Permission denied/);
    expect(useAppStore.getState().browsersLoading).toBe(false);
  });

  it("stores browsers from API", async () => {
    mockInvoke.mockResolvedValueOnce(MOCK_BROWSERS);
    await useAppStore.getState().loadBrowsers();
    const { browsers } = useAppStore.getState();
    expect(browsers[0].kind).toBe("chrome");
    expect(browsers[0].profiles).toHaveLength(2);
    expect(browsers[1].is_running).toBe(true);
  });
});

describe("Migration wizard", () => {
  it("advances wizard step", () => {
    const { setWizardStep } = useAppStore.getState();
    setWizardStep("destination");
    expect(useAppStore.getState().wizard.step).toBe("destination");
  });

  it("sets source browser and profile", () => {
    const { setSourceBrowser } = useAppStore.getState();
    setSourceBrowser("chrome", MOCK_BROWSERS[0].profiles[0]);
    const { wizard } = useAppStore.getState();
    expect(wizard.sourceBrowser).toBe("chrome");
    expect(wizard.sourceProfile?.id).toBe("profile-1");
  });

  it("toggles categories", () => {
    const store = useAppStore.getState();
    const initial = [...store.wizard.selectedCategories];
    store.toggleCategory("cookies");
    expect(useAppStore.getState().wizard.selectedCategories).toContain("cookies");
    store.toggleCategory("cookies");
    expect(useAppStore.getState().wizard.selectedCategories).not.toContain("cookies");
    // Toggling the same category on then off should return to the exact
    // original selection, not just "no longer contains it".
    expect(useAppStore.getState().wizard.selectedCategories).toEqual(initial);
  });

  it("select all categories", () => {
    const { selectAllCategories } = useAppStore.getState();
    selectAllCategories(["bookmarks", "history", "cookies", "extensions"]);
    expect(useAppStore.getState().wizard.selectedCategories).toHaveLength(4);
  });

  it("dry run toggle", () => {
    const { setDryRun } = useAppStore.getState();
    setDryRun(true);
    expect(useAppStore.getState().wizard.dryRun).toBe(true);
    setDryRun(false);
    expect(useAppStore.getState().wizard.dryRun).toBe(false);
  });

  it("resets wizard to initial state", () => {
    const store = useAppStore.getState();
    store.setSourceBrowser("chrome", MOCK_BROWSERS[0].profiles[0]);
    store.setWizardStep("review");
    store.resetWizard();

    const { wizard, view } = useAppStore.getState();
    expect(wizard.step).toBe("source");
    expect(wizard.sourceBrowser).toBeUndefined();
    expect(view).toBe("dashboard");
  });

  it("refuses to start migration without profiles selected", async () => {
    const store = useAppStore.getState();
    // No source/dest set
    await store.startMigration();
    expect(useAppStore.getState().notification?.type).toBe("error");
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("starts migration and transitions to progress step", async () => {
    const runId = "test-run-123";
    mockInvoke.mockResolvedValueOnce(runId); // start_migration
    mockInvoke.mockResolvedValue(null); // get_migration_status poll

    const store = useAppStore.getState();
    store.setSourceBrowser("chrome", MOCK_BROWSERS[0].profiles[0]);
    store.setDestBrowser("firefox", MOCK_BROWSERS[1].profiles[0]);
    await store.startMigration();

    const { wizard } = useAppStore.getState();
    expect(wizard.step).toBe("progress");
    expect(wizard.activeRunId).toBe(runId);
  });
});

describe("Consent", () => {
  it("gives consent and saves settings", async () => {
    mockInvoke.mockResolvedValueOnce(undefined); // save_settings
    await useAppStore.getState().giveConsent();
    expect(useAppStore.getState().consentGiven).toBe(true);
    expect(useAppStore.getState().settings.consent_given).toBe(true);
  });
});

describe("Notifications", () => {
  it("shows and auto-clears notification", async () => {
    vi.useFakeTimers();
    const { notify } = useAppStore.getState();
    notify("success", "Done!");
    expect(useAppStore.getState().notification).toEqual({
      type: "success",
      message: "Done!",
    });
    vi.advanceTimersByTime(4001);
    expect(useAppStore.getState().notification).toBeUndefined();
    vi.useRealTimers();
  });

  it("manually clears notification", () => {
    const store = useAppStore.getState();
    store.notify("error", "Oops");
    store.clearNotification();
    expect(useAppStore.getState().notification).toBeUndefined();
  });
});

describe("Settings", () => {
  it("loads settings from API", async () => {
    mockInvoke.mockResolvedValueOnce(MOCK_SETTINGS);
    await useAppStore.getState().loadSettings();
    expect(useAppStore.getState().settings).toEqual(MOCK_SETTINGS);
    expect(useAppStore.getState().consentGiven).toBe(true);
  });

  it("saves settings", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);
    const newSettings = { ...MOCK_SETTINGS, theme: "light" };
    await useAppStore.getState().saveSettings(newSettings);
    expect(useAppStore.getState().settings.theme).toBe("light");
    expect(useAppStore.getState().notification?.type).toBe("success");
  });
});
