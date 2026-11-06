use crate::{BrowserKind, CookieInfo};
use crate::adapters::{
    chrome::ChromeAdapter, firefox::FirefoxAdapter, edge::EdgeAdapter, 
    brave::BraveAdapter, opera::OperaAdapter, safari::SafariAdapter
};




use std::path::PathBuf;

#[tauri::command]
pub async fn read_cookies(
    browser: BrowserKind,
    profile_path: String,
) -> Result<Vec<CookieInfo>, String> {
    use crate::adapters::*;
    let path = PathBuf::from(&profile_path);

    let result = match browser {
        BrowserKind::Chrome => ChromeAdapter::new().read_cookies(&path),
        BrowserKind::Firefox => FirefoxAdapter::new().read_cookies(&path),
        BrowserKind::Edge => EdgeAdapter::new().read_cookies(&path),
        BrowserKind::Brave => BraveAdapter::new().read_cookies(&path),
        BrowserKind::Opera => OperaAdapter::new().read_cookies(&path),
        BrowserKind::Safari => SafariAdapter::new().read_cookies(&path),
    };

    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_cookies(
    browser: BrowserKind,
    profile_path: String,
    output_path: String,
    include_auth: bool,
) -> Result<String, String> {
    use crate::adapters::*;
    let path = PathBuf::from(&profile_path);

    let cookies = match browser {
        BrowserKind::Chrome => ChromeAdapter::new().read_cookies(&path),
        BrowserKind::Firefox => FirefoxAdapter::new().read_cookies(&path),
        BrowserKind::Edge => EdgeAdapter::new().read_cookies(&path),
        BrowserKind::Brave => BraveAdapter::new().read_cookies(&path),
        BrowserKind::Opera => OperaAdapter::new().read_cookies(&path),
        BrowserKind::Safari => SafariAdapter::new().read_cookies(&path),
    }
    .map_err(|e| e.to_string())?;

    let filtered: Vec<&CookieInfo> = if include_auth {
        cookies.iter().collect()
    } else {
        cookies.iter().filter(|c| !c.is_auth_likely).collect()
    };

    let json = serde_json::to_string_pretty(&filtered).map_err(|e| e.to_string())?;
    std::fs::write(&output_path, json).map_err(|e| e.to_string())?;

    Ok(format!("Exported {} cookies", filtered.len()))
}
