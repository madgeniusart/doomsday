import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  BackupRecord,
  BackupRequest,
  Bookmark,
  BrowserKind,
  BrowserProfile,
  CookieInfo,
  ExtensionInfo,
  CompatibleExtension,
  HistoryEntry,
  InstalledBrowser,
  MigrationItemResult,
  MigrationRecord,
  MigrationRequest,
  MigrationStatus,
} from "@/types";

export type { Bookmark };

// ─── Browser detection ────────────────────────────────────────────────────────

export const detectBrowsers = (): Promise<InstalledBrowser[]> =>
  invoke("detect_browsers");

export const getBrowserProfiles = (
  browser: BrowserKind
): Promise<BrowserProfile[]> => invoke("get_browser_profiles", { browser });

export const verifyProfileAccess = (profilePath: string): Promise<boolean> =>
  invoke("verify_profile_access", { profilePath });

// ─── Bookmarks ────────────────────────────────────────────────────────────────

export const readBookmarks = (
  browser: BrowserKind,
  profilePath: string
): Promise<Bookmark[]> => invoke("read_bookmarks", { browser, profilePath });

export const exportBookmarks = (
  browser: BrowserKind,
  profilePath: string,
  outputPath: string,
  format: "html" | "json" = "html"
): Promise<string> =>
  invoke("export_bookmarks", { browser, profilePath, outputPath, format });

// ─── History ──────────────────────────────────────────────────────────────────

export const readHistory = (
  browser: BrowserKind,
  profilePath: string,
  maxDays?: number
): Promise<HistoryEntry[]> =>
  invoke("read_history", { browser, profilePath, maxDays });

// ─── Cookies ─────────────────────────────────────────────────────────────────

export const readCookies = (
  browser: BrowserKind,
  profilePath: string
): Promise<CookieInfo[]> => invoke("read_cookies", { browser, profilePath });

export const exportCookies = (
  browser: BrowserKind,
  profilePath: string,
  outputPath: string,
  includeAuth: boolean = false
): Promise<string> =>
  invoke("export_cookies", {
    browser,
    profilePath,
    outputPath,
    includeAuth,
  });

// ─── Extensions ───────────────────────────────────────────────────────────────

export const readExtensions = (
  browser: BrowserKind,
  profilePath: string
): Promise<ExtensionInfo[]> =>
  invoke("read_extensions", { browser, profilePath });

export const getCompatibleExtensions = (
  sourceBrowser: BrowserKind,
  destBrowser: BrowserKind,
  extensions: ExtensionInfo[]
): Promise<CompatibleExtension[]> =>
  invoke("get_compatible_extensions", {
    sourceBrowser,
    destBrowser,
    extensions,
  });

// ─── Migration ────────────────────────────────────────────────────────────────

export const startMigration = (
  request: MigrationRequest
): Promise<string> => invoke("start_migration", { request });

export const getMigrationStatus = (
  runId: string
): Promise<MigrationStatus | null> =>
  invoke("get_migration_status", { runId });

export const cancelMigration = (runId: string): Promise<void> =>
  invoke("cancel_migration", { runId });

export const getMigrationHistory = (
  limit?: number
): Promise<MigrationRecord[]> =>
  invoke("get_migration_history", { limit });

export const getMigrationReport = (
  runId: string
): Promise<MigrationItemResult[] | null> =>
  invoke("get_migration_report", { runId });

// ─── Backup ───────────────────────────────────────────────────────────────────

export const createBackup = (
  request: BackupRequest
): Promise<BackupRecord> => invoke("create_backup", { request });

export const restoreBackup = (
  backupPath: string,
  outputDir: string,
  encrypted: boolean
): Promise<string> =>
  invoke("restore_backup", { backupPath, outputDir, encrypted });

export const listBackups = (): Promise<BackupRecord[]> =>
  invoke("list_backups");

export const deleteBackup = (backupId: string): Promise<void> =>
  invoke("delete_backup", { backupId });

// ─── Settings ─────────────────────────────────────────────────────────────────

export const getSettings = (): Promise<AppSettings> =>
  invoke("get_settings");

export const saveSettings = (settings: AppSettings): Promise<void> =>
  invoke("save_settings", { settings });

// ─── Secure credential import / companion bridge ────────────────────────────
export interface PasswordImportResult {
  imported: number;
  skipped: number;
  encrypted_file: string;
}

export const importPasswordCsv = (csvPath: string): Promise<PasswordImportResult> =>
  invoke("import_password_csv", { csvPath });

export const listPasswordImports = (): Promise<Array<{ file: string; created_at: string; encrypted: boolean }>> =>
  invoke("list_password_imports");

export const drainExtensionEvents = (): Promise<Array<{ received_at: string; event: Record<string, unknown> }>> =>
  invoke("drain_extension_events");

export const exportPasswordCsv = (outputPath: string): Promise<number> =>
  invoke("export_password_csv", { outputPath });
