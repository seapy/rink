use std::collections::HashMap;
use std::path::PathBuf;

const STATUS_DIR: &str = "/tmp/rink";
const STATUS_EXPIRY_SECS: u64 = 600;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaudeStatus {
    Working,
    Waiting,
    Done,
    Unknown,
}

impl ClaudeStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            ClaudeStatus::Working => "*work",
            ClaudeStatus::Waiting => "?wait",
            ClaudeStatus::Done => "+done",
            ClaudeStatus::Unknown => "",
        }
    }

    pub fn parse_lossy(s: &str) -> Self {
        match s.trim() {
            "working" => ClaudeStatus::Working,
            "waiting" => ClaudeStatus::Waiting,
            "done" => ClaudeStatus::Done,
            _ => ClaudeStatus::Unknown,
        }
    }
}

/// Read all Claude status files from /tmp/rink/.
/// Returns a map of session_name -> ClaudeStatus
pub fn read_all_statuses() -> HashMap<String, ClaudeStatus> {
    let mut statuses = HashMap::new();
    let dir = PathBuf::from(STATUS_DIR);

    if !dir.exists() {
        return statuses;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    // Check file modification time for expiry
                    if let Ok(metadata) = path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let mod_secs = modified
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            if now - mod_secs > STATUS_EXPIRY_SECS {
                                continue;
                            }
                        }
                    }

                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let status = ClaudeStatus::parse_lossy(&content);
                        if status != ClaudeStatus::Unknown {
                            statuses.insert(filename.to_string(), status);
                        }
                    }
                }
            }
        }
    }

    statuses
}

/// Write a status file for a session
pub fn write_status(session_name: &str, status: &str) {
    let dir = PathBuf::from(STATUS_DIR);
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(session_name);
    let _ = std::fs::write(path, status);
}

/// Generate hook configuration JSON for Claude Code
pub fn hook_config() -> String {
    serde_json::json!({
        "hooks": {
            "UserPromptSubmit": [{
                "hooks": [{"type": "command", "command": "rink hook UserPromptSubmit"}]
            }],
            "PreToolUse": [{
                "hooks": [{"type": "command", "command": "rink hook PreToolUse"}]
            }],
            "PostToolUse": [{
                "hooks": [{"type": "command", "command": "rink hook PostToolUse"}]
            }],
            "Notification": [{
                "hooks": [{"type": "command", "command": "rink hook Notification"}]
            }],
            "Stop": [{
                "hooks": [{"type": "command", "command": "rink hook Stop"}]
            }]
        }
    })
    .to_string()
}

/// Merge rink hooks into ~/.claude/settings.json automatically
pub fn merge_hooks_into_settings() -> Result<String, String> {
    let rink_bin = std::env::current_exe()
        .map_err(|e| format!("Failed to get rink binary path: {}", e))?
        .to_string_lossy()
        .to_string();

    let settings_path = dirs::home_dir()
        .ok_or("Cannot find home directory")?
        .join(".claude")
        .join("settings.json");

    let mut settings: serde_json::Value = if settings_path.exists() {
        let content = std::fs::read_to_string(&settings_path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))?
    } else {
        serde_json::json!({})
    };

    // Ensure hooks object exists
    if settings.get("hooks").is_none() {
        settings["hooks"] = serde_json::json!({});
    }

    let rink_events = [
        "UserPromptSubmit",
        "PreToolUse",
        "PostToolUse",
        "Notification",
        "Stop",
    ];

    let hooks_obj = settings["hooks"]
        .as_object_mut()
        .ok_or("hooks is not an object")?;

    for event in &rink_events {
        let rink_command = format!("{} hook {}", rink_bin, event);
        let rink_entry = serde_json::json!({
            "hooks": [{"type": "command", "command": rink_command}]
        });

        // Remove any existing rink hook entries first
        if let Some(entries) = hooks_obj.get_mut(*event) {
            if let Some(arr) = entries.as_array_mut() {
                arr.retain(|entry| {
                    !entry["hooks"]
                        .as_array()
                        .map(|hooks| {
                            hooks.iter().any(|h| {
                                h["command"]
                                    .as_str()
                                    .map(|c| c.contains("rink hook") || c.contains("rink hook"))
                                    .unwrap_or(false)
                            })
                        })
                        .unwrap_or(false)
                });
                arr.push(rink_entry);
            }
        } else {
            hooks_obj.insert(event.to_string(), serde_json::json!([rink_entry]));
        }
    }

    // Create .claude dir if needed
    if let Some(parent) = settings_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create .claude dir: {}", e))?;
    }

    let formatted = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_path, &formatted)
        .map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(settings_path.to_string_lossy().to_string())
}
