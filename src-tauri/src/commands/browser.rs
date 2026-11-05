use crate::adapters::{
    chrome::ChromeAdapter, firefox::FirefoxAdapter, edge::EdgeAdapter, 
    brave::BraveAdapter, opera::OperaAdapter, safari::SafariAdapter
};

use tauri::State;
use crate::AppState;

#[tauri::command]
pub async fn detect_browsers() -> Result<Vec<InstalledBrowser>, String> {
    log::info!("Detecting installed browsers");
    let browsers = BrowserDetector::detect_all();
    log::info!("Found {} browser(s)", browsers.len());
    Ok(browsers)
}

#[tauri::command]
pub async fn get_browser_profiles(
    browser: BrowserKind,
) -> Result<Vec<BrowserProfile>, String> {
    use crate::adapters::*;
    let profiles = match browser {
        BrowserKind::Chrome => ChromeAdapter::new().enumerate_profiles(),
        BrowserKind::Firefox => FirefoxAdapter::new().enumerate_profiles(),
        BrowserKind::Edge => EdgeAdapter::new().enumerate_profiles(),
        BrowserKind::Brave => BraveAdapter::new().enumerate_profiles(),
        BrowserKind::Opera => OperaAdapter::new().enumerate_profiles(),
        BrowserKind::Safari => SafariAdapter::new().enumerate_profiles(),
    }
    .map_err(|e| e.to_string())?;

    Ok(profiles)
}

#[tauri::command]
pub async fn verify_profile_access(
    profile_path: String,
    _state: State<'_, AppState>,
) -> Result<bool, String> {
    let path = std::path::PathBuf::from(&profile_path);
    if !path.exists() {
        return Ok(false);
    }
    // Try to read directory listing as access check
    match std::fs::read_dir(&path) {
        Ok(_) => Ok(true),
        Err(e) => {
            log::warn!("Cannot access profile at {}: {}", profile_path, e);
            Ok(false)
        }
    }
}
