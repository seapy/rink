use std::process::Command;
use tempfile::TempDir;

fn rink_bin() -> &'static str {
    env!("CARGO_BIN_EXE_rink")
}

#[test]
fn doctor_reports_missing_dependencies() {
    let output = Command::new(rink_bin())
        .arg("doctor")
        .env("PATH", "")
        .output()
        .expect("run rink doctor");

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rink doctor"), "stdout was: {stdout}");
    assert!(stdout.contains("missing: tmux"), "stdout was: {stdout}");
    assert!(stdout.contains("missing: zellij"), "stdout was: {stdout}");
    assert!(stdout.contains("Run: rink setup"), "stdout was: {stdout}");
}

#[test]
fn setup_dry_run_prints_install_plan_without_running_it() {
    let output = Command::new(rink_bin())
        .args(["setup", "--dry-run"])
        .env("PATH", "")
        .output()
        .expect("run rink setup --dry-run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rink setup"), "stdout was: {stdout}");
    assert!(stdout.contains("dry run"), "stdout was: {stdout}");
    assert!(stdout.contains("Install tmux"), "stdout was: {stdout}");
    assert!(stdout.contains("Install zellij"), "stdout was: {stdout}");
    assert!(
        stdout.contains("zellij-org/zellij/releases"),
        "stdout was: {stdout}"
    );
}

#[test]
fn doctor_reset_dry_run_lists_runtime_state_without_deleting_it() {
    let home = TempDir::new().expect("temp home");
    let xdg_data = home.path().join("data");
    let tmp = home.path().join("tmp");
    std::fs::create_dir_all(xdg_data.join("rink")).expect("create data dir");
    std::fs::create_dir_all(tmp.join("rink")).expect("create runtime dir");
    std::fs::write(xdg_data.join("rink/layout.kdl"), "bad layout").expect("layout");
    std::fs::write(xdg_data.join("rink/zellij.kdl"), "bad config").expect("config");
    std::fs::write(tmp.join("rink/client_tty"), "/dev/pts/99").expect("tty");

    let output = Command::new(rink_bin())
        .args(["doctor", "reset", "--dry-run"])
        .env("HOME", home.path())
        .env("XDG_DATA_HOME", &xdg_data)
        .env("RINK_RUNTIME_DIR", &tmp)
        .env("PATH", "")
        .output()
        .expect("run rink doctor reset --dry-run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rink doctor reset"), "stdout was: {stdout}");
    assert!(stdout.contains("dry run"), "stdout was: {stdout}");
    assert!(stdout.contains("_rink_dash"), "stdout was: {stdout}");
    assert!(stdout.contains("_rink_default"), "stdout was: {stdout}");
    assert!(stdout.contains("layout.kdl"), "stdout was: {stdout}");
    assert!(stdout.contains("client_tty"), "stdout was: {stdout}");
    assert!(xdg_data.join("rink/layout.kdl").exists());
    assert!(tmp.join("rink/client_tty").exists());
}

#[test]
fn doctor_reset_removes_generated_rink_files_even_without_tmux_or_zellij() {
    let home = TempDir::new().expect("temp home");
    let xdg_data = home.path().join("data");
    let tmp = home.path().join("tmp");
    std::fs::create_dir_all(xdg_data.join("rink")).expect("create data dir");
    std::fs::create_dir_all(tmp.join("rink")).expect("create runtime dir");
    std::fs::write(xdg_data.join("rink/layout.kdl"), "bad layout").expect("layout");
    std::fs::write(xdg_data.join("rink/zellij.kdl"), "bad config").expect("config");
    std::fs::write(tmp.join("rink/client_tty"), "/dev/pts/99").expect("tty");

    let output = Command::new(rink_bin())
        .args(["doctor", "reset"])
        .env("HOME", home.path())
        .env("XDG_DATA_HOME", &xdg_data)
        .env("RINK_RUNTIME_DIR", &tmp)
        .env("PATH", "")
        .output()
        .expect("run rink doctor reset");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("removed:"), "stdout was: {stdout}");
    assert!(!xdg_data.join("rink/layout.kdl").exists());
    assert!(!xdg_data.join("rink/zellij.kdl").exists());
    assert!(!tmp.join("rink/client_tty").exists());
}
