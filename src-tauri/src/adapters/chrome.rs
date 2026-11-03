use super::{
    BrowserAdapter, BrowserKind, BrowserProfile, Bookmark,
    CookieInfo, DataCategory, ExtensionInfo, HistoryEntry,
    chromium_base::*,
};
use anyhow::Result;
use std::path::PathBuf;

pub struct ChromeAdapter {
    user_data_dir: PathBuf,
}

impl ChromeAdapter {
    pub fn new() -> Self {
        Self {
            user_data_dir: chrome_user_data_dir(),
        }
    }
}

impl BrowserAdapter for ChromeAdapter {
    fn kind(&self) -> BrowserKind {
        BrowserKind::Chrome
    }

    fn enumerate_profiles(&self) -> Result<Vec<BrowserProfile>> {
        enumerate_chromium_profiles(&self.user_data_dir)
    }

    fn read_bookmarks(&self, profile_path: &PathBuf) -> Result<Vec<Bookmark>> {
        read_chromium_bookmarks(profile_path)
    }

    fn read_history(&self, profile_path: &PathBuf, max_days: u32) -> Result<Vec<HistoryEntry>> {
        read_chromium_history(profile_path, max_days)
    }

    fn read_cookies(&self, profile_path: &PathBuf) -> Result<Vec<CookieInfo>> {
        read_chromium_cookies(profile_path)
    }

    fn read_extensions(&self, profile_path: &PathBuf) -> Result<Vec<ExtensionInfo>> {
        read_chromium_extensions(profile_path)
    }

    fn supported_categories(&self) -> Vec<DataCategory> {
        chromium_supported_categories()
    }
}

fn chrome_user_data_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(&home).join(".config/google-chrome")
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(&home).join("Library/Application Support/Google/Chrome")
    }
    #[cfg(target_os = "windows")]
    {
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
        PathBuf::from(&local_app_data).join("Google\\Chrome\\User Data")
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    PathBuf::new()
}
