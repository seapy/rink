use rink::launcher::generate_kdl_layout;

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
}
