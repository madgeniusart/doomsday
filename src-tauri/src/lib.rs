use tauri::Manager;

mod adapters;
mod commands;
mod database;
mod encryption;
mod migration;

pub use adapters::*;
pub use database::AppDatabase;

/// Application state shared across Tauri commands
pub struct AppState {
    pub app_data_dir: std::path::PathBuf,
    pub db: std::sync::Mutex<AppDatabase>,
    pub encryption: std::sync::Mutex<encryption::EncryptionService>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir)?;

            let db_path = app_dir.join("universal_browser_sync.db");
            let db = AppDatabase::new(&db_path).expect("failed to open database");

            let enc = encryption::EncryptionService::new()
                .expect("failed to initialize encryption service");

            app.manage(AppState {
                app_data_dir: app_dir.clone(),
                db: std::sync::Mutex::new(db),
                encryption: std::sync::Mutex::new(enc),
            });

            log::info!("Universal Browser Sync initialized, db at: {}", db_path.display());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Browser detection
            commands::browser::detect_browsers,
            commands::browser::get_browser_profiles,
            commands::browser::verify_profile_access,
            // Migration
            commands::migration::start_migration,
            commands::migration::get_migration_status,
            commands::migration::cancel_migration,
            commands::migration::get_migration_history,
            commands::migration::get_migration_report,
            // Bookmarks
            commands::bookmarks::read_bookmarks,
            commands::bookmarks::export_bookmarks,
            // History
            commands::history::read_history,
            // Cookies
            commands::cookies::read_cookies,
            commands::cookies::export_cookies,
            // Extensions
            commands::extensions::read_extensions,
            commands::extensions::get_compatible_extensions,
            // Backup
            commands::backup::create_backup,
            commands::backup::restore_backup,
            commands::backup::list_backups,
            commands::backup::delete_backup,
            // Settings
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::credentials::import_password_csv,
            commands::credentials::list_password_imports,
            commands::credentials::export_password_csv,
            commands::bridge::drain_extension_events,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
