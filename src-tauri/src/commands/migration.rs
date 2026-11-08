use anyhow::Context;
use crate::DataCategory;
use crate::BrowserKind;
use crate::adapters::{
    chrome::ChromeAdapter, firefox::FirefoxAdapter, edge::EdgeAdapter,
    brave::BraveAdapter, opera::OperaAdapter, safari::SafariAdapter,
    BrowserAdapter, BrowserDetector, chromium_base::{import_chromium_bookmarks, import_chromium_history}
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
    Partial,
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

    tokio::spawn(async move {
        let result = run_migration(&run_id_clone, &req).await;

        let completed_at = chrono::Utc::now().to_rfc3339();
        let final_snapshot = {
            let mut state_map = migration_state().lock().unwrap();
            let status = match state_map.get_mut(&run_id_clone) {
                Some(s) => s,
                None => return,
            };

            match result {
                Ok(()) => {
                    if status.status != MigrationStatusKind::Cancelled {
                        let needs_user_action = !req.dry_run && status.report.iter().any(|item|
                            matches!(item.status.as_str(), "exported" | "skipped" | "incompatible" | "failed")
                        );
                        status.status = if status.items_failed > 0 || !status.errors.is_empty() || needs_user_action {
                            MigrationStatusKind::Partial
                        } else {
                            MigrationStatusKind::Completed
                        };
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

        if let Err(e) = persist_migration_result(&app, &req, &final_snapshot) {
            log::error!("Failed to persist migration run {}: {}", run_id_clone, e);
        }
    });

    Ok(run_id)
}

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
        MigrationStatusKind::Partial => "partial",
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
    let source_path = std::path::PathBuf::from(&request.source_profile_path);
    let dest_path = std::path::PathBuf::from(&request.dest_profile_path);

    let total = request.data_categories.len() as u64;
    let mut processed = 0u64;

    if request.data_categories.is_empty() {
        anyhow::bail!("Select at least one data category to migrate");
    }
    if !source_path.is_dir() {
        anyhow::bail!("Source profile directory does not exist or cannot be accessed");
    }
    if !dest_path.is_dir() {
        anyhow::bail!("Destination profile directory does not exist or cannot be accessed");
    }
    let canonical_source = source_path.canonicalize()?;
    let canonical_dest = dest_path.canonicalize()?;
    if canonical_source == canonical_dest {
        anyhow::bail!("Source and destination profiles cannot be the same");
    }
    if canonical_source.starts_with(&canonical_dest) || canonical_dest.starts_with(&canonical_source) {
        anyhow::bail!("Source and destination profile directories cannot contain one another");
    }

    for category in &request.data_categories {
        {
            let state_map = migration_state().lock().unwrap();
            if let Some(s) = state_map.get(run_id) {
                if s.status == MigrationStatusKind::Cancelled {
                    return Ok(());
                }
            }
        }

        {
            let mut state_map = migration_state().lock().unwrap();
            if let Some(s) = state_map.get_mut(run_id) {
                s.status = MigrationStatusKind::Running;
                s.current_category = Some(category.label().to_string());
                s.progress = processed as f32 / total as f32;
            }
        }

        let result = migrate_category(category, request, &source_path, &dest_path).await;

        let (mut items, errors) = match result {
            Ok(items) => (items, vec![]),
            Err(e) => {
                let msg = format!("{}: {}", category.label(), e);
                log::error!("Migration category {:?} failed: {}", category, msg);
                let failed_item = MigrationItemResult {
                    category: category.label().to_string(),
                    item_key: "category_failure".to_string(),
                    label: format!("{} migration failed", category.label()),
                    status: "failed".to_string(),
                    message: Some(msg.clone()),
                };
                (vec![failed_item], vec![msg])
            }
        };

        if request.dry_run {
            for item in &mut items {
                if item.status == "exported" {
                    item.status = "planned".to_string();
                    item.message = Some(format!(
                        "Dry run only — {}",
                        item.message.as_deref().unwrap_or("would be exported")
                    ));
                }
            }
        }

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
    let source_path = source_path.to_path_buf();
    let dest_path = dest_path.to_path_buf();

    match category {
        DataCategory::Bookmarks => {
            let bookmarks = match &request.source_browser {
                BrowserKind::Chrome => ChromeAdapter::new().read_bookmarks(&source_path)?,
                BrowserKind::Firefox => FirefoxAdapter::new().read_bookmarks(&source_path)?,
                BrowserKind::Edge => EdgeAdapter::new().read_bookmarks(&source_path)?,
                BrowserKind::Brave => BraveAdapter::new().read_bookmarks(&source_path)?,
                BrowserKind::Opera => OperaAdapter::new().read_bookmarks(&source_path)?,
                BrowserKind::Safari => SafariAdapter::new().read_bookmarks(&source_path)?,
            };

            let destination_is_chromium = request.dest_browser.is_chromium_based();
            let imported_count = if request.dry_run {
                0
            } else if destination_is_chromium {
                if BrowserDetector::is_running(&request.dest_browser) {
                    anyhow::bail!("Close the destination browser before importing bookmarks. This prevents Chromium from overwriting its bookmark database.");
                }
                import_chromium_bookmarks(&dest_path, &bookmarks)?
            } else {
                write_bookmarks_to_dest(&bookmarks, &dest_path, &request.dest_browser)?;
                0
            };

            let (result_status, result_message, result_label) = if request.dry_run {
                ("planned", "Dry run: source bookmarks were inspected; no destination files were changed".to_string(), format!("{} bookmarks planned", bookmarks.len()))
            } else if destination_is_chromium && imported_count > 0 {
                ("migrated", "Bookmarks imported into the destination Chromium database; a timestamped backup was created first".to_string(), format!("{} bookmarks imported", imported_count))
            } else if destination_is_chromium {
                ("skipped", "No new bookmarks were imported because all source URLs already existed or used unsupported URL schemes".to_string(), "No new bookmarks to import".to_string())
            } else {
                ("exported", "Created a Netscape HTML bookmark file in the destination profile; use the browser's Import Bookmarks command to complete this category".to_string(), format!("{} bookmarks exported", bookmarks.len()))
            };
            Ok(vec![MigrationItemResult {
                category: "Bookmarks".to_string(),
                item_key: "bookmarks_batch".to_string(),
                label: result_label,
                status: result_status.to_string(),
                message: Some(result_message),
            }])
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

            if history.is_empty() {
                return Ok(vec![MigrationItemResult {
                    category: "History".to_string(),
                    item_key: "history_empty".to_string(),
                    label: "No history entries found".to_string(),
                    status: if request.dry_run { "planned" } else { "skipped" }.to_string(),
                    message: Some("No entries were found in the selected time window".to_string()),
                }]);
            }

            if request.dest_browser.is_chromium_based() {
                if request.dry_run {
                    return Ok(vec![MigrationItemResult {
                        category: "History".to_string(), item_key: "history_batch".to_string(),
                        label: format!("{} history visits planned", history.len()), status: "planned".to_string(),
                        message: Some("Dry run: visits will be merged into the destination Chromium History database after the browser is closed; a database backup is created first".to_string()),
                    }]);
                }
                if BrowserDetector::is_running(&request.dest_browser) {
                    anyhow::bail!("Close the destination browser before importing history; this prevents database corruption");
                }
                let imported = import_chromium_history(&dest_path, &history)?;
                return Ok(vec![MigrationItemResult {
                    category: "History".to_string(), item_key: "history_batch".to_string(),
                    label: format!("{} history visits imported", imported),
                    status: if imported > 0 { "migrated" } else { "skipped" }.to_string(),
                    message: Some(format!("Imported {imported} visits into the destination Chromium History database. A timestamped database backup was created.")),
                }]);
            }

            let export_path = dest_path.join(format!("ubs_history_export_{}.json", Uuid::new_v4()));
            if !request.dry_run {
                let json = serde_json::to_vec_pretty(&history)?;
                write_new_export(&export_path, &json)?;
            }

            Ok(vec![MigrationItemResult {
                category: "History".to_string(),
                item_key: "history_batch".to_string(),
                label: format!("{} history entries", history.len()),
                status: (if request.dry_run { "planned" } else { "exported" }).to_string(),
                message: Some(if request.dry_run {
                    "Dry run: history will be exported to JSON because this destination does not have a supported native history importer".to_string()
                } else {
                    format!("History exported to {}; it has not been imported into the destination browser", export_path.display())
                }),
            }])
        }
        DataCategory::Cookies => {
            let cookies = match &request.source_browser {
                BrowserKind::Chrome => ChromeAdapter::new().read_cookies(&source_path)?,
                BrowserKind::Firefox => FirefoxAdapter::new().read_cookies(&source_path)?,
                BrowserKind::Edge => EdgeAdapter::new().read_cookies(&source_path)?,
                BrowserKind::Brave => BraveAdapter::new().read_cookies(&source_path)?,
                BrowserKind::Opera => OperaAdapter::new().read_cookies(&source_path)?,
                BrowserKind::Safari => return Err(anyhow::anyhow!("Safari cookie metadata is unavailable through the supported adapter")),
            };

            let non_auth: Vec<_> = cookies.iter().filter(|cookie| !cookie.is_auth_likely).collect();
            let auth_count = cookies.len() - non_auth.len();
            let mut results = Vec::new();

            if !non_auth.is_empty() {
                let export_path = dest_path.join(format!("ubs_cookie_metadata_{}.json", Uuid::new_v4()));
                if !request.dry_run {
                    let json = serde_json::to_vec_pretty(&non_auth)?;
                    write_new_export(&export_path, &json)?;
                }
                results.push(MigrationItemResult {
                    category: "Cookies".to_string(),
                    item_key: "cookie_metadata".to_string(),
                    label: format!("{} non-authentication cookie records", non_auth.len()),
                    status: (if request.dry_run { "planned" } else { "exported" }).to_string(),
                    message: Some(if request.dry_run {
                        "Dry run: non-authentication cookie metadata would be exported; cookie values are never copied".to_string()
                    } else {
                        format!("Metadata only exported to {}; cookie values and active sessions are not copied", export_path.display())
                    }),
                });
            }

            if auth_count > 0 {
                results.push(MigrationItemResult {
                    category: "Cookies".to_string(),
                    item_key: "auth_cookies".to_string(),
                    label: format!("{} authentication-like cookies", auth_count),
                    status: "skipped".to_string(),
                    message: Some("Only metadata is detected. Authentication values and active sessions are not extracted or transferred; sign in again on destination sites".to_string()),
                });
            }
            if results.is_empty() {
                results.push(MigrationItemResult {
                    category: "Cookies".to_string(),
                    item_key: "cookies_empty".to_string(),
                    label: "No cookies found".to_string(),
                    status: if request.dry_run { "planned" } else { "skipped" }.to_string(),
                    message: Some("No cookie records were found in the selected profile".to_string()),
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
            if extensions.is_empty() {
                return Ok(vec![MigrationItemResult {
                    category: "Extensions".to_string(),
                    item_key: "extensions_empty".to_string(),
                    label: "No installed extensions found".to_string(),
                    status: if request.dry_run { "planned" } else { "skipped" }.to_string(),
                    message: Some("No extension entries were discovered in the selected profile".to_string()),
                }]);
            }

            let same_family = request.source_browser.is_chromium_based()
                && request.dest_browser.is_chromium_based();
            let export_path = dest_path.join(format!("ubs_extensions_{}.json", Uuid::new_v4()));
            if !request.dry_run {
                let json = serde_json::to_vec_pretty(&extensions)?;
                write_new_export(&export_path, &json)?;
            }
            Ok(vec![MigrationItemResult {
                category: "Extensions".to_string(),
                item_key: "extensions_manifest".to_string(),
                label: format!("{} installed extensions", extensions.len()),
                status: (if request.dry_run { "planned" } else if same_family { "exported" } else { "incompatible" }).to_string(),
                message: Some(if request.dry_run {
                    "Dry run: extension inventory would be exported; extensions are not installed automatically".to_string()
                } else if same_family {
                    format!("Extension inventory saved to {}; install each extension in the destination browser", export_path.display())
                } else {
                    format!("Extension inventory saved to {}, but browser extension formats may differ; install compatible alternatives manually", export_path.display())
                }),
            }])
        }

        DataCategory::Preferences => {
            let prefs_path = source_path.join("Preferences");
            let ff_prefs = source_path.join("prefs.js");
            let source_file = if prefs_path.is_file() {
                Some((&prefs_path, "json"))
            } else if ff_prefs.is_file() {
                Some((&ff_prefs, "js"))
            } else {
                None
            };

            let Some((source_file, extension)) = source_file else {
                return Ok(vec![MigrationItemResult {
                    category: "Preferences".to_string(),
                    item_key: "preferences_missing".to_string(),
                    label: "No supported preferences file found".to_string(),
                    status: if request.dry_run { "planned" } else { "skipped" }.to_string(),
                    message: Some("The selected profile does not contain a supported Preferences or prefs.js file".to_string()),
                }]);
            };

            let export_path = dest_path.join(format!("ubs_preferences_{}.{}", Uuid::new_v4(), extension));
            if !request.dry_run {
                let bytes = std::fs::read(source_file)?;
                write_new_export(&export_path, &bytes)?;
            }
            Ok(vec![MigrationItemResult {
                category: "Preferences".to_string(),
                item_key: "preferences".to_string(),
                label: "Browser Preferences".to_string(),
                status: (if request.dry_run { "planned" } else { "exported" }).to_string(),
                message: Some(if request.dry_run {
                    "Dry run: preferences would be copied to a separate reference file; settings will not be applied automatically".to_string()
                } else {
                    format!("Preferences copied to {}; they have not been applied to the destination browser", export_path.display())
                }),
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

/// Create an export file without overwriting an existing user file.
fn write_new_export(path: &std::path::Path, bytes: &[u8]) -> anyhow::Result<()> {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .with_context(|| format!("Could not create export file {}", path.display()))?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn write_bookmarks_to_dest(
    bookmarks: &[crate::adapters::Bookmark],
    dest_path: &std::path::Path,
    dest_browser: &BrowserKind,
) -> anyhow::Result<()> {
    let html = crate::commands::bookmarks::bookmarks_to_netscape_html(bookmarks);
    let export_path = dest_path.join(format!("ubs_bookmarks_import_{}.html", Uuid::new_v4()));
    write_new_export(&export_path, html.as_bytes())?;
    let _ = dest_browser; // Format is intentionally browser-neutral Netscape HTML.
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
    {
        let state_map = migration_state().lock().unwrap();
        if let Some(status) = state_map.get(&run_id) {
            return Ok(Some(status.report.clone()));
        }
    }

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
