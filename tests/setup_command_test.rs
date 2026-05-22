use std::process::Command;

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
