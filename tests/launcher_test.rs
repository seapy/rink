use rink::launcher::{generate_kdl_layout, list_sessions_contains, zellij_launch_args};
use std::path::Path;

#[test]
fn session_exists_detects_alive_detached_and_exited_states() {
    // Each entry below is sampled from `zellij list-sessions --no-formatting`.
    // The launcher must recognize the rink session in every state, because
    // zellij refuses `--new-session-with-layout` when an EXITED session with
    // the same name still exists. Failing to detect EXITED here is what made
    // the left sidebar appear "missing" — zellij would not start at all.
    let alive = "dev [Created 4m 52s ago] (current)\n_rink_dash [Created 3s ago]";
    let detached = "_rink_dash [Created 3s ago]";
    let exited = "_rink_dash [Created 27s ago] (EXITED - attach to resurrect)";

    assert!(list_sessions_contains(alive, "_rink_dash"));
    assert!(list_sessions_contains(detached, "_rink_dash"));
    assert!(list_sessions_contains(exited, "_rink_dash"));

    // Must not false-match a session whose name merely starts with ours.
    let look_alike = "_rink_dash_extra [Created 5s ago]";
    assert!(!list_sessions_contains(look_alike, "_rink_dash"));

    // Must not match when the session is absent.
    let none = "dev [Created 1s ago] (current)";
    assert!(!list_sessions_contains(none, "_rink_dash"));
}

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
        layout.contains("exec tmux new-session -A -s '_rink_default'"),
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
