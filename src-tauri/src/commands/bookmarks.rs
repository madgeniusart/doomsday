use crate::adapters::{
    chrome::ChromeAdapter, firefox::FirefoxAdapter, edge::EdgeAdapter, 
    brave::BraveAdapter, opera::OperaAdapter, safari::SafariAdapter
};

use std::path::PathBuf;

fn get_adapter(browser: &BrowserKind) -> Box<dyn BrowserAdapter> {
    use crate::adapters::*;
    match browser {
        BrowserKind::Chrome => Box::new(ChromeAdapter::new()),
        BrowserKind::Firefox => Box::new(FirefoxAdapter::new()),
        BrowserKind::Edge => Box::new(EdgeAdapter::new()),
        BrowserKind::Brave => Box::new(BraveAdapter::new()),
        BrowserKind::Opera => Box::new(OperaAdapter::new()),
        BrowserKind::Safari => Box::new(SafariAdapter::new()),
    }
}

#[tauri::command]
pub async fn read_bookmarks(
    browser: BrowserKind,
    profile_path: String,
) -> Result<Vec<Bookmark>, String> {
    let path = PathBuf::from(&profile_path);
    let adapter = get_adapter(&browser);
    adapter.read_bookmarks(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_bookmarks(
    browser: BrowserKind,
    profile_path: String,
    output_path: String,
    format: String, // "html" | "json"
) -> Result<String, String> {
    let path = PathBuf::from(&profile_path);
    let adapter = get_adapter(&browser);
    let bookmarks = adapter.read_bookmarks(&path).map_err(|e| e.to_string())?;

    let output = PathBuf::from(&output_path);

    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&bookmarks).map_err(|e| e.to_string())?;
            std::fs::write(&output, json).map_err(|e| e.to_string())?;
        }
        "html" | _ => {
            let html = bookmarks_to_netscape_html(&bookmarks);
            std::fs::write(&output, html).map_err(|e| e.to_string())?;
        }
    }

    Ok(format!("Exported {} bookmarks", bookmarks.len()))
}

/// Export bookmarks in Netscape HTML format (importable by all browsers)
pub fn bookmarks_to_netscape_html(bookmarks: &[Bookmark]) -> String {
    let mut html = String::from(
        "<!DOCTYPE NETSCAPE-Bookmark-file-1>\n\
         <!-- This is an automatically generated file. Do not edit. -->\n\
         <META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n\
         <TITLE>Bookmarks</TITLE>\n\
         <H1>Bookmarks Menu</H1>\n\
         <DL><p>\n",
    );

    for bm in bookmarks {
        if let Some(url) = &bm.url {
            let add_date = bm.date_added.unwrap_or(0);
            html.push_str(&format!(
                "    <DT><A HREF=\"{}\" ADD_DATE=\"{}\">{}</A>\n",
                escape_html(url),
                add_date,
                escape_html(&bm.title)
            ));
        }
    }

    html.push_str("</DL><p>\n");
    html
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
