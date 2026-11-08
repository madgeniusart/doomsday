//! Shared logic for all Chromium-based browsers.
//! Chrome, Edge, Brave and Opera all use the same profile format.

use super::{Bookmark, BrowserProfile, CookieInfo, DataCategory, ExtensionInfo, HistoryEntry};
use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
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

    // Reconcile Local State with the on-disk profile folders. Chrome-family
    // browsers can retain a profile directory even when its metadata cache is
    // missing or stale (for example after account/profile synchronization).
    // Discover those directories rather than silently hiding them.
    if let Ok(entries) = std::fs::read_dir(user_data_dir) {
        for entry in entries.flatten() {
            let profile_path = entry.path();
            if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                continue;
            }
            let Some(profile_dir) = profile_path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let is_known_profile_name = profile_dir == "Default" || profile_dir.starts_with("Profile ");
            if !is_known_profile_name
                || profiles.iter().any(|profile| profile.path == profile_path)
            {
                continue;
            }

            // Avoid presenting arbitrary folders as browser profiles.
            let preferences_path = profile_path.join("Preferences");
            let bookmarks_path = profile_path.join("Bookmarks");
            if !preferences_path.is_file() && !bookmarks_path.is_file() {
                continue;
            }

            let preferences = std::fs::read_to_string(&preferences_path)
                .ok()
                .and_then(|text| serde_json::from_str::<Value>(&text).ok());
            let display_name = preferences
                .as_ref()
                .and_then(|value| value.get("profile"))
                .and_then(|profile| profile.get("name"))
                .and_then(Value::as_str)
                .filter(|name| !name.trim().is_empty())
                .unwrap_or(profile_dir)
                .to_string();
            let email = preferences
                .as_ref()
                .and_then(|value| value.get("profile"))
                .and_then(|profile| profile.get("user_name"))
                .and_then(Value::as_str)
                .map(str::to_string);

            let is_default = profile_dir == "Default";

profiles.push(BrowserProfile {
    id: Uuid::new_v4().to_string(),
    display_name,
    path: profile_path,
    avatar_icon: None,
    is_default,
    email,
    last_used: None,
});
            profiles.push(BrowserProfile {
                id: Uuid::new_v4().to_string(),
                display_name,
                path: profile_path,
                avatar_icon: None,
                is_default: profile_dir == "Default",
                email,
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


/// Merge bookmarks into a Chromium-family destination profile.
/// The browser must be closed: Chromium rewrites this file while running.
pub fn import_chromium_bookmarks(profile_path: &PathBuf, bookmarks: &[Bookmark]) -> Result<usize> {
    use std::io::Write;

    let bookmarks_path = profile_path.join("Bookmarks");
    if let Some(user_data_dir) = profile_path.parent() {
        for marker in ["SingletonLock", "SingletonSocket"] {
            if std::fs::symlink_metadata(user_data_dir.join(marker)).is_ok() {
                anyhow::bail!("Chromium profile appears to be in use ({} exists); close the browser before migrating bookmarks", marker);
            }
        }
    }
    if !bookmarks_path.is_file() {
        anyhow::bail!("Destination Chromium profile has no Bookmarks file");
    }

    let content = std::fs::read_to_string(&bookmarks_path)
        .context("Failed to read destination Bookmarks file")?;
    let mut document: Value = serde_json::from_str(&content)
        .context("Destination Bookmarks file is not valid JSON")?;
    let bookmark_bar_exists = document.get("roots")
        .and_then(|roots| roots.get("bookmark_bar"))
        .and_then(Value::as_object)
        .is_some();
    if !bookmark_bar_exists {
        anyhow::bail!("Destination profile has no bookmark bar root");
    }

    let mut existing_urls = std::collections::HashSet::new();
    collect_chromium_urls(&document, &mut existing_urls);
    let mut next_id = max_chromium_bookmark_id(&document).saturating_add(1).max(1);
    let mut imported = 0usize;
    let import_root = "Imported from Universal Browser Sync".to_string();
    let now = chrono::Utc::now().timestamp_micros() + 11_644_473_600_000_000;

    // Reborrow the bookmark bar after the immutable scans above.
    let roots = document.get_mut("roots").and_then(Value::as_object_mut)
        .context("Destination Bookmarks roots disappeared")?;
    let bookmark_bar = roots.get_mut("bookmark_bar").and_then(Value::as_object_mut)
        .context("Destination bookmark bar disappeared")?;
    let children = bookmark_bar.entry("children").or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut().context("Destination bookmark bar children are invalid")?;

    for bookmark in bookmarks {
        let Some(url) = bookmark.url.as_deref() else { continue };
        let normalized = url.trim();
        if !(normalized.starts_with("https://") || normalized.starts_with("http://") || normalized.starts_with("ftp://")) {
            continue;
        }
        if !existing_urls.insert(normalized.to_string()) {
            continue;
        }

        let id = next_id.to_string();
        next_id = next_id.saturating_add(1);
        let node = serde_json::json!({
            "type": "url",
            "name": bookmark.title,
            "url": normalized,
            "id": id,
            "guid": Uuid::new_v4().simple().to_string(),
            "date_added": now.to_string(),
            "date_last_used": "0"
        });
        let mut path = vec![import_root.clone()];
        path.extend(bookmark.folder_path.iter().skip(1).filter(|part| !part.trim().is_empty()).cloned());
        insert_chromium_bookmark_node(children, &path, node, &mut next_id, now);
        imported += 1;
    }

    if imported == 0 {
        return Ok(0);
    }

    let serialized = serde_json::to_vec_pretty(&document)?;
    let backup_path = profile_path.join(format!("Bookmarks.ubs-backup-{}", chrono::Utc::now().format("%Y%m%dT%H%M%S%f")));
    std::fs::copy(&bookmarks_path, &backup_path)
        .context("Could not create a safety copy of destination bookmarks")?;

    let temp_path = profile_path.join(format!("Bookmarks.ubs-{}.tmp", Uuid::new_v4()));
    let write_result = (|| -> Result<()> {
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&temp_path)
            .context("Could not create temporary bookmarks file")?;
        file.write_all(&serialized).context("Could not write temporary bookmarks file")?;
        file.sync_all().context("Could not flush temporary bookmarks file")?;
        #[cfg(windows)]
        {
            // Windows rename does not replace an existing destination file.
            // Keep the original in place as a rollback point until the new file lands.
            let displaced = profile_path.join(format!("Bookmarks.ubs-displaced-{}.tmp", Uuid::new_v4()));
            std::fs::rename(&bookmarks_path, &displaced)
                .context("Could not stage the existing bookmarks file for replacement")?;
            if let Err(error) = std::fs::rename(&temp_path, &bookmarks_path) {
                let _ = std::fs::rename(&displaced, &bookmarks_path);
                return Err(error).context("Could not install the migrated bookmarks file");
            }
            let _ = std::fs::remove_file(displaced);
        }
        #[cfg(not(windows))]
        std::fs::rename(&temp_path, &bookmarks_path)
            .context("Could not atomically replace destination bookmarks file")?;
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = std::fs::remove_file(&temp_path);
        return Err(error);
    }
    Ok(imported)
}

fn collect_chromium_urls(value: &Value, urls: &mut std::collections::HashSet<String>) {
    match value {
        Value::Object(object) => {
            if object.get("type").and_then(Value::as_str) == Some("url") {
                if let Some(url) = object.get("url").and_then(Value::as_str) {
                    urls.insert(url.to_string());
                }
            }
            for child in object.values() { collect_chromium_urls(child, urls); }
        }
        Value::Array(items) => for item in items { collect_chromium_urls(item, urls); },
        _ => {}
    }
}

fn max_chromium_bookmark_id(value: &Value) -> u64 {
    match value {
        Value::Object(object) => {
            let own = object.get("id").and_then(Value::as_str).and_then(|id| id.parse::<u64>().ok()).unwrap_or(0);
            object.values().map(max_chromium_bookmark_id).fold(own, u64::max)
        }
        Value::Array(items) => items.iter().map(max_chromium_bookmark_id).max().unwrap_or(0),
        _ => 0,
    }
}

fn insert_chromium_bookmark_node(
    children: &mut Vec<Value>, path: &[String], node: Value, next_id: &mut u64, now: i64,
) {
    if path.is_empty() { children.push(node); return; }
    let folder_name = &path[0];
    let index = children.iter().position(|child| {
        child.get("type").and_then(Value::as_str) == Some("folder")
            && child.get("name").and_then(Value::as_str) == Some(folder_name.as_str())
    }).unwrap_or_else(|| {
        let id = next_id.to_string();
        *next_id = (*next_id).saturating_add(1);
        children.push(serde_json::json!({
            "type": "folder", "name": folder_name, "id": id,
            "guid": Uuid::new_v4().simple().to_string(), "date_added": now.to_string(),
            "date_modified": now.to_string(), "children": []
        }));
        children.len() - 1
    });
    if let Some(folder_children) = children[index].get_mut("children").and_then(Value::as_array_mut) {
        insert_chromium_bookmark_node(folder_children, &path[1..], node, next_id, now);
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


/// Import history into a Chromium-family destination using its supported SQLite
/// history schema. The browser must be closed; the destination DB is backed up
/// before a transaction is started. Source timestamps are Unix microseconds.
pub fn import_chromium_history(profile_path: &PathBuf, entries: &[HistoryEntry]) -> Result<usize> {
    let db_path = profile_path.join("History");
    if !db_path.is_file() {
        anyhow::bail!("Destination Chromium profile has no History database");
    }
    if let Some(user_data_dir) = profile_path.parent() {
        for marker in ["SingletonLock", "SingletonSocket"] {
            if std::fs::symlink_metadata(user_data_dir.join(marker)).is_ok() {
                anyhow::bail!("Destination Chromium profile appears to be in use; close the browser before importing history");
            }
        }
    }
    let backup_path = profile_path.join(format!("History.ubs-backup-{}", chrono::Utc::now().format("%Y%m%dT%H%M%S%f")));
    std::fs::copy(&db_path, &backup_path).context("Could not back up destination history database")?;
    let mut conn = Connection::open(&db_path).context("Could not open destination history database")?;
    let tx = conn.transaction().context("Could not begin history import transaction")?;
    let chrome_epoch_offset: i64 = 11_644_473_600_000_000;
    let mut imported = 0usize;
    {
        let mut find = tx.prepare("SELECT id FROM urls WHERE url = ?1")?;
        let mut add_url = tx.prepare(
            "INSERT INTO urls (url, title, visit_count, typed_count, last_visit_time, hidden)
             VALUES (?1, ?2, 0, 0, 0, 0)")?;
        let mut add_visit = tx.prepare(
            "INSERT INTO visits (url, visit_time, from_visit, transition, segment_id, visit_duration)
             VALUES (?1, ?2, 0, 0, 0, 0)")?;
        let mut find_visit = tx.prepare("SELECT id FROM visits WHERE url = ?1 AND visit_time = ?2 LIMIT 1")?;
        for entry in entries {
            let url = entry.url.trim();
            if !(url.starts_with("http://") || url.starts_with("https://") || url.starts_with("ftp://")) { continue; }
            let chrome_time = entry.visit_time.saturating_add(chrome_epoch_offset);
            if chrome_time <= 0 { continue; }
            let existing: Option<i64> = find.query_row(params![url], |r| r.get(0)).optional()?;
            let url_id = match existing {
                Some(id) => id,
                None => {
                    add_url.execute(params![url, entry.title.as_deref().unwrap_or(url)])?;
                    tx.last_insert_rowid()
                }
            };
            let duplicate: Option<i64> = find_visit.query_row(params![url_id, chrome_time], |r| r.get(0)).optional()?;
            if duplicate.is_some() { continue; }
            add_visit.execute(params![url_id, chrome_time])?;
            imported += 1;
        }
    }
    // Recompute aggregates from the visits table, preserving existing counts.
    tx.execute("UPDATE urls SET visit_count = (SELECT COUNT(*) FROM visits WHERE visits.url = urls.id), last_visit_time = COALESCE((SELECT MAX(visit_time) FROM visits WHERE visits.url = urls.id), last_visit_time)", [])?;
    tx.commit().context("Could not commit destination history import")?;
    Ok(imported)
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

#[cfg(test)]
mod profile_discovery_tests {
    use super::*;

    #[test]
    fn discovers_profile_folders_missing_from_local_state() {
        let root = std::env::temp_dir().join(format!("ubs_chrome_profiles_{}", Uuid::new_v4()));
        let profile = root.join("Profile 2");
        std::fs::create_dir_all(&profile).unwrap();
        std::fs::write(
            profile.join("Preferences"),
            r#"{"profile":{"name":"Synced profile","user_name":"user@example.test"}}"#,
        )
        .unwrap();
        std::fs::write(root.join("Local State"), r#"{"profile":{"info_cache":{}}}"#).unwrap();

        let profiles = enumerate_chromium_profiles(&root).unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].display_name, "Synced profile");
        assert_eq!(profiles[0].email.as_deref(), Some("user@example.test"));
        assert_eq!(profiles[0].path, profile);

        let _ = std::fs::remove_dir_all(root);
    }
}


#[cfg(test)]
mod chromium_bookmark_import_tests {
    use super::*;

    #[test]
    fn imports_new_bookmarks_and_deduplicates_existing_urls() {
        let dir = std::env::temp_dir().join(format!("ubs-bookmark-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let original = serde_json::json!({
            "version": 1,
            "roots": {
                "bookmark_bar": {"type":"folder", "name":"Bookmarks bar", "id":"1", "children":[
                    {"type":"url", "name":"Already there", "url":"https://example.com", "id":"2"}
                ]},
                "other": {"type":"folder", "name":"Other bookmarks", "id":"3", "children":[]}
            }
        });
        std::fs::write(dir.join("Bookmarks"), serde_json::to_vec(&original).unwrap()).unwrap();
        let source = vec![
            Bookmark { id:"a".into(), title:"Duplicate".into(), url:Some("https://example.com".into()), folder_path:vec!["Bookmarks Menu".into()], date_added:None, date_modified:None, children:vec![] },
            Bookmark { id:"b".into(), title:"New link".into(), url:Some("https://new.example/path".into()), folder_path:vec!["Bookmarks Menu".into(), "Research".into()], date_added:None, date_modified:None, children:vec![] },
        ];

        let imported = import_chromium_bookmarks(&dir, &source).unwrap();
        assert_eq!(imported, 1);
        assert!(dir.read_dir().unwrap().any(|entry| entry.unwrap().file_name().to_string_lossy().starts_with("Bookmarks.ubs-backup-")));
        let result: Value = serde_json::from_slice(&std::fs::read(dir.join("Bookmarks")).unwrap()).unwrap();
        let mut urls = std::collections::HashSet::new();
        collect_chromium_urls(&result, &mut urls);
        assert_eq!(urls.len(), 2);
        assert!(urls.contains("https://new.example/path"));
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(test)]
mod chromium_history_import_tests {
    use super::*;

    #[test]
    fn imports_visits_and_creates_database_backup() {
        let root = std::env::temp_dir().join(format!("ubs-history-test-{}", Uuid::new_v4()));
        let profile = root.join("Default");
        std::fs::create_dir_all(&profile).unwrap();
        let db_path = profile.join("History");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE urls (id INTEGER PRIMARY KEY, url LONGVARCHAR NOT NULL, title LONGVARCHAR, visit_count INTEGER DEFAULT 0, typed_count INTEGER DEFAULT 0, last_visit_time INTEGER DEFAULT 0, hidden INTEGER DEFAULT 0);
             CREATE TABLE visits (id INTEGER PRIMARY KEY, url INTEGER NOT NULL, visit_time INTEGER NOT NULL, from_visit INTEGER DEFAULT 0, transition INTEGER DEFAULT 0, segment_id INTEGER DEFAULT 0, visit_duration INTEGER DEFAULT 0);"
        ).unwrap();
        drop(conn);

        let entries = vec![HistoryEntry {
            url: "https://history.example/page".into(), title: Some("History page".into()),
            visit_time: chrono::Utc::now().timestamp_micros(), visit_count: 1, typed_count: Some(0),
        }];
        let imported = import_chromium_history(&profile, &entries).unwrap();
        assert_eq!(imported, 1);
        let verify = Connection::open(&db_path).unwrap();
        let count: i64 = verify.query_row("SELECT COUNT(*) FROM visits", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
        assert!(profile.read_dir().unwrap().any(|e| e.unwrap().file_name().to_string_lossy().starts_with("History.ubs-backup-")));
        std::fs::remove_dir_all(root).unwrap();
    }
}
