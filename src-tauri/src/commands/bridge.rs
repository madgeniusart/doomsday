//! Drains metadata-only events emitted by the companion native-messaging host.
use crate::AppState;
use serde::Serialize;
use std::{fs, io::{BufRead, BufReader}};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct ExtensionBridgeEvent {
    pub received_at: String,
    pub event: serde_json::Value,
}

#[tauri::command]
pub async fn drain_extension_events(state: State<'_, AppState>) -> Result<Vec<ExtensionBridgeEvent>, String> {
    let path = state.app_data_dir.join("extension-bridge-events.jsonl");
    if !path.exists() { return Ok(Vec::new()); }
    let processing = state.app_data_dir.join(format!("extension-bridge-events-{}.processing", uuid::Uuid::new_v4()));
    // Atomically claim the current queue. New host events append to a fresh file.
    if fs::rename(&path, &processing).is_err() { return Ok(Vec::new()); }
    let file = fs::File::open(&processing).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);
    let mut events = Vec::new();
    let mut line = String::new();
    while reader.read_line(&mut line).map_err(|e| e.to_string())? > 0 {
        if let Ok(event) = serde_json::from_str::<serde_json::Value>(&line) {
            events.push(ExtensionBridgeEvent {
                received_at: event.get("receivedAt").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                event,
            });
        }
        line.clear();
    }
    drop(reader);
    fs::remove_file(&processing).map_err(|e| e.to_string())?;
    Ok(events)
}
