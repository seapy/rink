use ratatui::backend::TestBackend;
use ratatui::Terminal;
use rink::app::{App, Mode};
use rink::command::Command;
use rink::config::Config;
use rink::sort::SortMode;
use rink::state::PersistentState;
use rink::status::ClaudeStatus;
use rink::tmux::{FakeTmuxClient, Session};

fn make_sessions() -> Vec<Session> {
    vec![
        Session {
            name: "work-api".to_string(),
            windows: 3,
            attached: true,
            last_attached: Some(1700000000),
            created: Some(1699000000),
        },
        Session {
            name: "work-frontend".to_string(),
            windows: 2,
            attached: false,
            last_attached: Some(1699999000),
            created: Some(1699000000),
        },
        Session {
            name: "work-db".to_string(),
            windows: 1,
            attached: false,
            last_attached: Some(1699998000),
            created: Some(1699000000),
        },
        Session {
            name: "personal-blog".to_string(),
            windows: 1,
            attached: false,
            last_attached: Some(1699997000),
            created: Some(1699000000),
        },
        Session {
            name: "personal-dotfiles".to_string(),
            windows: 1,
            attached: false,
            last_attached: Some(1699996000),
            created: Some(1699000000),
        },
        Session {
            name: "scratch".to_string(),
            windows: 1,
            attached: false,
            last_attached: Some(1699995000),
            created: Some(1699000000),
        },
    ]
}

fn setup_app(sessions: Vec<Session>) -> (App, FakeTmuxClient) {
    let config = Config::default();
    let state = PersistentState::default();
    let mut app = App::with_state(config, true, state);
    let client = FakeTmuxClient::new(sessions);
    app.refresh_sessions(&client);
    app.claude_statuses
        .insert("dummy-status".to_string(), ClaudeStatus::Working);
    (app, client)
}

fn render_to_string(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            rink::ui::render(frame, app);
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let mut output = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            let cell = &buf[(x, y)];
            output.push_str(cell.symbol());
        }
        output.push('\n');
    }
    output
}

#[test]
fn test_normal_view() {
    let (app, _) = setup_app(make_sessions());
    let output = render_to_string(&app, 50, 20);
    insta::assert_snapshot!("normal_view", output);
}

#[test]
fn test_search_mode() {
    let (mut app, client) = setup_app(make_sessions());
    app.execute(Command::Search, &client);
    app.execute(Command::InputChar('w'), &client);
    app.execute(Command::InputChar('o'), &client);
    app.execute(Command::InputChar('r'), &client);
    app.execute(Command::InputChar('k'), &client);
    let output = render_to_string(&app, 50, 20);
    insta::assert_snapshot!("search_mode", output);
}

#[test]
fn test_collapsed_category() {
    let (mut app, client) = setup_app(make_sessions());
    // Collapse the first category (work)
    app.execute(Command::ToggleCollapse, &client);
    let output = render_to_string(&app, 50, 20);
    insta::assert_snapshot!("collapsed_category", output);
}

#[test]
fn test_help_view() {
    let (mut app, client) = setup_app(make_sessions());
    app.execute(Command::ShowHelp, &client);
    let output = render_to_string(&app, 50, 20);
    insta::assert_snapshot!("help_view", output);
}

#[test]
fn test_confirm_kill() {
    let (mut app, client) = setup_app(make_sessions());
    // Move to a session entry
    app.execute(Command::MoveDown, &client);
    app.execute(Command::KillSession, &client);
    let output = render_to_string(&app, 50, 20);
    insta::assert_snapshot!("confirm_kill", output);
}

#[test]
fn test_empty_sessions() {
    let (app, _) = setup_app(vec![]);
    let output = render_to_string(&app, 50, 20);
    insta::assert_snapshot!("empty_sessions", output);
}

#[test]
fn test_create_session() {
    let (mut app, client) = setup_app(make_sessions());
    app.execute(Command::CreateSession, &client);
    let output = render_to_string(&app, 50, 20);
    insta::assert_snapshot!("create_session", output);
}

#[test]
fn test_selected_session() {
    let (mut app, client) = setup_app(make_sessions());
    app.execute(Command::MoveDown, &client);
    app.execute(Command::MoveDown, &client);
    let output = render_to_string(&app, 50, 20);
    insta::assert_snapshot!("selected_session", output);
}

#[test]
fn test_sort_indicator() {
    let (mut app, client) = setup_app(make_sessions());
    app.execute(Command::CycleSortMode, &client);
    assert_eq!(app.sort_mode, SortMode::Recent);
    let output = render_to_string(&app, 50, 20);
    insta::assert_snapshot!("sort_indicator", output);
}

#[test]
fn test_category_rename() {
    let (mut app, client) = setup_app(make_sessions());
    // Select the "personal" category header
    // First item is "personal" category, then sessions, then "work" category
    // With BTreeMap ordering: "" (ungrouped) comes first if exists, then "personal", then "work"
    app.execute(Command::RenameCategory, &client);
    // Type new name
    app.execute(Command::InputChar('d'), &client);
    app.execute(Command::InputChar('e'), &client);
    app.execute(Command::InputChar('v'), &client);
    let output = render_to_string(&app, 50, 20);
    insta::assert_snapshot!("category_rename", output);
}

// Unit tests for core logic
#[test]
fn test_category_extraction() {
    use rink::category::{extract_category, extract_session_suffix};

    assert_eq!(extract_category("work-api", "-"), "work");
    assert_eq!(extract_category("work-frontend", "-"), "work");
    assert_eq!(extract_category("scratch", "-"), "");
    assert_eq!(extract_session_suffix("work-api", "-"), "api");
    assert_eq!(extract_session_suffix("scratch", "-"), "scratch");
}

#[test]
fn test_sort_modes() {
    assert_eq!(SortMode::Name.next(), SortMode::Recent);
    assert_eq!(SortMode::Recent.next(), SortMode::Windows);
    assert_eq!(SortMode::Windows.next(), SortMode::Custom);
    assert_eq!(SortMode::Custom.next(), SortMode::Name);
}

#[test]
fn test_command_translate() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use rink::keys::translate;

    let mode = Mode::Normal;
    assert_eq!(
        translate(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), &mode),
        Command::MoveUp
    );
    assert_eq!(
        translate(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE), &mode),
        Command::MoveDown
    );
    assert_eq!(
        translate(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), &mode),
        Command::Select
    );
    assert_eq!(
        translate(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::NONE), &mode),
        Command::Search
    );
    assert_eq!(
        translate(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL),
            &mode
        ),
        Command::Quit
    );
}

#[test]
fn test_search_filter() {
    use rink::search::SearchState;

    let sessions = make_sessions();
    let mut search = SearchState::default();
    search.activate();
    search.push('w');
    search.push('o');
    search.push('r');
    search.push('k');

    let filtered = search.filter(&sessions);
    assert_eq!(filtered.len(), 3);
    assert!(filtered.iter().all(|s| s.name.contains("work")));
}

#[test]
fn test_persistent_state_sync() {
    use rink::state::PersistentState;

    let mut state = PersistentState::default();
    let names = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    state.sync_session_order(&names);
    assert_eq!(state.session_order, names);

    // Remove "b", add "d"
    let names2 = vec!["a".to_string(), "c".to_string(), "d".to_string()];
    state.sync_session_order(&names2);
    assert_eq!(state.session_order, vec!["a", "c", "d"]);
}
