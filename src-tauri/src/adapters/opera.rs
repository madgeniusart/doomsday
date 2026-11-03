use super::{
    BrowserAdapter, BrowserKind, BrowserProfile, Bookmark,
    CookieInfo, DataCategory, ExtensionInfo, HistoryEntry,
    chromium_base::*,
};
use anyhow::Result;
use std::path::PathBuf;

pub struct OperaAdapter {
    user_data_dir: PathBuf,
}

impl OperaAdapter {
    pub fn new() -> Self {
        Self { user_data_dir: opera_user_data_dir() }
    }
}

impl BrowserAdapter for OperaAdapter {
    fn kind(&self) -> BrowserKind { BrowserKind::Opera }

    fn enumerate_profiles(&self) -> Result<Vec<BrowserProfile>> {
        // Opera uses a flat profile structure (no multi-profile by default)
        if !self.user_data_dir.exists() {
            return Ok(vec![]);
        }
        // Opera stores data directly in user_data_dir without "Default" subdir
        let profile_path = self.user_data_dir.clone();
        if profile_path.join("Bookmarks").exists() || profile_path.join("History").exists() {
            Ok(vec![super::BrowserProfile {
                id: "opera_default".to_string(),
                display_name: "Opera".to_string(),
                path: profile_path,
                avatar_icon: None,
                is_default: true,
                email: None,
                last_used: None,
            }])
        } else {
            // Also try nested Default pattern
            enumerate_chromium_profiles(&self.user_data_dir)
        }
    }
    fn read_bookmarks(&self, p: &PathBuf) -> Result<Vec<Bookmark>> {
        read_chromium_bookmarks(p)
    }
    fn read_history(&self, p: &PathBuf, d: u32) -> Result<Vec<HistoryEntry>> {
        read_chromium_history(p, d)
    }
    fn read_cookies(&self, p: &PathBuf) -> Result<Vec<CookieInfo>> {
        read_chromium_cookies(p)
    }
    fn read_extensions(&self, p: &PathBuf) -> Result<Vec<ExtensionInfo>> {
        read_chromium_extensions(p)
    }
    fn supported_categories(&self) -> Vec<DataCategory> {
        chromium_supported_categories()
    }
}

fn opera_user_data_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(&home).join(".config/opera")
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(&home).join("Library/Application Support/com.operasoftware.Opera")
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        PathBuf::from(&appdata).join("Opera Software\\Opera Stable")
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    PathBuf::new()
}
