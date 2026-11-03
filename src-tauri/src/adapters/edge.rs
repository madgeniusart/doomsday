use super::{
    BrowserAdapter, BrowserKind, BrowserProfile, Bookmark,
    CookieInfo, DataCategory, ExtensionInfo, HistoryEntry,
    chromium_base::*,
};
use anyhow::Result;
use std::path::PathBuf;

pub struct EdgeAdapter {
    user_data_dir: PathBuf,
}

impl EdgeAdapter {
    pub fn new() -> Self {
        Self { user_data_dir: edge_user_data_dir() }
    }
}

impl BrowserAdapter for EdgeAdapter {
    fn kind(&self) -> BrowserKind { BrowserKind::Edge }

    fn enumerate_profiles(&self) -> Result<Vec<BrowserProfile>> {
        enumerate_chromium_profiles(&self.user_data_dir)
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

fn edge_user_data_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(&home).join(".config/microsoft-edge")
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(&home).join("Library/Application Support/Microsoft Edge")
    }
    #[cfg(target_os = "windows")]
    {
        let local = std::env::var("LOCALAPPDATA").unwrap_or_default();
        PathBuf::from(&local).join("Microsoft\\Edge\\User Data")
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    PathBuf::new()
}
