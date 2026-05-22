use std::process::Command;
use tempfile::TempDir;

fn rink_bin() -> &'static str {
    env!("CARGO_BIN_EXE_rink")
}

#[cfg(unix)]
fn write_fake_executable(dir: &std::path::Path, name: &str) {
    use std::os::unix::fs::PermissionsExt;

    let path = dir.join(name);
    std::fs::write(
        &path,
        "#!/bin/sh\necho unexpected $0 $@ >> \"$FAKE_COMMAND_LOG\"\nexit 0\n",
    )
    .expect("write fake executable");
    let mut perms = std::fs::metadata(&path)
        .expect("fake executable metadata")
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod fake executable");
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
fn doctor_reset_ignores_xdg_runtime_dir_when_no_runtime_override_is_set() {
    let home = TempDir::new().expect("temp home");
    let xdg_data = home.path().join("data");
    let xdg_runtime = home.path().join("runtime");
    std::fs::create_dir_all(xdg_runtime.join("rink")).expect("runtime dir");
    std::fs::write(xdg_runtime.join("rink/client_tty"), "/dev/pts/77").expect("tty");

    let output = Command::new(rink_bin())
        .args(["doctor", "reset", "--dry-run"])
        .env("HOME", home.path())
        .env("XDG_DATA_HOME", &xdg_data)
        .env("XDG_RUNTIME_DIR", &xdg_runtime)
        .env_remove("RINK_RUNTIME_DIR")
        .env("PATH", "")
        .output()
        .expect("run rink doctor reset --dry-run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("/tmp/rink/client_tty"),
        "stdout was: {stdout}"
    );
    assert!(
        !stdout.contains(&xdg_runtime.join("rink/client_tty").display().to_string()),
        "stdout should not use XDG_RUNTIME_DIR: {stdout}"
    );
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

#[test]
#[cfg(unix)]
fn plain_rink_inside_zellij_reports_explicit_mode_instead_of_hiding_frame() {
    let temp = TempDir::new().expect("temp dir");
    let bin = temp.path().join("bin");
    std::fs::create_dir_all(&bin).expect("fake bin dir");
    write_fake_executable(&bin, "tmux");
    write_fake_executable(&bin, "zellij");
    let command_log = temp.path().join("commands.log");

    let output = Command::new(rink_bin())
        .env("PATH", &bin)
        .env("ZELLIJ", "0")
        .env("FAKE_COMMAND_LOG", &command_log)
        .output()
        .expect("run rink inside zellij env");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already running inside a zellij environment"),
        "stderr was: {stderr}"
    );
    assert!(
        stderr.contains("rink --standalone"),
        "stderr should point to explicit dashboard-only mode: {stderr}"
    );
    assert!(
        !command_log.exists(),
        "plain rink inside zellij should not try to launch or mutate zellij/tmux"
    );
}

#[test]
fn doctor_inspect_prints_runtime_and_live_state_sections() {
    let home = TempDir::new().expect("temp home");
    let runtime = home.path().join("runtime");
    std::fs::create_dir_all(runtime.join("rink")).expect("runtime dir");
    std::fs::write(runtime.join("rink/client_tty"), "/dev/pts/123").expect("tty");

    let output = Command::new(rink_bin())
        .args(["doctor", "inspect"])
        .env("RINK_RUNTIME_DIR", &runtime)
        .env("PATH", "")
        .output()
        .expect("run rink doctor inspect");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("rink doctor inspect"),
        "stdout was: {stdout}"
    );
    assert!(stdout.contains("runtime dir:"), "stdout was: {stdout}");
    assert!(
        stdout.contains("client tty value: /dev/pts/123"),
        "stdout was: {stdout}"
    );
    assert!(
        stdout.contains("== zellij sessions =="),
        "stdout was: {stdout}"
    );
    assert!(
        stdout.contains("skip: zellij not found"),
        "stdout was: {stdout}"
    );
}
