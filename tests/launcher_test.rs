use rink::launcher::{generate_kdl_layout, zellij_launch_args};
use std::path::Path;

#[test]
fn left_pane_runs_rink_inside_through_shell_and_keeps_errors_visible() {
    let layout = generate_kdl_layout("/tmp/rink binary/rink");

    assert!(layout.contains("pane size=\"35%\" name=\"sessions\""));
    assert!(layout.contains("command \"sh\""), "layout was: {layout}");
    assert!(layout.contains("args \"-lc\""), "layout was: {layout}");
    assert!(layout.contains("--inside"), "layout was: {layout}");
    assert!(
        layout.contains("rink --inside exited"),
        "layout was: {layout}"
    );
    assert!(
        layout.contains("/tmp/rink binary/rink"),
        "layout was: {layout}"
    );
    assert!(
        layout.contains("export TERM=xterm-256color"),
        "layout was: {layout}"
    );
}

#[test]
fn right_pane_sets_safe_term_before_running_tmux() {
    let layout = generate_kdl_layout("/tmp/rink");

    let term_fix = "if [ -z \\\"${TERM:-}\\\" ] || [ \\\"$TERM\\\" = dumb ]; then export TERM=xterm-256color; fi";
    assert!(layout.contains(term_fix), "layout was: {layout}");
    assert!(
        layout.contains("exec tmux new-session -A -s _rink_default"),
        "layout was: {layout}"
    );
    assert!(
        layout.find(term_fix).unwrap() < layout.find("exec tmux new-session").unwrap(),
        "TERM fallback must run before tmux: {layout}"
    );
}

#[test]
fn launcher_uses_long_new_session_with_layout_flag_instead_of_short_flag() {
    let args = zellij_launch_args(
        Path::new("/tmp/rink/zellij.kdl"),
        Path::new("/tmp/rink/layout.kdl"),
    );

    assert_eq!(args[0], "--session");
    assert!(
        args.contains(&"--new-session-with-layout".to_string()),
        "args were: {args:?}"
    );
    assert!(
        args.contains(&"--config".to_string()),
        "args were: {args:?}"
    );
    assert!(!args.contains(&"-n".to_string()), "args were: {args:?}");
}
