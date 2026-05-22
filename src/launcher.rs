use std::path::PathBuf;

const ZELLIJ_SESSION_NAME: &str = "_rink_dash";

/// Path to the file where the right pane's tty is stored.
pub fn client_tty_path() -> PathBuf {
    PathBuf::from("/tmp/rink/client_tty")
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn kdl_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Generate a KDL layout for zellij with rink TUI on the left and tmux on the right.
pub fn generate_kdl_layout(rink_binary: &str) -> String {
    let tty_path = client_tty_path();
    let tty_path_arg = shell_quote(&tty_path.to_string_lossy());
    let shell_command = format!(
        "mkdir -p /tmp/rink && tty > {tty_path_arg} && exec tmux new-session -A -s _rink_default"
    );
    format!(
        r#"layout {{
    tab {{
        pane split_direction="vertical" {{
            pane size="35%" name="sessions" {{
                command "{}"
                args "--inside"
            }}
            pane size="65%" {{
                command "sh"
                args "-c" "{}"
            }}
        }}
    }}
}}

"#,
        kdl_string(rink_binary),
        kdl_string(&shell_command)
    )
}

fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rink")
}

fn write_layout() -> Result<PathBuf, String> {
    let rink_binary = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?
        .to_string_lossy()
        .to_string();

    let dir = data_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create data dir: {}", e))?;

    let layout_path = dir.join("layout.kdl");
    std::fs::write(&layout_path, generate_kdl_layout(&rink_binary))
        .map_err(|e| format!("Failed to write layout file: {}", e))?;

    Ok(layout_path)
}

fn write_zellij_config() -> Result<PathBuf, String> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create data dir: {}", e))?;

    let config_path = dir.join("zellij.kdl");
    let content = r#"// rink zellij config
ui {
    pane_frames {
        rounded_corners true
        hide_session_name true
    }
}
"#;
    std::fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write zellij config: {}", e))?;

    Ok(config_path)
}

/// Check if our zellij session exists and is alive (not EXITED)
fn session_is_alive() -> bool {
    let output = std::process::Command::new("zellij")
        .args(["list-sessions", "--no-formatting"])
        .output();

    match output {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout);
            text.lines()
                .any(|line| line.starts_with(ZELLIJ_SESSION_NAME) && !line.contains("EXITED"))
        }
        Err(_) => false,
    }
}

/// Kill any dead/exited zellij session with our name
fn kill_dead_session() {
    let _ = std::process::Command::new("zellij")
        .args(["delete-session", ZELLIJ_SESSION_NAME])
        .output();
}

/// Launch zellij with the rink layout, or attach to existing session
pub fn launch_zellij() -> Result<(), String> {
    if session_is_alive() {
        let status = std::process::Command::new("zellij")
            .args(["attach", ZELLIJ_SESSION_NAME])
            .status()
            .map_err(|e| format!("Failed to attach to zellij session: {}", e))?;

        if status.success() {
            return Ok(());
        }
    }

    // Clean up dead session if any, then create new
    kill_dead_session();

    let layout = write_layout()?;
    let config = write_zellij_config()?;

    let status = std::process::Command::new("zellij")
        .args([
            "-s",
            ZELLIJ_SESSION_NAME,
            "-c",
            &config.to_string_lossy(),
            "-n",
            &layout.to_string_lossy(),
        ])
        .status()
        .map_err(|e| format!("Failed to launch zellij: {}", e))?;

    if !status.success() {
        return Err("zellij exited with non-zero status".to_string());
    }

    Ok(())
}
