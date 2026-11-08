//! Safari adapter â€” macOS only.
//! Access is limited by System Integrity Protection and Full Disk Access requirement.
//! Users must grant Full Disk Access to the app in System Preferences.

use super::{
    BrowserAdapter, BrowserKind, BrowserProfile, Bookmark,
    CookieInfo, DataCategory, ExtensionInfo, HistoryEntry,
};
use anyhow::{bail, Result};
use std::path::PathBuf;

#[cfg(target_os = "macos")]
use uuid::Uuid;

pub struct SafariAdapter;
impl SafariAdapter {
    pub fn new() -> Self { SafariAdapter }
}

impl BrowserAdapter for SafariAdapter {
    fn kind(&self) -> BrowserKind { BrowserKind::Safari }

    fn enumerate_profiles(&self) -> Result<Vec<BrowserProfile>> {
        #[cfg(target_os = "macos")]
        {
            let safari_dir = safari_data_dir();
            if safari_dir.exists() {
                return Ok(vec![BrowserProfile {
                    id: Uuid::new_v4().to_string(),
                    display_name: "Safari".to_string(),
                    path: safari_dir,
                    avatar_icon: None,
                    is_default: true,
                    email: None,
                    last_used: None,
                }]);
            }
        }
        Ok(vec![])
    }

    fn read_bookmarks(&self, _profile_path: &PathBuf) -> Result<Vec<Bookmark>> {
        #[cfg(target_os = "macos")]
        {
            // Safari bookmarks are stored as a plist (usually binary format).
            // The `plist` crate reads both binary and XML plists directly â€”
            // no need to shell out to `plutil` first.
            let bookmarks_plist = _profile_path.join("Bookmarks.plist");
            if !bookmarks_plist.exists() {
                bail!("Safari Bookmarks.plist not found. Full Disk Access may be required.");
            }

            let root = plist::Value::from_file(&bookmarks_plist)
                .map_err(|e| anyhow::anyhow!("Failed to parse Bookmarks.plist: {e}. Full Disk Access may be required."))?;

            let mut bookmarks = Vec::new();
            parse_safari_bookmark_node(&root, &[], &mut bookmarks);
            Ok(bookmarks)
        }
        #[cfg(not(target_os = "macos"))]
        {
            bail!("Safari is only supported on macOS");
        }
    }

    fn read_history(&self, _profile_path: &PathBuf, _max_days: u32) -> Result<Vec<HistoryEntry>> {
        #[cfg(target_os = "macos")]
        {
            use rusqlite::{Connection, OpenFlags, params};
            let history_db = _profile_path.join("History.db");
            if !history_db.exists() {
                bail!("Safari History.db not found. Full Disk Access required.");
            }

            let tmp = std::env::temp_dir()
                .join(format!("ubs_safari_{}", Uuid::new_v4()));
            std::fs::copy(&history_db, &tmp)?;

            let conn = Connection::open_with_flags(&tmp, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

            let cutoff = chrono::Utc::now()
                - chrono::Duration::try_days(_max_days as i64).unwrap_or_default();
            // Safari uses CoreData timestamps: seconds since 2001-01-01
            let apple_epoch_offset: i64 = 978_307_200;
            let cutoff_apple = cutoff.timestamp() - apple_epoch_offset;

            let mut stmt = conn.prepare(
                "SELECT hi.url, hi.title, hv.visit_time
                 FROM history_visits hv
                 JOIN history_items hi ON hv.history_item = hi.id
                 WHERE hv.visit_time > ?1
                 ORDER BY hv.visit_time DESC
                 LIMIT 50000",
            )?;

            let entries: Vec<HistoryEntry> = stmt
                .query_map(params![cutoff_apple as f64], |row| {
                    let visit_apple: f64 = row.get(2)?;
                    Ok(HistoryEntry {
                        url: row.get(0)?,
                        title: row.get(1)?,
                        visit_time: (visit_apple as i64 + apple_epoch_offset) * 1_000_000,
                        visit_count: 1,
                        typed_count: None,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();

            let _ = std::fs::remove_file(&tmp);
            Ok(entries)
        }
        #[cfg(not(target_os = "macos"))]
        bail!("Safari is only supported on macOS")
    }
    fn read_cookies(&self, _profile_path: &PathBuf) -> Result<Vec<CookieInfo>> {
        bail!("Safari cookie access requires Keychain entitlement and Full Disk Access. Not supported in this version.")
    }

    fn read_extensions(&self, _profile_path: &PathBuf) -> Result<Vec<ExtensionInfo>> {
        // Safari extensions are distributed via Mac App Store; no direct file access
        Ok(vec![])
    }

    fn supported_categories(&self) -> Vec<DataCategory> {
        vec![
            DataCategory::Bookmarks,
            DataCategory::History,
        ]
    }
}

#[cfg(target_os = "macos")]
fn safari_data_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(&home).join("Library/Safari")
}

/// Recursively walk Safari's Bookmarks.plist structure.
///
/// Safari's bookmark tree is a nested dictionary. Each node has a
/// `WebBookmarkType` of either:
///   - `WebBookmarkTypeLeaf`  â€” an individual bookmark (`URLString` + title)
///   - `WebBookmarkTypeList`  â€” a folder (`Title` + `Children` array)
///   - `WebBookmarkTypeProxy` â€” a dynamic/smart folder (e.g. Reading List),
///     which has no real children to migrate and is skipped.
#[cfg(target_os = "macos")]
fn parse_safari_bookmark_node(node: &plist::Value, folder_path: &[String], out: &mut Vec<Bookmark>) {
    let Some(dict) = node.as_dictionary() else { return };

    let bookmark_type = dict
        .get("WebBookmarkType")
        .and_then(|v| v.as_string())
        .unwrap_or("");

    match bookmark_type {
        "WebBookmarkTypeLeaf" => {
            let url = dict.get("URLString").and_then(|v| v.as_string());
            if let Some(url) = url {
                // Title is usually nested under URIDictionary.title; some
                // entries (bookmarklets, older Safari versions) store it
                // directly as a top-level "title" key instead.
                let title = dict
                    .get("URIDictionary")
                    .and_then(|v| v.as_dictionary())
                    .and_then(|d| d.get("title"))
                    .and_then(|v| v.as_string())
                    .or_else(|| dict.get("title").and_then(|v| v.as_string()))
                    .unwrap_or(url)
                    .to_string();

                out.push(Bookmark {
                    id: Uuid::new_v4().to_string(),
                    title,
                    url: Some(url.to_string()),
                    folder_path: folder_path.to_vec(),
                    date_added: None,
                    date_modified: None,
                    children: vec![],
                });
            }
        }
        "WebBookmarkTypeList" => {
            let title = dict
                .get("Title")
                .and_then(|v| v.as_string())
                .unwrap_or("Folder")
                .to_string();

            // Top-level pseudo-folders ("BookmarksBar", "BookmarksMenu") use
            // internal names; prefer them as-is for the path but don't let
            // an empty title collapse the hierarchy.
            let mut path = folder_path.to_vec();
            path.push(title);

            if let Some(children) = dict.get("Children").and_then(|v| v.as_array()) {
                for child in children {
                    parse_safari_bookmark_node(child, &path, out);
                }
            }
        }
        "WebBookmarkTypeProxy" => {
            // Smart folders (Reading List, etc.) â€” nothing to migrate.
        }
        _ => {
            // Root dictionary has no WebBookmarkType of its own; descend
            // into its Children to reach the real top-level folders.
            if let Some(children) = dict.get("Children").and_then(|v| v.as_array()) {
                for child in children {
                    parse_safari_bookmark_node(child, folder_path, out);
                }
            }
        }
    }
}
