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
use std::path::PathBuf;
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

    /// Config file path
    #[arg(long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize config files
    Init,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    check_platform()?;

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            init_config()?;
            return Ok(());
        }
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

fn check_platform() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(not(target_os = "macos")) {
        eprintln!("Error: rink only supports macOS.");
        std::process::exit(1);
    }
    Ok(())
}

fn has_command(name: &str) -> bool {
    ProcessCommand::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn brew_install(package: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !has_command("brew") {
        return Err(format!(
            "Homebrew is not installed. Install it first: https://brew.sh\nThen run: brew install {}",
            package
        ).into());
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

fn ensure_dependencies(standalone: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !has_command("tmux") {
        brew_install("tmux")?;
    }

    if !standalone && !has_command("zellij") {
        brew_install("zellij")?;
    }

    Ok(())
}

fn run_tui(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load(cli.config.as_ref());
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

fn init_config() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rink");
    std::fs::create_dir_all(&config_dir)?;

    let config_path = config_dir.join("config.toml");
    if !config_path.exists() {
        let default_config = r#"# Rink configuration
separator = "-"
refresh_interval_ms = 2000
default_sort = "name"

# [categories.work]
# name = "Work"
# color = "blue"
"#;
        std::fs::write(&config_path, default_config)?;
        println!("Created config: {}", config_path.display());
    } else {
        println!("Config already exists: {}", config_path.display());
    }

    Ok(())
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
