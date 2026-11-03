use super::{
    BrowserAdapter, BrowserKind, BrowserProfile, Bookmark,
    CookieInfo, DataCategory, ExtensionInfo, HistoryEntry,
};
use anyhow::{Context, Result};
use rusqlite::{Connection, OpenFlags, params};
use std::path::PathBuf;
use uuid::Uuid;

pub struct FirefoxAdapter {
    profiles_dir: PathBuf,
}

impl FirefoxAdapter {
    pub fn new() -> Self {
        Self {
            profiles_dir: firefox_profiles_dir(),
        }
    }

    fn open_db_readonly(db_path: &PathBuf) -> Result<Connection> {
        // Copy to temp to avoid locking issues while Firefox is open
        let tmp = std::env::temp_dir().join(format!(
            "ubs_ff_{}",
            Uuid::new_v4()
        ));
        std::fs::copy(db_path, &tmp).context("Cannot copy Firefox DB (browser may be open)")?;
        let conn = Connection::open_with_flags(&tmp, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        Ok(conn)
    }
}

impl BrowserAdapter for FirefoxAdapter {
    fn kind(&self) -> BrowserKind {
        BrowserKind::Firefox
    }

    fn enumerate_profiles(&self) -> Result<Vec<BrowserProfile>> {
        if !self.profiles_dir.exists() {
            return Ok(vec![]);
        }

        // Parse profiles.ini
        let profiles_ini = self.profiles_dir.join("profiles.ini");
        if !profiles_ini.exists() {
            return Ok(vec![]);
        }

        let content =
            std::fs::read_to_string(&profiles_ini).context("Failed to read profiles.ini")?;

        let mut profiles = Vec::new();
        let mut current_name = None::<String>;
        let mut current_path = None::<String>;
        let mut current_default = false;
        let mut is_relative = true;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') && line.ends_with(']') {
                // Save previous profile
                if let (Some(name), Some(path)) = (current_name.take(), current_path.take()) {
                    let full_path = if is_relative {
                        self.profiles_dir.join(&path)
                    } else {
                        PathBuf::from(&path)
                    };
                    if full_path.is_dir() {
                        profiles.push(BrowserProfile {
                            id: Uuid::new_v4().to_string(),
                            display_name: name,
                            path: full_path,
                            avatar_icon: None,
                            is_default: current_default,
                            email: None,
                            last_used: None,
                        });
                    }
                }
                current_default = false;
                is_relative = true;
            } else if let Some(rest) = line.strip_prefix("Name=") {
                current_name = Some(rest.to_string());
            } else if let Some(rest) = line.strip_prefix("Path=") {
                current_path = Some(rest.to_string());
            } else if line == "Default=1" {
                current_default = true;
            } else if line == "IsRelative=0" {
                is_relative = false;
            }
        }

        // Last profile
        if let (Some(name), Some(path)) = (current_name, current_path) {
            let full_path = if is_relative {
                self.profiles_dir.join(&path)
            } else {
                PathBuf::from(&path)
            };
            if full_path.is_dir() {
                profiles.push(BrowserProfile {
                    id: Uuid::new_v4().to_string(),
                    display_name: name,
                    path: full_path,
                    avatar_icon: None,
                    is_default: current_default,
                    email: None,
                    last_used: None,
                });
            }
        }

        // Sort: default first
        profiles.sort_by(|a, b| b.is_default.cmp(&a.is_default));
        Ok(profiles)
    }

    fn read_bookmarks(&self, profile_path: &PathBuf) -> Result<Vec<Bookmark>> {
        let places_db = profile_path.join("places.sqlite");
        if !places_db.exists() {
            return Ok(vec![]);
        }

        let conn = Self::open_db_readonly(&places_db)?;

        // moz_bookmarks references moz_places for URLs and moz_bookmarks for folders
        let mut stmt = conn.prepare(
            "SELECT b.id, b.title, p.url, b.parent, b.position, b.dateAdded, b.lastModified,
                    b.type
             FROM moz_bookmarks b
             LEFT JOIN moz_places p ON b.fk = p.id
             WHERE b.type IN (1, 2)  -- 1=bookmark, 2=folder
             ORDER BY b.parent, b.position",
        )?;

        #[derive(Debug)]
        struct RawBookmark {
            id: i64,
            title: Option<String>,
            url: Option<String>,
            parent: i64,
            _position: i64,
            date_added: Option<i64>,
            date_modified: Option<i64>,
            bm_type: i64,
        }

        let raw: Vec<RawBookmark> = stmt
            .query_map([], |row| {
                Ok(RawBookmark {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    url: row.get(2)?,
                    parent: row.get(3)?,
                    _position: row.get(4)?,
                    date_added: row.get(5)?,
                    date_modified: row.get(6)?,
                    bm_type: row.get(7)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Build parent-title map
        let folder_titles: std::collections::HashMap<i64, String> = raw
            .iter()
            .filter(|r| r.bm_type == 2)
            .map(|r| (r.id, r.title.clone().unwrap_or_else(|| "Folder".to_string())))
            .collect();

        // Resolve folder path
        let get_path = |mut parent_id: i64| -> Vec<String> {
            let mut path = Vec::new();
            let mut seen = std::collections::HashSet::new();
            while let Some(name) = folder_titles.get(&parent_id) {
                if !seen.insert(parent_id) {
                    break;
                }
                path.push(name.clone());
                // Find parent of this folder
                if let Some(r) = raw.iter().find(|r| r.id == parent_id) {
                    parent_id = r.parent;
                } else {
                    break;
                }
            }
            path.reverse();
            path
        };

        let bookmarks: Vec<Bookmark> = raw
            .iter()
            .filter(|r| r.bm_type == 1 && r.url.is_some())
            .map(|r| Bookmark {
                id: r.id.to_string(),
                title: r.title.clone().unwrap_or_else(|| r.url.clone().unwrap_or_default()),
                url: r.url.clone(),
                folder_path: get_path(r.parent),
                date_added: r.date_added.map(|d| d / 1000), // microseconds → milliseconds
                date_modified: r.date_modified.map(|d| d / 1000),
                children: vec![],
            })
            .collect();

        Ok(bookmarks)
    }

    fn read_history(&self, profile_path: &PathBuf, max_days: u32) -> Result<Vec<HistoryEntry>> {
        let places_db = profile_path.join("places.sqlite");
        if !places_db.exists() {
            return Ok(vec![]);
        }

        let conn = Self::open_db_readonly(&places_db)?;

        // Firefox stores visit_date as microseconds since Unix epoch
        let cutoff = chrono::Utc::now()
            - chrono::Duration::try_days(max_days as i64).unwrap_or_default();
        let cutoff_us = cutoff.timestamp_micros();

        let mut stmt = conn.prepare(
            "SELECT p.url, p.title, h.visit_date, p.visit_count
             FROM moz_historyvisits h
             JOIN moz_places p ON h.place_id = p.id
             WHERE h.visit_date > ?1
             ORDER BY h.visit_date DESC
             LIMIT 50000",
        )?;

        let entries: Vec<HistoryEntry> = stmt
            .query_map(params![cutoff_us], |row| {
                Ok(HistoryEntry {
                    url: row.get(0)?,
                    title: row.get(1)?,
                    visit_time: row.get::<_, i64>(2)?,
                    visit_count: row.get(3)?,
                    typed_count: None,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }

    fn read_cookies(&self, profile_path: &PathBuf) -> Result<Vec<CookieInfo>> {
        let cookies_db = profile_path.join("cookies.sqlite");
        if !cookies_db.exists() {
            return Ok(vec![]);
        }

        let conn = Self::open_db_readonly(&cookies_db)?;

        let mut stmt = conn.prepare(
            "SELECT host, name, path, isSecure, isHttpOnly, sameSite, expiry, isSession
             FROM moz_cookies
             ORDER BY host, name",
        )?;

        let entries: Vec<CookieInfo> = stmt
            .query_map([], |row| {
                let name: String = row.get(1)?;
                let same_site_val: i64 = row.get(5)?;
                let same_site = match same_site_val {
                    0 => "None",
                    1 => "Lax",
                    2 => "Strict",
                    _ => "None",
                }
                .to_string();

                Ok(CookieInfo {
                    domain: row.get(0)?,
                    name: name.clone(),
                    path: row.get(2)?,
                    secure: row.get::<_, bool>(3)?,
                    http_only: row.get::<_, bool>(4)?,
                    same_site,
                    expires: row.get(6)?,
                    is_session: row.get::<_, bool>(7)?,
                    is_auth_likely: is_auth_cookie_name(&name),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }

    fn read_extensions(&self, profile_path: &PathBuf) -> Result<Vec<ExtensionInfo>> {
        let extensions_json = profile_path.join("extensions.json");
        if !extensions_json.exists() {
            return Ok(vec![]);
        }

        let content = std::fs::read_to_string(&extensions_json)?;
        let json: serde_json::Value = serde_json::from_str(&content)?;

        let addons = json
            .get("addons")
            .and_then(|a| a.as_array())
            .cloned()
            .unwrap_or_default();

        let extensions: Vec<ExtensionInfo> = addons
            .iter()
            .filter_map(|addon| {
                let id = addon.get("id")?.as_str()?.to_string();
                let name = addon
                    .get("defaultLocale")
                    .and_then(|l| l.get("name"))
                    .and_then(|n| n.as_str())
                    .or_else(|| addon.get("name")?.as_str())?
                    .to_string();
                let version = addon
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let description = addon
                    .get("defaultLocale")
                    .and_then(|l| l.get("description"))
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string());
                let enabled = addon
                    .get("active")
                    .and_then(|a| a.as_bool())
                    .unwrap_or(false);
                let store_url = addon
                    .get("sourceURI")
                    .and_then(|u| u.as_str())
                    .map(|s| s.to_string());

                Some(ExtensionInfo {
                    id,
                    name,
                    version,
                    description,
                    enabled,
                    store_url,
                    permissions: vec![],
                })
            })
            .collect();

        Ok(extensions)
    }

    fn supported_categories(&self) -> Vec<DataCategory> {
        vec![
            DataCategory::Bookmarks,
            DataCategory::History,
            DataCategory::Cookies,
            DataCategory::Extensions,
            DataCategory::Preferences,
        ]
    }
}

fn firefox_profiles_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(&home).join(".mozilla/firefox")
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(&home).join("Library/Application Support/Firefox/Profiles")
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        PathBuf::from(&appdata).join("Mozilla\\Firefox\\Profiles")
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    PathBuf::new()
}

fn is_auth_cookie_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    ["session", "auth", "token", "jwt", "login", "sid", "user_id"]
        .iter()
        .any(|p| lower.contains(p))
}
