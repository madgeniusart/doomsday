use super::{
    BrowserKind, InstalledBrowser, BrowserProfile,
    chrome::ChromeAdapter,
    firefox::FirefoxAdapter,
    edge::EdgeAdapter,
    brave::BraveAdapter,
    opera::OperaAdapter,
    safari::SafariAdapter,
};
use anyhow::Result;
use std::path::PathBuf;

/// Detects installed browsers without hardcoding paths — uses platform conventions
pub struct BrowserDetector;

impl BrowserDetector {
    pub fn detect_all() -> Vec<InstalledBrowser> {
        let kinds = [
            BrowserKind::Chrome,
            BrowserKind::Firefox,
            BrowserKind::Edge,
            BrowserKind::Brave,
            BrowserKind::Opera,
            BrowserKind::Safari,
        ];

        kinds
            .iter()
            .filter_map(|kind| Self::detect_browser(kind).ok().flatten())
            .collect()
    }

    fn detect_browser(kind: &BrowserKind) -> Result<Option<InstalledBrowser>> {
        let executable = Self::find_executable(kind);

        match kind {
            BrowserKind::Chrome => {
                let adapter = ChromeAdapter::new();
                match adapter.enumerate_profiles() {
                    Ok(profiles) if !profiles.is_empty() || executable.is_some() => {
                        Ok(Some(InstalledBrowser {
                            kind: kind.clone(),
                            display_name: kind.display_name().to_string(),
                            version: executable.as_ref().and_then(|e| get_version(e)),
                            executable_path: executable,
                            profiles,
                            is_running: is_process_running("chrome") || is_process_running("Google Chrome"),
                        }))
                    }
                    Ok(_) => Ok(None),
                    Err(e) => {
                        log::debug!("Chrome detection failed: {e}");
                        Ok(None)
                    }
                }
            }
            BrowserKind::Firefox => {
                let adapter = FirefoxAdapter::new();
                match adapter.enumerate_profiles() {
                    Ok(profiles) if !profiles.is_empty() || executable.is_some() => {
                        Ok(Some(InstalledBrowser {
                            kind: kind.clone(),
                            display_name: kind.display_name().to_string(),
                            version: executable.as_ref().and_then(|e| get_version(e)),
                            executable_path: executable,
                            profiles,
                            is_running: is_process_running("firefox"),
                        }))
                    }
                    Ok(_) => Ok(None),
                    Err(e) => {
                        log::debug!("Firefox detection failed: {e}");
                        Ok(None)
                    }
                }
            }
            BrowserKind::Edge => {
                let adapter = EdgeAdapter::new();
                match adapter.enumerate_profiles() {
                    Ok(profiles) if !profiles.is_empty() || executable.is_some() => {
                        Ok(Some(InstalledBrowser {
                            kind: kind.clone(),
                            display_name: kind.display_name().to_string(),
                            version: None,
                            executable_path: executable,
                            profiles,
                            is_running: is_process_running("msedge") || is_process_running("Microsoft Edge"),
                        }))
                    }
                    Ok(_) => Ok(None),
                    Err(e) => {
                        log::debug!("Edge detection failed: {e}");
                        Ok(None)
                    }
                }
            }
            BrowserKind::Brave => {
                let adapter = BraveAdapter::new();
                match adapter.enumerate_profiles() {
                    Ok(profiles) if !profiles.is_empty() || executable.is_some() => {
                        Ok(Some(InstalledBrowser {
                            kind: kind.clone(),
                            display_name: kind.display_name().to_string(),
                            version: None,
                            executable_path: executable,
                            profiles,
                            is_running: is_process_running("brave") || is_process_running("Brave Browser"),
                        }))
                    }
                    Ok(_) => Ok(None),
                    Err(e) => {
                        log::debug!("Brave detection failed: {e}");
                        Ok(None)
                    }
                }
            }
            BrowserKind::Opera => {
                let adapter = OperaAdapter::new();
                match adapter.enumerate_profiles() {
                    Ok(profiles) if !profiles.is_empty() || executable.is_some() => {
                        Ok(Some(InstalledBrowser {
                            kind: kind.clone(),
                            display_name: kind.display_name().to_string(),
                            version: None,
                            executable_path: executable,
                            profiles,
                            is_running: is_process_running("opera"),
                        }))
                    }
                    Ok(_) => Ok(None),
                    Err(e) => {
                        log::debug!("Opera detection failed: {e}");
                        Ok(None)
                    }
                }
            }
            BrowserKind::Safari => {
                #[cfg(target_os = "macos")]
                {
                    let adapter = SafariAdapter::new();
                    match adapter.enumerate_profiles() {
                        Ok(profiles) if !profiles.is_empty() => {
                            return Ok(Some(InstalledBrowser {
                                kind: kind.clone(),
                                display_name: kind.display_name().to_string(),
                                version: None,
                                executable_path: executable,
                                profiles,
                                is_running: is_process_running("Safari"),
                            }));
                        }
                        _ => {}
                    }
                }
                Ok(None)
            }
        }
    }

    /// Locate browser executable using platform-appropriate search paths
    fn find_executable(kind: &BrowserKind) -> Option<PathBuf> {
        #[cfg(target_os = "linux")]
        return Self::find_executable_linux(kind);

        #[cfg(target_os = "macos")]
        return Self::find_executable_macos(kind);

        #[cfg(target_os = "windows")]
        return Self::find_executable_windows(kind);
    }

    #[cfg(target_os = "linux")]
    fn find_executable_linux(kind: &BrowserKind) -> Option<PathBuf> {
        let candidates: &[&str] = match kind {
            BrowserKind::Chrome => &["google-chrome-stable", "google-chrome", "chromium-browser", "chromium"],
            BrowserKind::Firefox => &["firefox", "firefox-esr"],
            BrowserKind::Edge => &["microsoft-edge-stable", "microsoft-edge"],
            BrowserKind::Brave => &["brave-browser-stable", "brave-browser", "brave"],
            BrowserKind::Opera => &["opera", "opera-stable"],
            BrowserKind::Safari => &[],
        };
        candidates.iter().find_map(|name| which(name))
    }

    #[cfg(target_os = "macos")]
    fn find_executable_macos(kind: &BrowserKind) -> Option<PathBuf> {
        let app_paths: &[&str] = match kind {
            BrowserKind::Chrome => &[
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
                "~/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            ],
            BrowserKind::Firefox => &[
                "/Applications/Firefox.app/Contents/MacOS/firefox",
                "~/Applications/Firefox.app/Contents/MacOS/firefox",
            ],
            BrowserKind::Edge => &[
                "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            ],
            BrowserKind::Brave => &[
                "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
            ],
            BrowserKind::Opera => &["/Applications/Opera.app/Contents/MacOS/Opera"],
            BrowserKind::Safari => &["/Applications/Safari.app/Contents/MacOS/Safari"],
        };
        app_paths
            .iter()
            .map(|p| PathBuf::from(shellexpand::tilde(p).as_ref()))
            .find(|p| p.exists())
    }

    #[cfg(target_os = "windows")]
    fn find_executable_windows(kind: &BrowserKind) -> Option<PathBuf> {
        // Read from registry HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths
        // Fallback to well-known ProgramFiles locations
        let program_files = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
        let program_files_x86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();

        let candidates: Vec<PathBuf> = match kind {
            BrowserKind::Chrome => vec![
                PathBuf::from(&local_app_data).join("Google\\Chrome\\Application\\chrome.exe"),
                PathBuf::from(&program_files).join("Google\\Chrome\\Application\\chrome.exe"),
                PathBuf::from(&program_files_x86).join("Google\\Chrome\\Application\\chrome.exe"),
            ],
            BrowserKind::Firefox => vec![
                PathBuf::from(&program_files).join("Mozilla Firefox\\firefox.exe"),
                PathBuf::from(&program_files_x86).join("Mozilla Firefox\\firefox.exe"),
            ],
            BrowserKind::Edge => vec![
                PathBuf::from(&program_files).join("Microsoft\\Edge\\Application\\msedge.exe"),
                PathBuf::from(&program_files_x86).join("Microsoft\\Edge\\Application\\msedge.exe"),
            ],
            BrowserKind::Brave => vec![
                PathBuf::from(&local_app_data).join("BraveSoftware\\Brave-Browser\\Application\\brave.exe"),
            ],
            BrowserKind::Opera => vec![
                PathBuf::from(&local_app_data).join("Programs\\Opera\\opera.exe"),
            ],
            BrowserKind::Safari => vec![], // No Safari on Windows
        };

        candidates.into_iter().find(|p| p.exists())
    }
}

/// Try to find a binary in PATH
#[cfg(target_os = "linux")]
fn which(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in path_var.split(':') {
        let candidate = PathBuf::from(dir).join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// Get version from binary --version output
fn get_version(exe: &PathBuf) -> Option<String> {
    std::process::Command::new(exe)
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Check if a browser process is running (best-effort)
fn is_process_running(name: &str) -> bool {
    #[cfg(unix)]
    {
        std::process::Command::new("pgrep")
            .args(["-x", name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        std::process::Command::new("tasklist")
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .to_lowercase()
                    .contains(&name.to_lowercase())
            })
            .unwrap_or(false)
    }
}
