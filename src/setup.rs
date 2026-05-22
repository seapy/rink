use crate::launcher;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct DependencyStatus {
    pub name: &'static str,
    pub present: bool,
    pub required_for_default_ui: bool,
}

#[derive(Debug, Clone)]
pub struct SetupStep {
    pub title: String,
    pub command: String,
}

pub fn has_command(name: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&paths).any(|dir| {
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

pub fn dependency_statuses() -> Vec<DependencyStatus> {
    vec![
        DependencyStatus {
            name: "tmux",
            present: has_command("tmux"),
            required_for_default_ui: true,
        },
        DependencyStatus {
            name: "zellij",
            present: has_command("zellij"),
            required_for_default_ui: true,
        },
    ]
}

pub fn print_doctor() -> bool {
    println!("rink doctor");
    println!("-----------");

    let statuses = dependency_statuses();
    let mut ok = true;

    for status in &statuses {
        if status.present {
            println!("ok: {}", status.name);
        } else {
            ok = false;
            println!("missing: {}", status.name);
        }
    }

    if ok {
        println!("\nAll required commands are available.");
    } else {
        println!("\nRun: rink setup");
        println!("Or use dashboard-only mode without zellij: rink --standalone");
    }

    ok
}

pub fn run_reset(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("rink doctor reset");
    println!("-----------------");
    if dry_run {
        println!("dry run: state that would be reset\n");
    }

    reset_zellij_session(dry_run)?;
    reset_tmux_session(dry_run)?;
    reset_generated_files(dry_run)?;

    if dry_run {
        println!("\nDry run complete. Run without --dry-run to reset rink state.");
    } else {
        println!("\nReset complete. Run: rink");
    }

    Ok(())
}

fn reset_zellij_session(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let session = launcher::ZELLIJ_SESSION_NAME;
    if !has_command("zellij") {
        println!("skip: zellij session {session} (zellij not found)");
        return Ok(());
    }

    if dry_run {
        println!("would delete: zellij session {session}");
        return Ok(());
    }

    let output = Command::new("zellij")
        .args(["delete-session", "--force", session])
        .output()?;
    if output.status.success() {
        println!("deleted: zellij session {session}");
    } else {
        println!("ok: zellij session {session} was not active");
    }
    Ok(())
}

fn reset_tmux_session(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let session = launcher::TMUX_SESSION_NAME;
    if !has_command("tmux") {
        println!("skip: tmux session {session} (tmux not found)");
        return Ok(());
    }

    if dry_run {
        println!("would kill: tmux session {session}");
        return Ok(());
    }

    let output = Command::new("tmux")
        .args(["kill-session", "-t", session])
        .output()?;
    if output.status.success() {
        println!("killed: tmux session {session}");
    } else {
        println!("ok: tmux session {session} was not active");
    }
    Ok(())
}

fn reset_generated_files(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let targets = reset_file_targets();
    for target in targets {
        if dry_run {
            println!("would remove: {}", target.display());
            continue;
        }

        if target.is_dir() {
            std::fs::remove_dir_all(&target)?;
            println!("removed: {}", target.display());
        } else if target.exists() {
            std::fs::remove_file(&target)?;
            println!("removed: {}", target.display());
        } else {
            println!("ok: {} already absent", target.display());
        }
    }
    Ok(())
}

fn reset_file_targets() -> Vec<PathBuf> {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("rink");

    vec![
        data_dir.join("layout.kdl"),
        data_dir.join("zellij.kdl"),
        launcher::client_tty_path(),
        launcher::runtime_dir(),
    ]
}

pub fn setup_steps() -> Vec<SetupStep> {
    let mut steps = Vec::new();

    if !has_command("tmux") {
        steps.push(SetupStep {
            title: "Install tmux".to_string(),
            command: tmux_install_command(),
        });
    }

    if !has_command("zellij") {
        steps.push(SetupStep {
            title: "Install zellij".to_string(),
            command: zellij_install_command(),
        });
    }

    steps
}

pub fn run_setup(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("rink setup");
    println!("----------");

    let steps = setup_steps();
    if steps.is_empty() {
        println!("tmux and zellij are already installed.");
        return Ok(());
    }

    if dry_run {
        println!("dry run: commands that would be executed\n");
    }

    for step in steps {
        println!("==> {}", step.title);
        println!("{}", step.command);

        if dry_run {
            println!();
            continue;
        }

        let status = Command::new("/bin/sh")
            .arg("-c")
            .arg(&step.command)
            .status()?;
        if !status.success() {
            return Err(format!("setup step failed: {}", step.title).into());
        }
        println!();
    }

    if dry_run {
        println!("Dry run complete. Run without --dry-run to install missing dependencies.");
    } else {
        ensure_local_bin_hint();
        println!("Done. Run: rink doctor");
    }
    Ok(())
}

fn tmux_install_command() -> String {
    if cfg!(target_os = "macos") {
        "brew install tmux".to_string()
    } else if cfg!(target_os = "linux") {
        if has_command("apt-get") {
            "if [ \"$(id -u)\" = 0 ]; then apt-get update && apt-get install -y tmux; else sudo apt-get update && sudo apt-get install -y tmux; fi".to_string()
        } else if has_command("dnf") {
            "if [ \"$(id -u)\" = 0 ]; then dnf install -y tmux; else sudo dnf install -y tmux; fi"
                .to_string()
        } else if has_command("pacman") {
            "if [ \"$(id -u)\" = 0 ]; then pacman -S --needed tmux; else sudo pacman -S --needed tmux; fi".to_string()
        } else {
            "echo 'No supported package manager found. Install tmux with your distro package manager.' >&2; exit 1".to_string()
        }
    } else {
        "echo 'Install tmux with your OS package manager.' >&2; exit 1".to_string()
    }
}

fn zellij_install_command() -> String {
    if cfg!(target_os = "macos") {
        "brew install zellij".to_string()
    } else if cfg!(target_os = "linux") {
        let install_dir = install_dir();
        format!(
            r#"mkdir -p {install_dir}
tmp=$(mktemp -d)
arch=$(uname -m)
case "$arch" in
  x86_64) zellij_target="x86_64-unknown-linux-musl" ;;
  aarch64|arm64) zellij_target="aarch64-unknown-linux-musl" ;;
  *) echo "Unsupported zellij arch: $arch" >&2; exit 1 ;;
esac
zellij_tag=$(curl -fsSL https://api.github.com/repos/zellij-org/zellij/releases/latest | grep '"tag_name"' | sed 's/.*: "//;s/".*//')
curl -fsSL "https://github.com/zellij-org/zellij/releases/download/${{zellij_tag}}/zellij-${{zellij_target}}.tar.gz" -o "$tmp/zellij.tar.gz"
tar -xzf "$tmp/zellij.tar.gz" -C "$tmp"
install -m 0755 "$tmp/zellij" {install_dir}/zellij
rm -rf "$tmp""#,
            install_dir = shell_quote(&install_dir.to_string_lossy())
        )
    } else {
        "echo 'Install zellij from https://zellij.dev/documentation/installation' >&2; exit 1"
            .to_string()
    }
}

fn install_dir() -> PathBuf {
    if let Some(dir) = env::var_os("INSTALL_DIR") {
        return PathBuf::from(dir);
    }

    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home).join(".local/bin");
    }

    PathBuf::from("/usr/local/bin")
}

fn ensure_local_bin_hint() {
    let dir = install_dir();
    let Some(paths) = env::var_os("PATH") else {
        println!(
            "Add to your shell profile: export PATH=\"{}:$PATH\"",
            dir.display()
        );
        return;
    };

    if !env::split_paths(&paths).any(|path| path == dir) {
        println!(
            "Add to your shell profile: export PATH=\"{}:$PATH\"",
            dir.display()
        );
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
