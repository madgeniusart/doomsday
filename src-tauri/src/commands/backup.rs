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
    let profile_path = PathBuf::from(&request.profile_path);
    let output_dir = PathBuf::from(&request.output_dir);

    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let filename = format!(
        "ubs_backup_{}_{}.tar{}",
        request.browser,
        timestamp,
        if request.encrypt { ".enc" } else { ".gz" }
    );
    let output_path = output_dir.join(&filename);

    // Create tar archive of profile
    let tar_data = create_tar_archive(&profile_path).map_err(|e| e.to_string())?;

    if request.encrypt {
        let enc = state.encryption.lock().unwrap();
        let encrypted = enc.encrypt(&tar_data).map_err(|e| e.to_string())?;
        std::fs::write(&output_path, encrypted).map_err(|e| e.to_string())?;
    } else {
        // Compress with gzip
        use flate2::{write::GzEncoder, Compression};
        use std::io::Write;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&tar_data).map_err(|e| e.to_string())?;
        let compressed = encoder.finish().map_err(|e| e.to_string())?;
        std::fs::write(&output_path, compressed).map_err(|e| e.to_string())?;
    }

    let size_bytes = std::fs::metadata(&output_path)
        .map(|m| m.len() as i64)
        .unwrap_or(0);

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

    let db = state.db.lock().unwrap();
    db.record_backup(&record).map_err(|e| e.to_string())?;

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
        let enc = state.encryption.lock().unwrap();
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

    for entry in walkdir::WalkDir::new(dir)
        .follow_links(false)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let filename = path.file_name().unwrap_or_default().to_string_lossy();

        if skip_patterns.iter().any(|p| filename.contains(p)) {
            continue;
        }

        if path.is_file() {
            let relative = path.strip_prefix(dir)?;
            // Skip files larger than 100MB (e.g., large history DBs)
            if let Ok(meta) = std::fs::metadata(path) {
                if meta.len() > 100 * 1024 * 1024 {
                    log::warn!("Skipping large file in backup: {}", path.display());
                    continue;
                }
            }
            if let Ok(mut f) = std::fs::File::open(path) {
                let _ = archive.append_file(relative, &mut f);
            }
        }
    }

    Ok(archive.into_inner()?)
}

fn extract_tar_archive(data: &[u8], dest: &PathBuf) -> anyhow::Result<()> {
    use tar::Archive;
    let mut archive = Archive::new(data);
    archive.unpack(dest)?;
    Ok(())
}
