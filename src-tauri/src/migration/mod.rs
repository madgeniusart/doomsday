//! Migration engine — coordinates data transfer between browser adapters.
//! This module contains the higher-level migration orchestration logic.

use crate::adapters::{BrowserKind, DataCategory};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub source_browser: BrowserKind,
    pub dest_browser: BrowserKind,
    pub categories: Vec<CategoryCompatibility>,
    pub warnings: Vec<String>,
    pub sites_requiring_relogin: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCompatibility {
    pub category: DataCategory,
    pub supported: bool,
    pub notes: Option<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Safe,
    Low,
    Medium,
    High,
}

/// Analyze compatibility between source and destination browsers
pub fn analyze_compatibility(
    source: &BrowserKind,
    dest: &BrowserKind,
) -> CompatibilityReport {
    let mut categories = Vec::new();
    let mut warnings = Vec::new();

    let source_chromium = source.is_chromium_based();
    let dest_chromium = dest.is_chromium_based();

    // Bookmarks
    let bookmark_notes = if dest_chromium {
        "Imported into the destination Chromium bookmark database after the browser is closed; a timestamped backup is created first."
    } else {
        "Exported as Netscape HTML into the destination profile; use the destination browser's Import Bookmarks command to finish."
    };
    categories.push(CategoryCompatibility {
        category: DataCategory::Bookmarks,
        supported: true,
        notes: Some(bookmark_notes.to_string()),
        risk_level: RiskLevel::Safe,
    });

    // History
    let history_notes = if source_chromium && dest_chromium {
        "History exported as a JSON reference; it is not imported into the destination browser."
    } else {
        "History format differs. A JSON reference export is created; native import is not implemented."
    };
    categories.push(CategoryCompatibility {
        category: DataCategory::History,
        supported: true,
        notes: Some(history_notes.to_string()),
        risk_level: RiskLevel::Low,
    });

    // Cookies
    categories.push(CategoryCompatibility {
        category: DataCategory::Cookies,
        supported: !matches!(dest, BrowserKind::Safari),
        notes: Some("Cookie metadata can be inventoried and exported, but cookie values and active authentication sessions are not transferred. Some sites may require sign-in again.".to_string()),
        risk_level: RiskLevel::Medium,
    });

    // Extensions
    let ext_compatible = source_chromium && dest_chromium;
    categories.push(CategoryCompatibility {
        category: DataCategory::Extensions,
        supported: true,
        notes: if ext_compatible {
            Some("Extension details are listed for manual installation; extensions are not installed automatically.".to_string())
        } else {
            Some("Extension format differs; equivalent extensions may need to be installed manually.".to_string())
        },
        risk_level: if ext_compatible { RiskLevel::Low } else { RiskLevel::Medium },
    });

    // Preferences
    categories.push(CategoryCompatibility {
        category: DataCategory::Preferences,
        supported: true,
        notes: Some("Preferences are exported as a reference file; settings are not applied automatically to the destination browser.".to_string()),
        risk_level: RiskLevel::Low,
    });

    // Platform warnings
    #[cfg(target_os = "macos")]
    if matches!(source, BrowserKind::Safari) || matches!(dest, BrowserKind::Safari) {
        warnings.push("Safari requires Full Disk Access permission. Enable it in System Settings → Privacy & Security → Full Disk Access.".to_string());
    }

    if source == dest {
        warnings.push("Source and destination browsers are the same. Consider selecting different profiles.".to_string());
    }

    // Sites that will require re-login
    let sites_requiring_relogin = vec![
        "All sites using session cookies".to_string(),
        "Sites with browser-fingerprint authentication".to_string(),
    ];

    CompatibilityReport {
        source_browser: source.clone(),
        dest_browser: dest.clone(),
        categories,
        warnings,
        sites_requiring_relogin,
    }
}
