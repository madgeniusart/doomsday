//! Minimal Chrome/Edge/Brave/Opera Native Messaging host for UBS.
//! It exchanges metadata-only health/profile-count messages and queues them
//! for the desktop app. It deliberately rejects cookie/password/session data.
use serde_json::{json, Value};
use std::{env, fs::{self, OpenOptions}, io::{self, Read, Write}, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

const MAX_MESSAGE: usize = 1024 * 1024;

fn app_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    { return env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(env::temp_dir).join("com.universal-browser-sync.app"); }
    #[cfg(target_os = "macos")]
    { return env::var_os("HOME").map(PathBuf::from).unwrap_or_else(env::temp_dir).join("Library/Application Support/com.universal-browser-sync.app"); }
    #[cfg(target_os = "linux")]
    { return env::var_os("XDG_DATA_HOME").map(PathBuf::from).or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share"))).unwrap_or_else(env::temp_dir).join("com.universal-browser-sync.app"); }
}

fn write_message(value: &Value) -> io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let len = u32::try_from(bytes.len()).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "message too large"))?;
    let mut out = io::stdout().lock();
    out.write_all(&len.to_ne_bytes())?;
    out.write_all(&bytes)?;
    out.flush()
}

fn queue_event(mut event: Value) -> io::Result<()> {
    let dir = app_data_dir();
    fs::create_dir_all(&dir)?;
    if let Some(obj) = event.as_object_mut() {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        obj.insert("receivedAt".into(), json!(now.to_string()));
    }
    let path = dir.join("extension-bridge-events.jsonl");
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, &event).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    file.write_all(b"\n")?;
    Ok(())
}

fn main() -> io::Result<()> {
    let mut input = io::stdin().lock();
    loop {
        let mut header = [0u8; 4];
        match input.read_exact(&mut header) {
            Ok(()) => {},
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(()),
            Err(e) => return Err(e),
        }
        let length = u32::from_ne_bytes(header) as usize;
        if length == 0 || length > MAX_MESSAGE { return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid native message length")); }
        let mut payload = vec![0u8; length];
        input.read_exact(&mut payload)?;
        let request: Value = serde_json::from_slice(&payload)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid JSON message"))?;
        let action = request.get("action").and_then(Value::as_str).unwrap_or("");
        let response = match action {
            "PING" => {
                let event = json!({"type":"bridge_connected", "protocol":1});
                match queue_event(event) {
                    Ok(()) => json!({"ok":true,"host":"com.universal-browser-sync.bridge","protocol":1,"mode":"metadata_only"}),
                    Err(_) => json!({"ok":false,"error":"could not queue bridge event for desktop app"}),
                }
            }
            "PROFILE_SUMMARY" => {
                let browser = request.get("browser").and_then(Value::as_str).unwrap_or("unknown");
                let allowed = ["chrome", "edge", "brave", "opera", "firefox"];
                if !allowed.contains(&browser) {
                    json!({"ok":false,"error":"unsupported browser identifier"})
                } else {
                    let counts = request.get("counts").cloned().unwrap_or_else(|| json!({}));
                    let safe_count = |key: &str| counts.get(key).and_then(Value::as_u64).unwrap_or(0).min(1_000_000);
                    let event = json!({"type":"profile_summary","browser":browser,"profileName":request.get("profileName").and_then(Value::as_str).unwrap_or("Default").chars().take(80).collect::<String>(),"counts":{"bookmarks":safe_count("bookmarks"),"history":safe_count("history"),"extensions":safe_count("extensions")}});
                    match queue_event(event) {
                        Ok(()) => json!({"ok":true,"accepted":true}),
                        Err(_) => json!({"ok":false,"error":"could not queue event for desktop app"}),
                    }
                }
            }
            _ => json!({"ok":false,"error":"unknown action"}),
        };
        write_message(&response)?;
    }
}
