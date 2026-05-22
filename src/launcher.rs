use std::path::{Path, PathBuf};

pub const ZELLIJ_SESSION_NAME: &str = "_rink_dash";
pub const TMUX_SESSION_NAME: &str = "_rink_default";

/// Directory for transient runtime files shared between zellij panes.
pub fn runtime_dir() -> PathBuf {
    std::env::var_os("RINK_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("rink")
}

/// Path to the file where the right pane's tty is stored.
pub fn client_tty_path() -> PathBuf {
    runtime_dir().join("client_tty")
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
    let rink_binary_arg = shell_quote(rink_binary);
    let terminal_env =
        "if [ -z \"${TERM:-}\" ] || [ \"$TERM\" = dumb ]; then export TERM=xterm-256color; fi";
    let left_shell_command = format!(
        "{terminal_env}; {rink_binary_arg} --inside; status=$?; printf '\\n'; printf 'rink --inside exited with status %s. Press Enter to close this pane.' \"$status\"; read _"
    );
    let right_shell_command = format!(
        "{terminal_env}; mkdir -p {} && tty > {tty_path_arg} && exec tmux new-session -A -s {}",
        shell_quote(&runtime_dir().to_string_lossy()),
        shell_quote(TMUX_SESSION_NAME)
    );
    format!(
        r#"layout {{
    tab {{
        pane split_direction="vertical" {{
            pane size="35%" name="sessions" {{
                command "sh"
                args "-lc" "{}"
            }}
            pane size="65%" {{
                command "sh"
                args "-lc" "{}"
            }}
        }}
    }}
}}

"#,
        kdl_string(&left_shell_command),
        kdl_string(&right_shell_command)
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

fn kill_session() {
    let _ = std::process::Command::new("zellij")
        .args(["delete-session", "--force", ZELLIJ_SESSION_NAME])
        .output();
}

/// Returns true when the current process is running inside any zellij session.
pub fn inside_zellij_environment() -> bool {
    std::env::var("ZELLIJ").is_ok()
}

/// Plain `rink` is the frame launcher. Do not silently switch it to dashboard-only
/// mode just because ZELLIJ is present: that happens inside the right tmux pane of
/// a rink frame and inside unrelated zellij sessions, and it looks exactly like
/// the left sidebar failed to appear.
pub fn reject_implicit_launch_inside_zellij() -> Result<(), String> {
    if inside_zellij_environment() {
        return Err(
            "rink is already running inside a zellij environment, so plain `rink` would not create the left/right frame.\n\
Run `rink --standalone` for dashboard-only mode in this pane, or run plain `rink` from outside zellij to create the left sidebar frame."
                .to_string(),
        );
    }
    Ok(())
}

/// Arguments used to start zellij with rink's layout.
pub fn zellij_launch_args(config: &Path, layout: &Path) -> Vec<String> {
    vec![
        "--session".to_string(),
        ZELLIJ_SESSION_NAME.to_string(),
        "--config".to_string(),
        config.to_string_lossy().to_string(),
        "--new-session-with-layout".to_string(),
        layout.to_string_lossy().to_string(),
    ]
}

/// Launch zellij with the rink layout.
pub fn launch_zellij() -> Result<(), String> {
    // The zellij session is only the outer frame; tmux keeps the actual work
    // session alive. Recreate the frame each time so stale/broken layouts from
    // older rink versions do not leave users in a blank zellij session.
    if session_is_alive() {
        kill_session();
    }

    let layout = write_layout()?;
    let config = write_zellij_config()?;

    let args = zellij_launch_args(&config, &layout);
    let status = std::process::Command::new("zellij")
        .args(&args)
        .status()
        .map_err(|e| format!("Failed to launch zellij: {}", e))?;

    if !status.success() {
        return Err("zellij exited with non-zero status".to_string());
    }

    Ok(())
}
