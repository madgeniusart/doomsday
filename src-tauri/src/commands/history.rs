use crate::adapters::{BrowserKind, HistoryEntry};
use std::path::PathBuf;

#[tauri::command]
pub async fn read_history(
    browser: BrowserKind,
    profile_path: String,
    max_days: Option<u32>,
) -> Result<Vec<HistoryEntry>, String> {
    use crate::adapters::*;
    let path = PathBuf::from(&profile_path);
    let days = max_days.unwrap_or(90);

    let result = match browser {
        BrowserKind::Chrome => ChromeAdapter::new().read_history(&path, days),
        BrowserKind::Firefox => FirefoxAdapter::new().read_history(&path, days),
        BrowserKind::Edge => EdgeAdapter::new().read_history(&path, days),
        BrowserKind::Brave => BraveAdapter::new().read_history(&path, days),
        BrowserKind::Opera => OperaAdapter::new().read_history(&path, days),
        BrowserKind::Safari => SafariAdapter::new().read_history(&path, days),
    };

    result.map_err(|e| e.to_string())
}
