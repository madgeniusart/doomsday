use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod brave;
pub mod chrome;
pub mod chromium_base;
pub mod edge;
pub mod firefox;
pub mod opera;
pub mod safari;
pub mod detector;

pub use detector::BrowserDetector;

// ─── Core types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum BrowserKind {
    Chrome,
    Firefox,
    Edge,
    Brave,
    Opera,
    Safari,
}

impl BrowserKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            BrowserKind::Chrome => "Google Chrome",
            BrowserKind::Firefox => "Mozilla Firefox",
            BrowserKind::Edge => "Microsoft Edge",
            BrowserKind::Brave => "Brave",
            BrowserKind::Opera => "Opera",
            BrowserKind::Safari => "Safari",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            BrowserKind::Chrome => "chrome",
            BrowserKind::Firefox => "firefox",
            BrowserKind::Edge => "edge",
            BrowserKind::Brave => "brave",
            BrowserKind::Opera => "opera",
            BrowserKind::Safari => "safari",
        }
    }

    /// The lowercase identifier used wherever BrowserKind is stored or
    /// displayed as plain text (database rows, history records, etc).
    /// Identical to `icon_name()` and to this type's serde representation
    /// (`#[serde(rename_all = "lowercase")]`) — kept as a distinct method
    /// so call sites read as "give me the canonical string id" rather than
    /// implying anything about icons.
    pub fn as_str(&self) -> &'static str {
        self.icon_name()
    }

    /// Whether this browser uses the Chromium profile format
    pub fn is_chromium_based(&self) -> bool {
        matches!(
            self,
            BrowserKind::Chrome | BrowserKind::Edge | BrowserKind::Brave | BrowserKind::Opera
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledBrowser {
    pub kind: BrowserKind,
    pub display_name: String,
    pub version: Option<String>,
    pub executable_path: Option<PathBuf>,
    pub profiles: Vec<BrowserProfile>,
    pub is_running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserProfile {
    pub id: String,
    pub display_name: String,
    pub path: PathBuf,
    pub avatar_icon: Option<String>,
    pub is_default: bool,
    pub email: Option<String>,
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DataCategory {
    Bookmarks,
    History,
    Passwords,    // read-only metadata only; never plaintext
    Cookies,
    Extensions,
    Preferences,
    SearchEngines,
    Permissions,
    LocalStorage,
}

impl DataCategory {
    pub fn label(&self) -> &'static str {
        match self {
            DataCategory::Bookmarks => "Bookmarks",
            DataCategory::History => "Browsing History",
            DataCategory::Passwords => "Saved Passwords (metadata)",
            DataCategory::Cookies => "Cookies",
            DataCategory::Extensions => "Extensions",
            DataCategory::Preferences => "Browser Preferences",
            DataCategory::SearchEngines => "Search Engines",
            DataCategory::Permissions => "Site Permissions",
            DataCategory::LocalStorage => "Local Storage / IndexedDB",
        }
    }
}

// ─── Bookmark types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub title: String,
    pub url: Option<String>,
    pub folder_path: Vec<String>,
    pub date_added: Option<i64>,
    pub date_modified: Option<i64>,
    pub children: Vec<Bookmark>,
}

// ─── History types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub url: String,
    pub title: Option<String>,
    pub visit_time: i64, // Unix timestamp microseconds
    pub visit_count: i64,
    pub typed_count: Option<i64>,
}

// ─── Cookie types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieInfo {
    pub domain: String,
    pub name: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: String,
    pub expires: Option<i64>, // Unix timestamp; None = session
    pub is_session: bool,
    pub is_auth_likely: bool, // heuristic flag
}

// ─── Extension types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub store_url: Option<String>,
    pub permissions: Vec<String>,
}

// ─── Adapter trait ────────────────────────────────────────────────────────────

pub trait BrowserAdapter: Send + Sync {
    fn kind(&self) -> BrowserKind;

    /// Enumerate profiles for this browser installation
    fn enumerate_profiles(&self) -> Result<Vec<BrowserProfile>>;

    /// Read bookmarks from a profile directory
    fn read_bookmarks(&self, profile_path: &PathBuf) -> Result<Vec<Bookmark>>;

    /// Read browsing history (capped by days)
    fn read_history(&self, profile_path: &PathBuf, max_days: u32) -> Result<Vec<HistoryEntry>>;

    /// Read cookie metadata (not values for auth cookies)
    fn read_cookies(&self, profile_path: &PathBuf) -> Result<Vec<CookieInfo>>;

    /// List installed extensions
    fn read_extensions(&self, profile_path: &PathBuf) -> Result<Vec<ExtensionInfo>>;

    /// Data categories supported by this adapter
    fn supported_categories(&self) -> Vec<DataCategory>;

    /// Whether writing to this browser's profile is supported
    fn supports_write(&self) -> bool {
        false
    }
}
