use crate::ExtensionInfo;
use crate::BrowserKind;
use crate::adapters::{
    chrome::ChromeAdapter, firefox::FirefoxAdapter, edge::EdgeAdapter, 
    brave::BraveAdapter, opera::OperaAdapter, safari::SafariAdapter
};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct CompatibleExtension {
    pub extension: ExtensionInfo,
    pub compatible_store_url: Option<String>,
    pub compatibility_note: Option<String>,
}

#[tauri::command]
pub async fn read_extensions(
    browser: BrowserKind,
    profile_path: String,
) -> Result<Vec<ExtensionInfo>, String> {
    use crate::adapters::*;
    let path = PathBuf::from(&profile_path);

    let result = match browser {
        BrowserKind::Chrome => ChromeAdapter::new().read_extensions(&path),
        BrowserKind::Firefox => FirefoxAdapter::new().read_extensions(&path),
        BrowserKind::Edge => EdgeAdapter::new().read_extensions(&path),
        BrowserKind::Brave => BraveAdapter::new().read_extensions(&path),
        BrowserKind::Opera => OperaAdapter::new().read_extensions(&path),
        BrowserKind::Safari => SafariAdapter::new().read_extensions(&path),
    };

    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_compatible_extensions(
    source_browser: BrowserKind,
    dest_browser: BrowserKind,
    extensions: Vec<ExtensionInfo>,
) -> Result<Vec<CompatibleExtension>, String> {
    let result = extensions
        .into_iter()
        .map(|ext| {
            let (compatible_store_url, compatibility_note) =
                check_cross_browser_compatibility(&source_browser, &dest_browser, &ext);
            CompatibleExtension {
                extension: ext,
                compatible_store_url,
                compatibility_note,
            }
        })
        .collect();

    Ok(result)
}

/// Determine if a Chromium extension works in another Chromium browser
fn check_cross_browser_compatibility(
    source: &BrowserKind,
    dest: &BrowserKind,
    ext: &ExtensionInfo,
) -> (Option<String>, Option<String>) {
    let source_chromium = source.is_chromium_based();
    let dest_chromium = dest.is_chromium_based();

    match (source_chromium, dest_chromium) {
        (true, true) => {
            // Same extension format; may be available on Chrome Web Store
            let store_url = Some(format!(
                "https://chrome.google.com/webstore/detail/{}",
                ext.id
            ));
            (store_url, Some("Extension should be compatible — install from Chrome Web Store".to_string()))
        }
        (true, false) if *dest == BrowserKind::Firefox => {
            // Some Chrome extensions have Firefox equivalents; no automatic mapping
            (
                Some(format!("https://addons.mozilla.org/en-US/firefox/search/?q={}", urlencoding::encode(&ext.name))),
                Some("Chrome extensions are not directly compatible with Firefox. Search for an equivalent on Firefox Add-ons.".to_string()),
            )
        }
        (false, true) if *source == BrowserKind::Firefox => {
            (
                Some(format!(
                    "https://chrome.google.com/webstore/search/{}",
                    urlencoding::encode(&ext.name)
                )),
                Some("Firefox extensions are not directly compatible with Chromium browsers. Search for an equivalent.".to_string()),
            )
        }
        _ => (
            None,
            Some("Extension compatibility cannot be determined automatically.".to_string()),
        ),
    }
}

/// Minimal URL encoding without external crate overhead
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut encoded = String::new();
        for c in s.chars() {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                    encoded.push(c);
                }
                ' ' => encoded.push('+'),
                c => {
                    for byte in c.to_string().as_bytes() {
                        encoded.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        encoded
    }
}
