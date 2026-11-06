use crate::DataCategory;
use crate::BrowserKind;
use crate::adapters::{
    chrome::ChromeAdapter, firefox::FirefoxAdapter, edge::EdgeAdapter, 
    brave::BraveAdapter, opera::OperaAdapter, safari::SafariAdapter
};

use crate::database::{MigrationItemRecord, MigrationRecord};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

/// Global in-memory migration state (since Tauri commands are stateless per call)
static MIGRATION_STATE: std::sync::OnceLock<Arc<Mutex<HashMap<String, MigrationStatus>>>> =
    std::sync::OnceLock::new();

fn migration_state() -> &'static Arc<Mutex<HashMap<String, MigrationStatus>>> {
    MIGRATION_STATE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRequest {
    pub source_browser: BrowserKind,
    pub source_profile_path: String,
    pub dest_browser: BrowserKind,
    pub dest_profile_path: String,
    pub data_categories: Vec<DataCategory>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MigrationStatusKind {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    pub id: String,
    pub status: MigrationStatusKind,
    pub current_category: Option<String>,
    pub progress: f32,         // 0.0–1.0
    pub items_processed: u64,
    pub items_total: u64,
    pub items_failed: u64,
    pub errors: Vec<String>,
    pub report: Vec<MigrationItemResult>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationItemResult {
    pub category: String,
    pub item_key: String,
    pub label: String,
    pub status: String, // migrated | failed | skipped | incompatible
    pub message: Option<String>,
}

#[tauri::command]
pub async fn start_migration(
    request: MigrationRequest,
    app: AppHandle,
) -> Result<String, String> {
    let run_id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let initial_status = MigrationStatus {
        id: run_id.clone(),
        status: MigrationStatusKind::Pending,
        current_category: None,
        progress: 0.0,
        items_processed: 0,
        items_total: request.data_categories.len() as u64,
        items_failed: 0,
        errors: vec![],
        report: vec![],
        started_at: now.clone(),
        completed_at: None,
    };

    migration_state()
        .lock()
        .unwrap()
        .insert(run_id.clone(), initial_status);

    let run_id_clone = run_id.clone();
    let req = request.clone();

    // Run migration in a background task. We take an AppHandle (Clone + 'static)
    // rather than State<'_, AppState> so the task can outlive this command call
    // and still reach the database to persist the final result.
    tokio::spawn(async move {
        let result = run_migration(&run_id_clone, &req).await;

        let completed_at = chrono::Utc::now().to_rfc3339();
        let final_snapshot = {
            let mut state_map = migration_state().lock().unwrap();
            let status = match state_map.get_mut(&run_id_clone) {
                Some(s) => s,
                None => return, // cancelled + removed, or impossible race
            };

            match result {
                Ok(()) => {
                    if status.status != MigrationStatusKind::Cancelled {
                        status.status = MigrationStatusKind::Completed;
                    }
                }
                Err(e) => {
                    status.status = MigrationStatusKind::Failed;
                    status.errors.push(e.to_string());
                }
            }
            status.completed_at = Some(completed_at.clone());
            status.progress = 1.0;
            status.clone()
        };

        // Persist the run and its per-item report so history and the report
        // screen survive an app restart.
        if let Err(e) = persist_migration_result(&app, &req, &final_snapshot) {
            log::error!("Failed to persist migration run {}: {}", run_id_clone, e);
        }
    });

    Ok(run_id)
}

/// Write the completed run and its items to SQLite.
fn persist_migration_result(
    app: &AppHandle,
    request: &MigrationRequest,
    status: &MigrationStatus,
) -> anyhow::Result<()> {
    let state = app.state::<AppState>();
    let db = state
        .db
        .lock()
        .map_err(|_| anyhow::anyhow!("database lock poisoned"))?;

    let status_str = match status.status {
        MigrationStatusKind::Completed => "completed",
        MigrationStatusKind::Failed => "failed",
        MigrationStatusKind::Cancelled => "cancelled",
        MigrationStatusKind::Running => "running",
        MigrationStatusKind::Pending => "pending",
    };

    let data_types = serde_json::to_string(&request.data_categories)?;

    let record = MigrationRecord {
        id: status.id.clone(),
        source_browser: request.source_browser.as_str().to_string(),
        source_profile: request.source_profile_path.clone(),
        dest_browser: request.dest_browser.as_str().to_string(),
        dest_profile: request.dest_profile_path.clone(),
        data_types,
        status: status_str.to_string(),
        started_at: status.started_at.clone(),
        completed_at: status.completed_at.clone(),
        items_migrated: status
            .report
            .iter()
            .filter(|i| i.status == "migrated")
            .count() as i64,
        items_failed: status.items_failed as i64,
        error_message: status.errors.first().cloned(),
    };

    db.record_migration(&record)?;

    let items: Vec<MigrationItemRecord> = status
        .report
        .iter()
        .map(|i| MigrationItemRecord {
            category: i.category.clone(),
            item_key: i.item_key.clone(),
            item_label: i.label.clone(),
            status: i.status.clone(),
            message: i.message.clone(),
        })
        .collect();

    if !items.is_empty() {
        db.record_migration_items(&status.id, &items)?;
    }

    Ok(())
}

async fn run_migration(run_id: &str, request: &MigrationRequest) -> anyhow::Result<()> {
    use crate::adapters::*;

    let source_path = std::path::PathBuf::from(&request.source_profile_path);
    let dest_path = std::path::PathBuf::from(&request.dest_profile_path);

    let total = request.data_categories.len() as u64;
    let mut processed = 0u64;

    // Safety: ensure we never write to source profile
    if source_path == dest_path {
        return Err(anyhow::anyhow!("Source and destination profiles cannot be the same"));
    }

    for category in &request.data_categories {
        // Check for cancellation
        {
            let state_map = migration_state().lock().unwrap();
            if let Some(s) = state_map.get(run_id) {
                if s.status == MigrationStatusKind::Cancelled {
                    return Ok(());
                }
            }
        }

        // Update progress
        {
            let mut state_map = migration_state().lock().unwrap();
            if let Some(s) = state_map.get_mut(run_id) {
                s.status = MigrationStatusKind::Running;
                s.current_category = Some(category.label().to_string());
                s.progress = processed as f32 / total as f32;
            }
        }

        let result = migrate_category(category, request, &source_path, &dest_path).await;

        let (items, errors) = match result {
            Ok(items) => (items, vec![]),
            Err(e) => {
                let msg = e.to_string();
                log::error!("Migration category {:?} failed: {}", category, msg);
                (vec![], vec![msg])
            }
        };

        let failed = items.iter().filter(|i| i.status == "failed").count() as u64;

        {
            let mut state_map = migration_state().lock().unwrap();
            if let Some(s) = state_map.get_mut(run_id) {
                s.items_failed += failed;
                s.items_processed += 1;
                s.errors.extend(errors);
                s.report.extend(items);
            }
        }

        processed += 1;
    }

    Ok(())
}

async fn migrate_category(
    category: &DataCategory,
    request: &MigrationRequest,
    source_path: &std::path::Path,
    dest_path: &std::path::Path,
) -> anyhow::Result<Vec<MigrationItemResult>> {
    use crate::adapters::*;

    let source_path = source_path.to_path_buf();
    let dest_path = dest_path.to_path_buf();

    match category {
        DataCategory::Bookmarks => {
            // Read source bookmarks
            let bookmarks = match &request.source_browser {
                BrowserKind::Chrome => ChromeAdapter::new().read_bookmarks(&source_path)?,
                BrowserKind::Firefox => FirefoxAdapter::new().read_bookmarks(&source_path)?,
                BrowserKind::Edge => EdgeAdapter::new().read_bookmarks(&source_path)?,
                BrowserKind::Brave => BraveAdapter::new().read_bookmarks(&source_path)?,
                BrowserKind::Opera => OperaAdapter::new().read_bookmarks(&source_path)?,
                BrowserKind::Safari => SafariAdapter::new().read_bookmarks(&source_path)?,
            };

            if !request.dry_run {
                // Write bookmarks to destination in Netscape HTML format
                // then place in destination profile if same format, otherwise export
                write_bookmarks_to_dest(&bookmarks, &dest_path, &request.dest_browser)?;
            }

            let results = bookmarks
                .iter()
                .map(|b| MigrationItemResult {
                    category: "Bookmarks".to_string(),
                    item_key: b.id.clone(),
                    label: b.title.clone(),
                    status: "migrated".to_string(),
                    message: None,
                })
                .collect();

            Ok(results)
        }

        DataCategory::History => {
            let history = match &request.source_browser {
                BrowserKind::Chrome => ChromeAdapter::new().read_history(&source_path, 90)?,
                BrowserKind::Firefox => FirefoxAdapter::new().read_history(&source_path, 90)?,
                BrowserKind::Edge => EdgeAdapter::new().read_history(&source_path, 90)?,
                BrowserKind::Brave => BraveAdapter::new().read_history(&source_path, 90)?,
                BrowserKind::Opera => OperaAdapter::new().read_history(&source_path, 90)?,
                BrowserKind::Safari => SafariAdapter::new().read_history(&source_path, 90)?,
            };

            // History write is complex and browser-specific.
            // We export to JSON for now; direct injection requires browser-specific SQLite.
            if !request.dry_run && history.is_empty() == false {
                let export_path = dest_path.join("ubs_history_export.json");
                let json = serde_json::to_string_pretty(&history)?;
                std::fs::write(&export_path, json)?;
            }

            let results = history
                .iter()
                .map(|h| MigrationItemResult {
                    category: "History".to_string(),
                    item_key: h.url.clone(),
                    label: h.title.clone().unwrap_or_else(|| h.url.clone()),
                    status: "migrated".to_string(),
                    message: Some("Exported to JSON; direct history import not supported by all browsers".to_string()),
                })
                .collect();

            Ok(results)
        }

        DataCategory::Cookies => {
            let cookies = match &request.source_browser {
                BrowserKind::Chrome => ChromeAdapter::new().read_cookies(&source_path)?,
                BrowserKind::Firefox => FirefoxAdapter::new().read_cookies(&source_path)?,
                BrowserKind::Edge => EdgeAdapter::new().read_cookies(&source_path)?,
                BrowserKind::Brave => BraveAdapter::new().read_cookies(&source_path)?,
                BrowserKind::Opera => OperaAdapter::new().read_cookies(&source_path)?,
                BrowserKind::Safari => return Err(anyhow::anyhow!("Safari cookie access not supported")),
            };

            let non_auth: Vec<_> = cookies.iter().filter(|c| !c.is_auth_likely).collect();
            let auth_count = cookies.len() - non_auth.len();

            if !request.dry_run {
                let export_path = dest_path.join("ubs_cookies_export.json");
                let json = serde_json::to_string_pretty(&non_auth)?;
                std::fs::write(&export_path, json)?;
            }

            let mut results: Vec<MigrationItemResult> = non_auth
                .iter()
                .map(|c| MigrationItemResult {
                    category: "Cookies".to_string(),
                    item_key: format!("{}:{}", c.domain, c.name),
                    label: format!("{} — {}", c.domain, c.name),
                    status: "migrated".to_string(),
                    message: None,
                })
                .collect();

            // Auth cookies: note as skipped
            if auth_count > 0 {
                results.push(MigrationItemResult {
                    category: "Cookies".to_string(),
                    item_key: "auth_cookies".to_string(),
                    label: format!("{} authentication cookies", auth_count),
                    status: "skipped".to_string(),
                    message: Some("Authentication cookies skipped for security. Re-login required on these sites.".to_string()),
                });
            }

            Ok(results)
        }

        DataCategory::Extensions => {
            let extensions = match &request.source_browser {
                BrowserKind::Chrome => ChromeAdapter::new().read_extensions(&source_path)?,
                BrowserKind::Firefox => FirefoxAdapter::new().read_extensions(&source_path)?,
                BrowserKind::Edge => EdgeAdapter::new().read_extensions(&source_path)?,
                BrowserKind::Brave => BraveAdapter::new().read_extensions(&source_path)?,
                BrowserKind::Opera => OperaAdapter::new().read_extensions(&source_path)?,
                BrowserKind::Safari => SafariAdapter::new().read_extensions(&source_path)?,
            };

            let results = extensions
                .iter()
                .map(|ext| {
                    let (status, message) = if request.source_browser.is_chromium_based()
                        && request.dest_browser.is_chromium_based()
                    {
                        ("migrated".to_string(), Some("Extension ID listed — install manually from Chrome Web Store".to_string()))
                    } else {
                        ("incompatible".to_string(), Some("Extension format differs between browsers. Search for equivalent.".to_string()))
                    };
                    MigrationItemResult {
                        category: "Extensions".to_string(),
                        item_key: ext.id.clone(),
                        label: ext.name.clone(),
                        status,
                        message,
                    }
                })
                .collect();

            Ok(results)
        }

        DataCategory::Preferences => {
            // Browser preferences are highly browser-specific
            // We export source prefs as JSON reference
            let prefs_path = source_path.join("Preferences");
            let ff_prefs = source_path.join("prefs.js");

            if prefs_path.exists() && !request.dry_run {
                let export_path = dest_path.join("ubs_prefs_export.json");
                std::fs::copy(&prefs_path, &export_path)?;
            } else if ff_prefs.exists() && !request.dry_run {
                let export_path = dest_path.join("ubs_prefs_export.js");
                std::fs::copy(&ff_prefs, &export_path)?;
            }

            Ok(vec![MigrationItemResult {
                category: "Preferences".to_string(),
                item_key: "preferences".to_string(),
                label: "Browser Preferences".to_string(),
                status: "migrated".to_string(),
                message: Some("Preferences exported as reference — direct import may not be supported".to_string()),
            }])
        }

        _ => Ok(vec![MigrationItemResult {
            category: category.label().to_string(),
            item_key: "unsupported".to_string(),
            label: category.label().to_string(),
            status: "incompatible".to_string(),
            message: Some(format!("{} migration not yet implemented", category.label())),
        }]),
    }
}

fn write_bookmarks_to_dest(
    bookmarks: &[crate::adapters::Bookmark],
    dest_path: &std::path::Path,
    dest_browser: &BrowserKind,
) -> anyhow::Result<()> {
    if dest_browser.is_chromium_based() {
        // Write Netscape HTML import file to the profile directory
        let html = crate::commands::bookmarks::bookmarks_to_netscape_html(bookmarks);
        let export_path = dest_path.join("ubs_bookmarks_import.html");
        std::fs::write(export_path, html)?;
        // Note: programmatic injection into Chrome's Bookmarks JSON requires matching
        // Chrome's sync hash; using the HTML import file is the supported path.
    } else if *dest_browser == BrowserKind::Firefox {
        let html = crate::commands::bookmarks::bookmarks_to_netscape_html(bookmarks);
        let export_path = dest_path.join("ubs_bookmarks_import.html");
        std::fs::write(export_path, html)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_migration_status(run_id: String) -> Result<Option<MigrationStatus>, String> {
    let state_map = migration_state().lock().unwrap();
    Ok(state_map.get(&run_id).cloned())
}

#[tauri::command]
pub async fn cancel_migration(run_id: String) -> Result<(), String> {
    let mut state_map = migration_state().lock().unwrap();
    if let Some(status) = state_map.get_mut(&run_id) {
        status.status = MigrationStatusKind::Cancelled;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_migration_history(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<MigrationRecord>, String> {
    let db = state.db.lock().unwrap();
    db.list_migration_runs(limit.unwrap_or(50))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_migration_report(
    run_id: String,
    state: State<'_, AppState>,
) -> Result<Option<Vec<MigrationItemResult>>, String> {
    // Prefer the in-memory copy (covers runs completed in this session).
    {
        let state_map = migration_state().lock().unwrap();
        if let Some(status) = state_map.get(&run_id) {
            return Ok(Some(status.report.clone()));
        }
    }

    // Fall back to the database for runs from a previous app session.
    let db = state.db.lock().unwrap();
    let items = db.get_migration_items(&run_id).map_err(|e| e.to_string())?;

    if items.is_empty() {
        return Ok(None);
    }

    let report = items
        .into_iter()
        .map(|i| MigrationItemResult {
            category: i.category,
            item_key: i.item_key,
            label: i.item_label,
            status: i.status,
            message: i.message,
        })
        .collect();

    Ok(Some(report))
}
