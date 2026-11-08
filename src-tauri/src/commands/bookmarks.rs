use crate::{BrowserKind, BrowserAdapter, Bookmark};
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

/// Export bookmarks in Netscape HTML format (importable by all browsers).
/// Preserve folder hierarchy and reject executable/non-web URL schemes.
pub fn bookmarks_to_netscape_html(bookmarks: &[Bookmark]) -> String {
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct Folder {
        folders: BTreeMap<String, Folder>,
        bookmarks: Vec<Bookmark>,
    }

    fn render_folder(folder: &Folder, output: &mut String, depth: usize) {
        let indent = "    ".repeat(depth);
        for (name, child) in &folder.folders {
            output.push_str(&format!("{indent}<DT><H3>{}</H3>\n{indent}<DL><p>\n", escape_html(name)));
            render_folder(child, output, depth + 1);
            output.push_str(&format!("{indent}</DL><p>\n"));
        }
        for bookmark in &folder.bookmarks {
            let Some(url) = bookmark.url.as_deref() else { continue };
            let lower = url.trim().to_ascii_lowercase();
            if !(lower.starts_with("https://") || lower.starts_with("http://") || lower.starts_with("ftp://")) {
                continue;
            }
            let add_date = bookmark.date_added.unwrap_or(0);
            output.push_str(&format!(
                "{indent}<DT><A HREF=\"{}\" ADD_DATE=\"{}\">{}</A>\n",
                escape_html(url), add_date, escape_html(&bookmark.title)
            ));
        }
    }

    let mut root = Folder::default();
    for bookmark in bookmarks {
        let mut folder = &mut root;
        for segment in &bookmark.folder_path {
            folder = folder.folders.entry(segment.clone()).or_default();
        }
        folder.bookmarks.push(bookmark.clone());
    }

    let mut html = String::from(
        "<!DOCTYPE NETSCAPE-Bookmark-file-1>\n\
         <!-- This is an automatically generated file. Do not edit. -->\n\
         <META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n\
         <TITLE>Bookmarks</TITLE>\n\
         <H1>Bookmarks Menu</H1>\n\
         <DL><p>\n",
    );
    render_folder(&root, &mut html, 1);
    html.push_str("</DL><p>\n");
    html
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::bookmarks_to_netscape_html;
    use crate::adapters::Bookmark;

    #[test]
    fn netscape_export_preserves_folders_and_escapes_content() {
        let bookmarks = vec![Bookmark {
            id: "1".into(),
            title: "A & B <docs>".into(),
            url: Some("https://example.com/?a=1&b=2".into()),
            folder_path: vec!["Bookmarks Bar".into(), "Work".into()],
            date_added: Some(123),
            date_modified: None,
            children: vec![],
        }];
        let html = bookmarks_to_netscape_html(&bookmarks);
        assert!(html.contains("<H3>Bookmarks Bar</H3>"));
        assert!(html.contains("<H3>Work</H3>"));
        assert!(html.contains("A &amp; B &lt;docs&gt;"));
        assert!(html.contains("https://example.com/?a=1&amp;b=2"));
    }

    #[test]
    fn netscape_export_rejects_executable_url_schemes() {
        let bookmarks = vec![Bookmark {
            id: "2".into(),
            title: "Do not import".into(),
            url: Some("javascript:alert(1)".into()),
            folder_path: vec![],
            date_added: None,
            date_modified: None,
            children: vec![],
        }];
        let html = bookmarks_to_netscape_html(&bookmarks);
        assert!(!html.contains("javascript:"));
        assert!(!html.contains("Do not import"));
    }
}
