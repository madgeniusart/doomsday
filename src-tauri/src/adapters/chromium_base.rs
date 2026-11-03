//! Shared logic for all Chromium-based browsers.
//! Chrome, Edge, Brave and Opera all use the same profile format.

use super::{Bookmark, BrowserProfile, CookieInfo, DataCategory, ExtensionInfo, HistoryEntry};
use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags, params};
use serde_json::Value;
use std::path::PathBuf;
use uuid::Uuid;

/// Read Chromium profiles from a given user-data directory
pub fn enumerate_chromium_profiles(user_data_dir: &PathBuf) -> Result<Vec<BrowserProfile>> {
    if !user_data_dir.exists() {
        return Ok(vec![]);
    }

    let mut profiles = Vec::new();

    // Read Local State to get profile metadata
    let local_state_path = user_data_dir.join("Local State");
    if let Ok(content) = std::fs::read_to_string(&local_state_path) {
        if let Ok(json) = serde_json::from_str::<Value>(&content) {
            if let Some(info_cache) = json
                .get("profile")
                .and_then(|p| p.get("info_cache"))
                .and_then(|c| c.as_object())
            {
                for (profile_dir, profile_info) in info_cache {
                    let profile_path = user_data_dir.join(profile_dir);
                    if !profile_path.is_dir() {
                        continue;
                    }

                    let display_name = profile_info
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or(profile_dir)
                        .to_string();

                    let email = profile_info
                        .get("user_name")
                        .and_then(|u| u.as_str())
                        .map(|s| s.to_string());

                    let avatar_icon = profile_info
                        .get("avatar_icon")
                        .and_then(|a| a.as_str())
                        .map(|s| s.to_string());

                    let is_default = profile_dir == "Default";

                    profiles.push(BrowserProfile {
                        id: Uuid::new_v4().to_string(),
                        display_name,
                        path: profile_path,
                        avatar_icon,
                        is_default,
                        email,
                        last_used: None,
                    });
                }
            }
        }
    }

    // Fallback: scan for Default directory even without Local State
    if profiles.is_empty() {
        let default_path = user_data_dir.join("Default");
        if default_path.is_dir() {
            profiles.push(BrowserProfile {
                id: Uuid::new_v4().to_string(),
                display_name: "Default".to_string(),
                path: default_path,
                avatar_icon: None,
                is_default: true,
                email: None,
                last_used: None,
            });
        }
    }

    // Sort: Default first, then alphabetical
    profiles.sort_by(|a, b| {
        b.is_default
            .cmp(&a.is_default)
            .then(a.display_name.cmp(&b.display_name))
    });

    Ok(profiles)
}

/// Read Chromium Bookmarks JSON file
pub fn read_chromium_bookmarks(profile_path: &PathBuf) -> Result<Vec<Bookmark>> {
    let bookmarks_path = profile_path.join("Bookmarks");
    if !bookmarks_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&bookmarks_path)
        .context("Failed to read Bookmarks file")?;
    let json: Value =
        serde_json::from_str(&content).context("Failed to parse Bookmarks JSON")?;

    let roots = json
        .get("roots")
        .context("No 'roots' in Bookmarks")?;

    let mut all_bookmarks = Vec::new();

    for root_key in &["bookmark_bar", "other", "synced"] {
        if let Some(root_node) = roots.get(root_key) {
            let folder_name = root_node
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or(root_key);
            parse_chromium_bookmark_node(
                root_node,
                &[folder_name.to_string()],
                &mut all_bookmarks,
            );
        }
    }

    Ok(all_bookmarks)
}

fn parse_chromium_bookmark_node(
    node: &Value,
    folder_path: &[String],
    out: &mut Vec<Bookmark>,
) {
    let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let id = node
        .get("id")
        .and_then(|i| i.as_str())
        .unwrap_or("")
        .to_string();
    let title = node
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();
    let date_added = node
        .get("date_added")
        .and_then(|d| d.as_str())
        .and_then(|d| d.parse::<i64>().ok());

    match node_type {
        "url" => {
            let url = node
                .get("url")
                .and_then(|u| u.as_str())
                .unwrap_or("")
                .to_string();
            out.push(Bookmark {
                id,
                title,
                url: Some(url),
                folder_path: folder_path.to_vec(),
                date_added,
                date_modified: None,
                children: vec![],
            });
        }
        "folder" => {
            let mut path = folder_path.to_vec();
            path.push(title.clone());

            if let Some(children) = node.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    parse_chromium_bookmark_node(child, &path, out);
                }
            }
        }
        _ => {}
    }
}

/// Read browsing history from Chromium's History SQLite database.
/// Opens in read-only mode — NEVER modifies source data.
pub fn read_chromium_history(
    profile_path: &PathBuf,
    max_days: u32,
) -> Result<Vec<HistoryEntry>> {
    let history_db = profile_path.join("History");
    if !history_db.exists() {
        return Ok(vec![]);
    }

    // Copy to temp file because Chrome may have it locked
    let tmp = std::env::temp_dir().join(format!(
        "ubs_history_{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::copy(&history_db, &tmp).context("Cannot copy History DB (browser may be open)")?;

    let conn = Connection::open_with_flags(&tmp, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    // Chrome stores timestamps as microseconds since 1601-01-01 00:00:00 UTC
    // Convert max_days to cutoff timestamp
    let cutoff_days_ago = chrono::Utc::now()
        - chrono::Duration::try_days(max_days as i64).unwrap_or_default();
    let cutoff_unix_us = cutoff_days_ago.timestamp_micros();
    // Chrome epoch offset: microseconds from 1601 to 1970
    let chrome_epoch_offset: i64 = 11_644_473_600_000_000;
    let cutoff_chrome = cutoff_unix_us + chrome_epoch_offset;

    let mut stmt = conn.prepare(
        "SELECT urls.url, urls.title, visits.visit_time, urls.visit_count, urls.typed_count
         FROM visits
         JOIN urls ON visits.url = urls.id
         WHERE visits.visit_time > ?1
         ORDER BY visits.visit_time DESC
         LIMIT 50000",
    )?;

    let entries: Vec<HistoryEntry> = stmt
        .query_map(params![cutoff_chrome], |row| {
            Ok(HistoryEntry {
                url: row.get(0)?,
                title: row.get(1)?,
                visit_time: row.get::<_, i64>(2)? - chrome_epoch_offset,
                visit_count: row.get(3)?,
                typed_count: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let _ = std::fs::remove_file(&tmp);
    Ok(entries)
}

/// Read cookie METADATA from Chromium's Cookies database.
/// Cookie values are intentionally excluded for auth-type cookies.
pub fn read_chromium_cookies(profile_path: &PathBuf) -> Result<Vec<CookieInfo>> {
    let cookies_db_path = profile_path
        .join("Network")
        .join("Cookies")
        .exists()
        .then(|| profile_path.join("Network").join("Cookies"))
        .unwrap_or_else(|| profile_path.join("Cookies"));

    if !cookies_db_path.exists() {
        return Ok(vec![]);
    }

    let tmp = std::env::temp_dir().join(format!("ubs_cookies_{}", uuid::Uuid::new_v4()));
    std::fs::copy(&cookies_db_path, &tmp)
        .context("Cannot copy Cookies DB")?;

    let conn = Connection::open_with_flags(&tmp, OpenFlags::SQLITE_OPEN_READ_ONLY)?;

    let chrome_epoch_offset: i64 = 11_644_473_600;

    let mut stmt = conn.prepare(
        "SELECT host_key, name, path, is_secure, is_httponly, samesite, expires_utc, is_persistent
         FROM cookies
         ORDER BY host_key, name",
    )?;

    let entries: Vec<CookieInfo> = stmt
        .query_map([], |row| {
            let host: String = row.get(0)?;
            let name: String = row.get(1)?;
            let path: String = row.get(2)?;
            let secure: bool = row.get::<_, i64>(3)? != 0;
            let http_only: bool = row.get::<_, i64>(4)? != 0;
            let same_site_val: i64 = row.get(5)?;
            let expires_chrome: i64 = row.get(6)?;
            let is_persistent: bool = row.get::<_, i64>(7)? != 0;

            let same_site = match same_site_val {
                0 => "None",
                1 => "Lax",
                2 => "Strict",
                _ => "None",
            }
            .to_string();

            let expires = if expires_chrome > 0 {
                Some((expires_chrome / 1_000_000) - chrome_epoch_offset)
            } else {
                None
            };

            // Heuristic: likely auth if name matches common session patterns
            let is_auth_likely = is_auth_cookie_name(&name);

            Ok(CookieInfo {
                domain: host,
                name,
                path,
                secure,
                http_only,
                same_site,
                expires,
                is_session: !is_persistent,
                is_auth_likely,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    let _ = std::fs::remove_file(&tmp);
    Ok(entries)
}

/// Read Chromium extensions from profile
pub fn read_chromium_extensions(profile_path: &PathBuf) -> Result<Vec<ExtensionInfo>> {
    let extensions_dir = profile_path.join("Extensions");
    if !extensions_dir.exists() {
        return Ok(vec![]);
    }

    // Also read from Preferences to get enabled state
    let prefs_path = profile_path.join("Preferences");
    let prefs_json: Value = std::fs::read_to_string(&prefs_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null);

    let extensions_settings = prefs_json
        .get("extensions")
        .and_then(|e| e.get("settings"))
        .and_then(|s| s.as_object());

    let mut extensions = Vec::new();

    for entry in std::fs::read_dir(&extensions_dir)? {
        let entry = entry?;
        let ext_id = entry.file_name().to_string_lossy().to_string();

        // Skip internal extensions
        if ext_id.starts_with('_') {
            continue;
        }

        // Find latest version directory
        let ext_dir = entry.path();
        let version_dir = std::fs::read_dir(&ext_dir)
            .ok()
            .and_then(|mut d| d.next())
            .and_then(|e| e.ok())
            .map(|e| e.path());

        if let Some(ver_dir) = version_dir {
            let manifest_path = ver_dir.join("manifest.json");
            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                if let Ok(manifest) = serde_json::from_str::<Value>(&content) {
                    let name = manifest
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("Unknown Extension")
                        .to_string();

                    // Skip if it looks like a placeholder message key
                    if name.starts_with("__MSG_") {
                        continue;
                    }

                    let version = manifest
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let description = manifest
                        .get("description")
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string());

                    let permissions: Vec<String> = manifest
                        .get("permissions")
                        .and_then(|p| p.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect()
                        })
                        .unwrap_or_default();

                    // Check enabled state from prefs
                    let enabled = extensions_settings
                        .and_then(|s| s.get(&ext_id))
                        .and_then(|e| e.get("state"))
                        .and_then(|s| s.as_i64())
                        .map(|s| s == 1)
                        .unwrap_or(true);

                    let store_url = Some(format!(
                        "https://chrome.google.com/webstore/detail/{}",
                        ext_id
                    ));

                    extensions.push(ExtensionInfo {
                        id: ext_id,
                        name,
                        version,
                        description,
                        enabled,
                        store_url,
                        permissions,
                    });
                }
            }
        }
    }

    extensions.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(extensions)
}

pub fn chromium_supported_categories() -> Vec<DataCategory> {
    vec![
        DataCategory::Bookmarks,
        DataCategory::History,
        DataCategory::Cookies,
        DataCategory::Extensions,
        DataCategory::Preferences,
        DataCategory::SearchEngines,
        DataCategory::Permissions,
    ]
}

/// Heuristic: common session/auth cookie names
fn is_auth_cookie_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    let auth_patterns = [
        "session", "sess", "auth", "token", "jwt", "login",
        "sid", "user_id", "userid", "account", "csrftoken",
        "xsrf", "access_token", "refresh_token", "id_token",
    ];
    auth_patterns.iter().any(|p| lower.contains(p))
}
