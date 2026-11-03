use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub auto_backup: bool,
    pub backup_encryption: bool,
    pub consent_given: bool,
    pub max_history_days: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            auto_backup: true,
            backup_encryption: true,
            consent_given: false,
            max_history_days: 90,
        }
    }
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let db = state.db.lock().unwrap();

    let theme = db.get_setting("theme")
        .ok()
        .flatten()
        .and_then(|s| serde_json::from_str::<String>(&s).ok())
        .unwrap_or_else(|| "dark".to_string());

    let auto_backup = db.get_setting("auto_backup")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(true);

    let backup_encryption = db.get_setting("backup_encryption")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(true);

    let consent_given = db.get_setting("consent_given")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<bool>().ok())
        .unwrap_or(false);

    let max_history_days = db.get_setting("max_history_days")
        .ok()
        .flatten()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(90);

    Ok(AppSettings {
        theme,
        auto_backup,
        backup_encryption,
        consent_given,
        max_history_days,
    })
}

#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().unwrap();

    db.set_setting("theme", &format!("\"{}\"", settings.theme))
        .map_err(|e| e.to_string())?;
    db.set_setting("auto_backup", &settings.auto_backup.to_string())
        .map_err(|e| e.to_string())?;
    db.set_setting("backup_encryption", &settings.backup_encryption.to_string())
        .map_err(|e| e.to_string())?;
    db.set_setting("consent_given", &settings.consent_given.to_string())
        .map_err(|e| e.to_string())?;
    db.set_setting("max_history_days", &settings.max_history_days.to_string())
        .map_err(|e| e.to_string())?;

    Ok(())
}
