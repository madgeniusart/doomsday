use crate::database::BackupRecord;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupRequest {
    pub browser: String,
    pub profile_path: String,
    pub output_dir: String,
    pub encrypt: bool,
}

#[tauri::command]
pub async fn create_backup(
    request: BackupRequest,
    state: State<'_, AppState>,
) -> Result<BackupRecord, String> {
    let backup_id = Uuid::new_v4().to_string();
    if request.browser.is_empty()
        || request.browser.len() > 32
        || !request.browser.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err("Browser identifier must contain only letters, numbers, hyphens, or underscores".to_string());
    }
    let profile_path = PathBuf::from(&request.profile_path);
    let output_dir = PathBuf::from(&request.output_dir);

    if !profile_path.is_dir() {
        return Err("Source profile directory does not exist or is not a directory".to_string());
    }
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    let canonical_profile = profile_path.canonicalize().map_err(|e| e.to_string())?;
    let canonical_output = output_dir.canonicalize().map_err(|e| e.to_string())?;
    if canonical_output.starts_with(&canonical_profile) {
        return Err("Backup output directory must be outside the browser profile".to_string());
    }

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!(
        "ubs_backup_{}_{}_{}.tar{}",
        request.browser,
        timestamp,
        &backup_id[..8],
        if request.encrypt { ".enc" } else { ".gz" }
    );
    let output_path = output_dir.join(&filename);

    // Create tar archive of profile
    let tar_data = create_tar_archive(&profile_path).map_err(|e| e.to_string())?;

    let output_data = if request.encrypt {
        let enc = state.encryption.lock().map_err(|_| "Encryption service lock poisoned".to_string())?;
        enc.encrypt(&tar_data).map_err(|e| e.to_string())?
    } else {
        // Compress with gzip
        use flate2::{write::GzEncoder, Compression};
        use std::io::Write;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&tar_data).map_err(|e| e.to_string())?;
        encoder.finish().map_err(|e| e.to_string())?
    };

    // Write to a unique temporary file and atomically publish the completed backup.
    let temp_path = output_path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    std::fs::write(&temp_path, &output_data).map_err(|e| e.to_string())?;
    if let Err(error) = std::fs::rename(&temp_path, &output_path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(format!("Could not finalize backup file: {error}"));
    }
    let size_bytes = std::fs::metadata(&output_path)
        .map_err(|e| e.to_string())?.len() as i64;

    let record = BackupRecord {
        id: backup_id,
        browser: request.browser,
        profile: profile_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        path: output_path.to_string_lossy().to_string(),
        size_bytes,
        created_at: chrono::Utc::now().to_rfc3339(),
        encrypted: request.encrypt,
    };

    let db = state.db.lock().map_err(|_| "Database lock poisoned".to_string())?;
    if let Err(error) = db.record_backup(&record) {
        let _ = std::fs::remove_file(&output_path);
        return Err(error.to_string());
    }

    Ok(record)
}

#[tauri::command]
pub async fn restore_backup(
    backup_path: String,
    output_dir: String,
    encrypted: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let backup_path = PathBuf::from(&backup_path);
    let output_dir = PathBuf::from(&output_dir);

    if !backup_path.exists() {
        return Err("Backup file not found".to_string());
    }

    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    let data = std::fs::read(&backup_path).map_err(|e| e.to_string())?;

    let tar_data = if encrypted {
        let enc = state.encryption.lock().map_err(|_| "Encryption service lock poisoned".to_string())?;
        enc.decrypt(&data).map_err(|e| e.to_string())?
    } else {
        use flate2::read::GzDecoder;
        use std::io::Read;
        let mut decoder = GzDecoder::new(data.as_slice());
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).map_err(|e| e.to_string())?;
        decompressed
    };

    extract_tar_archive(&tar_data, &output_dir).map_err(|e| e.to_string())?;

    Ok(format!("Backup restored to {}", output_dir.display()))
}

#[tauri::command]
pub async fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupRecord>, String> {
    let db = state.db.lock().unwrap();
    db.list_backups().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_backup(
    backup_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Get the path first
    let backups = {
        let db = state.db.lock().unwrap();
        db.list_backups().map_err(|e| e.to_string())?
    };

    if let Some(backup) = backups.iter().find(|b| b.id == backup_id) {
        let path = PathBuf::from(&backup.path);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
    }

    let db = state.db.lock().unwrap();
    db.delete_backup(&backup_id).map_err(|e| e.to_string())?;
    Ok(())
}

fn create_tar_archive(dir: &PathBuf) -> anyhow::Result<Vec<u8>> {
    use tar::Builder;
    if !dir.is_dir() {
        anyhow::bail!("Profile path is not a directory");
    }
    let mut archive = Builder::new(Vec::new());

    // Skip files that should not be backed up (lock files, cache, etc.)
    let skip_patterns = [
        "SingletonLock",
        "SingletonCookie",
        "SingletonSocket",
        "lock",
        ".tmp",
        "Cache",
        "cache",
        "GPUCache",
        "ShaderCache",
        "Code Cache",
    ];

    let walker = walkdir::WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            if entry.path() == dir || entry.file_type().is_symlink() {
                return true;
            }
            let name = entry.file_name().to_string_lossy();
            !skip_patterns.iter().any(|pattern| name == *pattern)
        });

    for entry in walker {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type().is_symlink() {
            continue;
        }
        if path.is_file() {
            let relative = path.strip_prefix(dir)?;
            let mut file = std::fs::File::open(path)
                .map_err(|e| anyhow::anyhow!("Could not read profile file {}: {e}", path.display()))?;
            archive.append_file(relative, &mut file)
                .map_err(|e| anyhow::anyhow!("Could not add profile file {} to backup: {e}", path.display()))?;
        }
    }

    Ok(archive.into_inner()?)
}

fn extract_tar_archive(data: &[u8], dest: &PathBuf) -> anyhow::Result<()> {
    use tar::Archive;
    let mut archive = Archive::new(data);
    for entry in archive.entries()? {
        let mut entry = entry?;
        // unpack_in rejects entries that escape the selected restore directory.
        if !entry.unpack_in(dest)? {
            anyhow::bail!("Backup contains an unsafe path and was not fully restored");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn test_dir(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("ubs_{label}_{}", Uuid::new_v4()))
    }

    #[test]
    fn backup_archive_round_trips_profile_files() {
        let source = test_dir("backup_source");
        let restore = test_dir("backup_restore");
        std::fs::create_dir_all(source.join("Bookmarks")).unwrap();
        std::fs::write(source.join("Bookmarks").join("data.json"), b"profile-data").unwrap();

        let archive = create_tar_archive(&source).unwrap();
        extract_tar_archive(&archive, &restore).unwrap();

        assert_eq!(
            std::fs::read(restore.join("Bookmarks").join("data.json")).unwrap(),
            b"profile-data"
        );
        let _ = std::fs::remove_dir_all(source);
        let _ = std::fs::remove_dir_all(restore);
    }

    #[test]
    fn restore_rejects_paths_outside_destination() {
        let restore = test_dir("backup_traversal");
        std::fs::create_dir_all(&restore).unwrap();
        let outside = restore.parent().unwrap().join("ubs_escape_test.txt");
        let _ = std::fs::remove_file(&outside);
        let mut builder = tar::Builder::new(Vec::new());
        let payload = b"must-not-escape";
        let mut header = tar::Header::new_gnu();
        header.set_size(payload.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(
                &mut header,
                "../ubs_escape_test.txt",
                Cursor::new(payload),
            )
            .unwrap();
        let archive = builder.into_inner().unwrap();

        assert!(extract_tar_archive(&archive, &restore).is_err());
        assert!(!outside.exists());
        let _ = std::fs::remove_dir_all(restore);
    }
}
