use clap::{Parser, Subcommand};
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use rink::app::{Action, App};
use rink::config::Config;
use rink::keys;
use rink::launcher;
use rink::status;
use rink::tmux::{RealTmuxClient, TmuxClient};
use rink::ui;
use std::io;
use std::path::Path;
use std::process::Command as ProcessCommand;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "rink", about = "tmux session dashboard", version)]
struct Cli {
    /// Run inside zellij (TUI mode)
    #[arg(long)]
    inside: bool,

    /// Run standalone without zellij frame
    #[arg(long)]
    standalone: bool,

    /// Filter by category
    #[arg(long)]
    category: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Handle Claude Code hooks
    Hook {
        /// Hook event type
        event: String,
    },
    /// Install Claude Code hooks
    HookInstall,
    /// Print hook configuration JSON
    HookConfig,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Hook { event }) => {
            handle_hook(&event)?;
            return Ok(());
        }
        Some(Commands::HookInstall) => {
            install_hooks()?;
            return Ok(());
        }
        Some(Commands::HookConfig) => {
            println!("{}", status::hook_config());
            return Ok(());
        }
        None => {}
    }

    let inside_zellij = std::env::var("ZELLIJ").is_ok();

    ensure_dependencies(cli.standalone || cli.inside || inside_zellij)?;

    if cli.inside || cli.standalone || inside_zellij {
        // Run TUI mode
        run_tui(cli)?;
    } else {
        // Launch zellij with rink layout
        if let Err(e) = launcher::launch_zellij() {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn has_command(name: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };

    std::env::split_paths(&paths).any(|dir| {
        let candidate = dir.join(name);
        candidate.is_file() && is_executable(&candidate)
    })
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.metadata()
        .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn install_with_homebrew(package: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !has_command("brew") {
        return Err(format!(
            "Homebrew is not installed. Install it first: https://brew.sh\nThen run: brew install {}",
            package
        )
        .into());
    }

    eprintln!("Installing {} via Homebrew...", package);
    let status = ProcessCommand::new("brew")
        .args(["install", package])
        .status()?;

    if !status.success() {
        return Err(format!("Failed to install {}", package).into());
    }

    eprintln!("{} installed successfully.", package);
    Ok(())
}

fn dependency_install_hint(package: &str) -> String {
    if cfg!(target_os = "macos") {
        format!("Run: brew install {package}")
    } else if cfg!(target_os = "linux") {
        match package {
            "tmux" => linux_tmux_install_hint(),
            "zellij" => linux_zellij_install_hint(),
            _ => format!(
                "Install {package} with your distro package manager and make sure it is on PATH."
            ),
        }
    } else {
        format!("Install {package} and make sure it is on PATH.")
    }
}

fn linux_tmux_install_hint() -> String {
    "Install tmux, then rerun rink. Examples:\n\n  Ubuntu/Debian:\n    sudo apt update\n    sudo apt install -y tmux\n\n  Fedora:\n    sudo dnf install tmux\n\n  Arch:\n    sudo pacman -S tmux"
        .to_string()
}

fn linux_zellij_install_hint() -> String {
    "Install zellij, then rerun rink. On Ubuntu/Debian, use the upstream prebuilt binary:\n\n  mkdir -p \"$HOME/.local/bin\"\n  tmp=$(mktemp -d)\n  arch=$(uname -m)\n  case \"$arch\" in\n    x86_64) zellij_target=\"x86_64-unknown-linux-musl\" ;;\n    aarch64|arm64) zellij_target=\"aarch64-unknown-linux-musl\" ;;\n    *) echo \"Unsupported zellij arch: $arch\" >&2; exit 1 ;;\n  esac\n  zellij_tag=$(curl -fsSL https://api.github.com/repos/zellij-org/zellij/releases/latest | grep '\"tag_name\"' | sed 's/.*: \"//;s/\".*//')\n  curl -fsSL \"https://github.com/zellij-org/zellij/releases/download/${zellij_tag}/zellij-${zellij_target}.tar.gz\" -o \"$tmp/zellij.tar.gz\"\n  tar -xzf \"$tmp/zellij.tar.gz\" -C \"$tmp\"\n  install -m 0755 \"$tmp/zellij\" \"$HOME/.local/bin/zellij\"\n  rm -rf \"$tmp\"\n\n  # If needed, add this to your shell profile:\n  export PATH=\"$HOME/.local/bin:$PATH\"\n\nOther options:\n  cargo install --locked zellij\n  See: https://zellij.dev/documentation/installation"
        .to_string()
}

fn ensure_dependency(
    package: &str,
    allow_auto_install: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if has_command(package) {
        return Ok(());
    }

    if allow_auto_install && cfg!(target_os = "macos") {
        install_with_homebrew(package)?;
        return Ok(());
    }

    Err(format!(
        "Required command '{}' was not found on PATH.\n{}",
        package,
        dependency_install_hint(package)
    )
    .into())
}

fn ensure_dependencies(standalone: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Keep the old macOS convenience behavior, but avoid running sudo-capable
    // package managers implicitly on Linux servers.
    let auto_install = cfg!(target_os = "macos");

    ensure_dependency("tmux", auto_install)?;

    if !standalone {
        ensure_dependency("zellij", auto_install)?;
    }

    Ok(())
}

fn run_tui(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();
    let refresh_interval = Duration::from_millis(config.refresh_interval_ms);

    let mut app = App::new(config, cli.standalone);
    let client = RealTmuxClient::new();

    // Initial session load
    app.refresh_sessions(&client);
    app.update_preview(&client);

    // Set initial tab name based on current tmux session
    if let Some(current) = client.current_session() {
        update_zellij_pane_name(&current);
    } else {
        update_zellij_pane_name("tmux dashboard");
    }

    // Apply category filter if specified
    if let Some(ref cat) = cli.category {
        app.category_focus = Some(cat.clone());
        app.rebuild_visible_items();
    }

    // Setup terminal
    terminal::enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal =
        ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(io::stdout()))?;

    let mut last_refresh = Instant::now();

    loop {
        // Render
        terminal.draw(|frame| {
            ui::render(frame, &app);
        })?;

        // Handle events with timeout for periodic refresh
        let timeout = refresh_interval
            .checked_sub(last_refresh.elapsed())
            .unwrap_or(Duration::from_millis(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events (not release/repeat)
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                let cmd = keys::translate(key, &app.mode);
                let action = app.execute(cmd, &client);

                match action {
                    Action::Quit => break,
                    Action::SwitchTmuxSession(ref name) => {
                        update_zellij_pane_name(name);
                        app.refresh_sessions(&client);
                    }
                    _ => {}
                }
                app.update_preview(&client);
            }
        }

        // Periodic refresh
        if last_refresh.elapsed() >= refresh_interval {
            app.refresh_sessions(&client);
            app.update_preview(&client);
            last_refresh = Instant::now();
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}

fn update_zellij_pane_name(session_name: &str) {
    if std::env::var("ZELLIJ").is_err() {
        return;
    }
    // Focus right pane → rename → focus back to left pane
    let _ = ProcessCommand::new("zellij")
        .args(["action", "focus-next-pane"])
        .output();
    let _ = ProcessCommand::new("zellij")
        .args(["action", "rename-pane", session_name])
        .output();
    let _ = ProcessCommand::new("zellij")
        .args(["action", "focus-previous-pane"])
        .output();
}

fn handle_hook(event: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Get the current tmux session name
    let session_name = std::process::Command::new("tmux")
        .args(["display-message", "-p", "#{session_name}"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_default();

    if session_name.is_empty() {
        return Ok(());
    }

    let status_str = match event {
        "UserPromptSubmit" | "PreToolUse" | "PostToolUse" => "working",
        "Notification" => "waiting",
        "Stop" => "done",
        // Legacy lowercase names
        "pre-tool-use" | "post-tool-use" => "working",
        "notification" => "waiting",
        "stop" => "done",
        _ => return Ok(()),
    };

    status::write_status(&session_name, status_str);
    Ok(())
}

fn install_hooks() -> Result<(), Box<dyn std::error::Error>> {
    match status::merge_hooks_into_settings() {
        Ok(path) => {
            println!("Hooks installed successfully to {}", path);
            println!("Restart Claude Code to activate.");
        }
        Err(e) => {
            eprintln!("Auto-install failed: {}", e);
            eprintln!();
            eprintln!("Add manually to ~/.claude/settings.json:");
            eprintln!("{}", status::hook_config());
        }
    }
    Ok(())
}
