//! User-mediated password CSV import. Password material is encrypted immediately
//! with the app's OS-keychain-backed AES-GCM key and is never logged or returned.
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::PathBuf};
use tauri::State;
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

const MAX_CSV_BYTES: usize = 10 * 1024 * 1024;
const MAX_ROWS: usize = 100_000;

#[derive(Debug, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
struct PasswordEntry {
    name: String,
    url: String,
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct PasswordImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub encrypted_file: String,
}

#[derive(Debug, Serialize)]
pub struct PasswordImportSummary {
    pub file: String,
    pub created_at: String,
    pub encrypted: bool,
}

/// Import a browser-exported CSV chosen explicitly by the user in the UI.
/// The CSV contents are encrypted before they touch persistent storage.
#[tauri::command]
pub async fn import_password_csv(
    csv_path: String,
    state: State<'_, AppState>,
) -> Result<PasswordImportResult, String> {
    let input_path = PathBuf::from(&csv_path);
    if input_path.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("csv")) != Some(true) {
        return Err("Choose a .csv password export file".into());
    }
    let metadata = fs::metadata(&input_path).map_err(|e| format!("Could not read selected CSV: {e}"))?;
    if !metadata.is_file() { return Err("Selected password export is not a regular file".into()); }
    if metadata.len() as usize > MAX_CSV_BYTES {
        return Err("Password CSV exceeds the 10 MiB import limit".into());
    }
    let mut csv_text = Zeroizing::new(fs::read_to_string(&input_path).map_err(|e| format!("Password CSV must be valid UTF-8: {e}"))?);
    if csv_text.starts_with('\u{feff}') { csv_text.replace_range(..3, ""); }
    if csv_text.trim().is_empty() {
        return Err("The selected password CSV is empty".into());
    }

    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(csv_text.as_bytes());
    let headers = reader.headers().map_err(|e| format!("Invalid CSV header: {e}"))?.clone();
    let index = |names: &[&str]| -> Option<usize> {
        headers.iter().position(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)))
    };
    let url_i = index(&["url", "origin", "website"])
        .ok_or("CSV must contain a url column")?;
    let username_i = index(&["username", "user name", "login"])
        .ok_or("CSV must contain a username column")?;
    let password_i = index(&["password", "pass"])
        .ok_or("CSV must contain a password column")?;
    let name_i = index(&["name", "title", "label"]);

    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    let mut skipped = 0usize;
    for (row_number, row) in reader.records().enumerate() {
        if row_number >= MAX_ROWS {
            return Err("Password CSV exceeds the 100,000 row import limit".into());
        }
        let row = row.map_err(|e| format!("Invalid CSV row {}: {e}", row_number + 2))?;
        let get = |i: usize| row.get(i).unwrap_or("").trim();
        let url = get(url_i).to_string();
        let username = get(username_i).to_string();
        let password = get(password_i).to_string();
        let parsed = match url::Url::parse(&url) {
            Ok(v) if matches!(v.scheme(), "http" | "https") => v,
            _ => { skipped += 1; continue; }
        };
        if password.is_empty() || username.is_empty() {
            skipped += 1;
            continue;
        }
        let key = format!("{}\0{}", parsed.as_str().to_lowercase(), username.to_lowercase());
        if !seen.insert(key) {
            skipped += 1;
            continue;
        }
        entries.push(PasswordEntry {
            name: name_i.map(|i| get(i).to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| parsed.host_str().unwrap_or("Saved login").to_string()),
            url,
            username,
            password,
        });
    }
    if entries.is_empty() {
        return Err("No valid username/password rows were found in the CSV".into());
    }

    let imported = entries.len();
    let payload = Zeroizing::new(serde_json::to_vec(&entries).map_err(|e| e.to_string())?);
    let encrypted = {
        let enc = state.encryption.lock().map_err(|_| "Encryption service lock poisoned".to_string())?;
        enc.encrypt(payload.as_slice()).map_err(|e| e.to_string())?
    };
    let vault_dir = state.app_data_dir.join("credential-imports");
    fs::create_dir_all(&vault_dir).map_err(|e| format!("Could not create encrypted credential vault: {e}"))?;
    let file_name = format!("password-import-{}.ubsenc", Uuid::new_v4());
    let final_path = vault_dir.join(file_name);
    let temp_path = final_path.with_extension("tmp");
    fs::write(&temp_path, &encrypted).map_err(|e| format!("Could not write encrypted credentials: {e}"))?;
    fs::rename(&temp_path, &final_path).map_err(|e| {
        let _ = fs::remove_file(&temp_path);
        format!("Could not finalize encrypted credential vault: {e}")
    })?;

    // Zeroizing wrappers and PasswordEntry::drop clear transient secret buffers.
    drop(entries);

    Ok(PasswordImportResult {
        imported,
        skipped,
        encrypted_file: final_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub async fn list_password_imports(state: State<'_, AppState>) -> Result<Vec<PasswordImportSummary>, String> {
    let dir: PathBuf = state.app_data_dir.join("credential-imports");
    if !dir.exists() { return Ok(Vec::new()); }
    let mut items = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_file() { continue; }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("ubsenc") { continue; }
        let created_at = entry.metadata().ok().and_then(|m| m.modified().ok())
            .map(chrono::DateTime::<chrono::Utc>::from).map(|d| d.to_rfc3339()).unwrap_or_default();
        items.push(PasswordImportSummary {
            file: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            created_at,
            encrypted: true,
        });
    }
    items.sort_by(|a,b| b.created_at.cmp(&a.created_at));
    Ok(items)
}

/// Decrypt staged passwords only when the user explicitly chooses an output path.
/// The resulting CSV is plaintext for the destination browser's built-in importer.
#[tauri::command]
pub async fn export_password_csv(
    output_path: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let output = PathBuf::from(&output_path);
    if output.file_name().is_none() {
        return Err("Choose a valid CSV output filename".into());
    }
    if output.exists() {
        return Err("The selected output file already exists; choose a new filename to avoid overwriting it".into());
    }
    let vault_dir = state.app_data_dir.join("credential-imports");
    if !vault_dir.exists() { return Err("No encrypted password imports are available".into()); }

    let mut entries: Vec<PasswordEntry> = Vec::new();
    for item in fs::read_dir(&vault_dir).map_err(|e| e.to_string())? {
        let item = item.map_err(|e| e.to_string())?;
        let path = item.path();
        if path.extension().and_then(|s| s.to_str()) != Some("ubsenc") { continue; }
        let encrypted = fs::read(&path).map_err(|e| e.to_string())?;
        let decrypted = {
            let enc = state.encryption.lock().map_err(|_| "Encryption service lock poisoned".to_string())?;
            Zeroizing::new(enc.decrypt(&encrypted).map_err(|e| format!("Could not decrypt {}: {e}", path.file_name().unwrap_or_default().to_string_lossy()))?)
        };
        let mut batch: Vec<PasswordEntry> = serde_json::from_slice(decrypted.as_slice()).map_err(|e| format!("Encrypted password file is invalid: {e}"))?;
        entries.append(&mut batch);
        if entries.len() > MAX_ROWS { return Err("Combined password vault exceeds the 100,000 login export limit".into()); }
    }
    if entries.is_empty() { return Err("No saved password entries were found".into()); }

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    }
    let temp = output.with_extension(format!("{}.tmp", Uuid::new_v4()));
    let write_result = (|| -> Result<(), String> {
        let file = fs::OpenOptions::new().write(true).create_new(true).open(&temp).map_err(|e| e.to_string())?;
        let mut writer = csv::WriterBuilder::new().from_writer(file);
        writer.write_record(["name", "url", "username", "password"]).map_err(|e| e.to_string())?;
        for entry in &entries {
            writer.write_record([entry.name.as_str(), entry.url.as_str(), entry.username.as_str(), entry.password.as_str()]).map_err(|e| e.to_string())?;
        }
        writer.flush().map_err(|e| e.to_string())?;
        let file = writer.into_inner().map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&temp, fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
        }
        fs::hard_link(&temp, &output).map_err(|e| format!("Could not create output without overwriting an existing file: {e}"))?;
        fs::remove_file(&temp).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    let count = entries.len();
    Ok(count)
}
