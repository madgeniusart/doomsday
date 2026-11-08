// ─── Browser types ────────────────────────────────────────────────────────────

export type BrowserKind =
  | "chrome"
  | "firefox"
  | "edge"
  | "brave"
  | "opera"
  | "safari";

export const BROWSER_META: Record<
  BrowserKind,
  { name: string; color: string; bgColor: string; textColor: string }
> = {
  chrome: {
    name: "Google Chrome",
    color: "#4285F4",
    bgColor: "rgba(66,133,244,0.12)",
    textColor: "#60a5fa",
  },
  firefox: {
    name: "Mozilla Firefox",
    color: "#FF7139",
    bgColor: "rgba(255,113,57,0.12)",
    textColor: "#fb923c",
  },
  edge: {
    name: "Microsoft Edge",
    color: "#0078D4",
    bgColor: "rgba(0,120,212,0.12)",
    textColor: "#38bdf8",
  },
  brave: {
    name: "Brave",
    color: "#FB542B",
    bgColor: "rgba(251,84,43,0.12)",
    textColor: "#f87171",
  },
  opera: {
    name: "Opera",
    color: "#FF1B2D",
    bgColor: "rgba(255,27,45,0.12)",
    textColor: "#f87171",
  },
  safari: {
    name: "Safari",
    color: "#006CFF",
    bgColor: "rgba(0,108,255,0.12)",
    textColor: "#38bdf8",
  },
};

export interface BrowserProfile {
  id: string;
  display_name: string;
  path: string;
  avatar_icon?: string;
  is_default: boolean;
  email?: string;
  last_used?: string;
}

export interface InstalledBrowser {
  kind: BrowserKind;
  display_name: string;
  version?: string;
  executable_path?: string;
  profiles: BrowserProfile[];
  is_running: boolean;
}

// ─── Data categories ──────────────────────────────────────────────────────────

export type DataCategory =
  | "bookmarks"
  | "history"
  | "passwords"
  | "cookies"
  | "extensions"
  | "preferences"
  | "search_engines"
  | "permissions"
  | "local_storage";

export const CATEGORY_META: Record<
  DataCategory,
  {
    label: string;
    description: string;
    icon: string;
    risk: "safe" | "low" | "medium" | "high";
    riskNote?: string;
  }
> = {
  bookmarks: {
    label: "Bookmarks",
    description: "All saved bookmarks and folder structure",
    icon: "Bookmark",
    risk: "safe",
  },
  history: {
    label: "Browsing History",
    description: "Visited URLs and page titles",
    icon: "History",
    risk: "low",
    riskNote: "Exported as JSON reference",
  },
  cookies: {
    label: "Cookies",
    description: "Non-authentication cookies only",
    icon: "Cookie",
    risk: "medium",
    riskNote: "Auth cookies skipped — re-login required",
  },
  extensions: {
    label: "Extensions",
    description: "Installed extension list with compatibility info",
    icon: "Puzzle",
    risk: "low",
    riskNote: "Install listed extensions manually",
  },
  preferences: {
    label: "Preferences",
    description: "Browser settings and configuration",
    icon: "Settings",
    risk: "low",
    riskNote: "Reference export only",
  },
  passwords: {
    label: "Saved Passwords",
    description: "Password metadata (no plaintext values)",
    icon: "KeyRound",
    risk: "high",
    riskNote: "Metadata only — use a password manager",
  },
  search_engines: {
    label: "Search Engines",
    description: "Custom search engine configurations",
    icon: "Search",
    risk: "safe",
  },
  permissions: {
    label: "Site Permissions",
    description: "Notifications, location, camera permissions",
    icon: "ShieldCheck",
    risk: "low",
  },
  local_storage: {
    label: "Local Storage",
    description: "Website local storage and IndexedDB data",
    icon: "Database",
    risk: "medium",
    riskNote: "Limited browser API access",
  },
};

// ─── Bookmark / History / Cookie / Extension types ────────────────────────────

export interface Bookmark {
  id: string;
  title: string;
  url?: string;
  folder_path: string[];
  date_added?: number;
  date_modified?: number;
  children: Bookmark[];
}

export interface HistoryEntry {
  url: string;
  title?: string;
  visit_time: number;
  visit_count: number;
  typed_count?: number;
}

export interface CookieInfo {
  domain: string;
  name: string;
  path: string;
  secure: boolean;
  http_only: boolean;
  same_site: string;
  expires?: number;
  is_session: boolean;
  is_auth_likely: boolean;
}

export interface ExtensionInfo {
  id: string;
  name: string;
  version: string;
  description?: string;
  enabled: boolean;
  store_url?: string;
  permissions: string[];
}

export interface CompatibleExtension {
  extension: ExtensionInfo;
  compatible_store_url?: string;
  compatibility_note?: string;
}

// ─── Migration types ──────────────────────────────────────────────────────────

export type MigrationStatusKind =
  | "pending"
  | "running"
  | "completed"
  | "partial"
  | "failed"
  | "cancelled";

export interface MigrationItemResult {
  category: string;
  item_key: string;
  label: string;
  status: "migrated" | "exported" | "planned" | "failed" | "skipped" | "incompatible";
  message?: string;
}

export interface MigrationStatus {
  id: string;
  status: MigrationStatusKind;
  current_category?: string;
  progress: number;
  items_processed: number;
  items_total: number;
  items_failed: number;
  errors: string[];
  report: MigrationItemResult[];
  started_at: string;
  completed_at?: string;
}

export interface MigrationRequest {
  source_browser: BrowserKind;
  source_profile_path: string;
  dest_browser: BrowserKind;
  dest_profile_path: string;
  data_categories: DataCategory[];
  dry_run: boolean;
}

export interface MigrationRecord {
  id: string;
  source_browser: string;
  source_profile: string;
  dest_browser: string;
  dest_profile: string;
  data_types: string;
  status: string;
  started_at: string;
  completed_at?: string;
  items_migrated: number;
  items_failed: number;
  error_message?: string;
}

// ─── Backup types ─────────────────────────────────────────────────────────────

export interface BackupRecord {
  id: string;
  browser: string;
  profile: string;
  path: string;
  size_bytes: number;
  created_at: string;
  encrypted: boolean;
}

export interface BackupRequest {
  browser: string;
  profile_path: string;
  output_dir: string;
  encrypt: boolean;
}

// ─── Settings ─────────────────────────────────────────────────────────────────

export interface AppSettings {
  theme: string;
  auto_backup: boolean;
  backup_encryption: boolean;
  consent_given: boolean;
  max_history_days: number;
}

// ─── Compatibility ────────────────────────────────────────────────────────────

export type RiskLevel = "safe" | "low" | "medium" | "high";

export interface CategoryCompatibility {
  category: DataCategory;
  supported: boolean;
  notes?: string;
  risk_level: RiskLevel;
}

export interface CompatibilityReport {
  source_browser: BrowserKind;
  dest_browser: BrowserKind;
  categories: CategoryCompatibility[];
  warnings: string[];
  sites_requiring_relogin: string[];
}
