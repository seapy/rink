use crate::app::{App, ListItem, Mode};
use crate::status::ClaudeStatus;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem as RatatuiListItem, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(1),   // Main content
            Constraint::Length(2), // Footer
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_main(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);

    // Render popups on top
    match app.mode {
        Mode::Help => render_help_popup(frame),
        Mode::ConfirmKill => render_confirm_popup(frame, app),
        Mode::Creating => render_input_popup(frame, "New Session", &app.input_buffer),
        Mode::Renaming => render_input_popup(frame, "Rename Session", &app.input_buffer),
        Mode::RenamingCategory => {
            render_input_popup(frame, "Rename Category", &app.input_buffer)
        }
        _ => {}
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let sort_label = format!(" Sort: {} ", app.sort_mode);

    let mut header_spans = vec![
        Span::styled(" rink ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(
            format!("[{}]", sort_label.trim()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{} sessions", app.session_count()),
            Style::default().fg(Color::DarkGray),
        ),
    ];

    // Claude status legend
    let has_claude = !app.claude_statuses.is_empty();
    if has_claude {
        header_spans.push(Span::raw("  "));
        header_spans.push(Span::styled("*", Style::default().fg(Color::Green)));
        header_spans.push(Span::styled("work ", Style::default().fg(Color::DarkGray)));
        header_spans.push(Span::styled("?", Style::default().fg(Color::Yellow)));
        header_spans.push(Span::styled("wait ", Style::default().fg(Color::DarkGray)));
        header_spans.push(Span::styled("+", Style::default().fg(Color::Blue)));
        header_spans.push(Span::styled("done", Style::default().fg(Color::DarkGray)));
    }

    if let Some(ref focus) = app.category_focus {
        header_spans.push(Span::raw("  "));
        header_spans.push(Span::styled(
            format!("[Focus: {}]", focus),
            Style::default().fg(Color::Magenta),
        ));
    }

    let header = Paragraph::new(Line::from(header_spans)).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(header, area);
}

fn render_main(frame: &mut Frame, app: &App, area: Rect) {
    // Split main area into session list and preview
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Session list
            Constraint::Min(5),    // Preview
        ])
        .split(area);

    render_session_list(frame, app, chunks[0]);
    render_preview(frame, app, chunks[1]);
}

fn render_session_list(frame: &mut Frame, app: &App, area: Rect) {
    if app.visible_items.is_empty() {
        let empty_msg = if app.search.active {
            "No matching sessions"
        } else {
            "No tmux sessions found. Press 'c' to create one."
        };
        let paragraph = Paragraph::new(empty_msg)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default());
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<RatatuiListItem> = app
        .visible_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == app.selected;
            match item {
                ListItem::CategoryHeader {
                    category,
                    session_count,
                    collapsed,
                } => {
                    let arrow = if *collapsed { "▸" } else { "▾" };
                    let display_name = if category.is_empty() {
                        "General".to_string()
                    } else {
                        category.clone()
                    };
                    let is_focused = app.category_focus.as_deref() == Some(category);
                    let mut style = Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD);
                    if is_selected {
                        style = style.add_modifier(Modifier::REVERSED);
                    } else if is_focused {
                        style = style.add_modifier(Modifier::UNDERLINED);
                    }
                    let marker = if is_focused { " <<" } else { "" };
                    let line = Line::from(vec![
                        Span::styled(
                            format!(" {} {} ({}){}", arrow, display_name, session_count, marker),
                            style,
                        ),
                    ]);
                    RatatuiListItem::new(line)
                }
                ListItem::SessionEntry {
                    session, category, ..
                } => {
                    let icon = if session.attached { "●" } else { "○" };
                    let display_name = if category.is_empty() {
                        session.name.clone()
                    } else {
                        crate::category::extract_session_suffix(
                            &session.name,
                            &app.config.separator,
                        )
                    };

                    let session_color = if session.attached {
                        Color::Green
                    } else {
                        Color::White
                    };
                    let sel_mod = if is_selected {
                        Modifier::REVERSED
                    } else {
                        Modifier::empty()
                    };

                    let indent = "   ";
                    let mut spans = vec![
                        Span::styled(
                            format!("{}{} ", indent, icon),
                            Style::default().fg(session_color).add_modifier(sel_mod),
                        ),
                        Span::styled(
                            display_name,
                            Style::default().fg(session_color).add_modifier(sel_mod),
                        ),
                    ];

                    // Claude status
                    if let Some(status) = app.claude_statuses.get(&session.name) {
                        let (claude_icon, color) = match status {
                            ClaudeStatus::Working => ("*", Color::Yellow),
                            ClaudeStatus::Waiting => ("?", Color::Cyan),
                            ClaudeStatus::Done => ("+", Color::Green),
                            ClaudeStatus::Unknown => ("", Color::DarkGray),
                        };
                        if !claude_icon.is_empty() {
                            spans.push(Span::styled(
                                format!(" {}", claude_icon),
                                Style::default().fg(color).add_modifier(sel_mod),
                            ));
                        }
                    }

                    let style = if is_selected {
                        Style::default()
                    } else {
                        Style::default()
                    };

                    RatatuiListItem::new(Line::from(spans)).style(style)
                }
            }
        })
        .collect();

    let list = List::new(items).block(Block::default());
    frame.render_widget(list, area);
}

fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .title(" Preview ")
        .border_style(Style::default().fg(Color::DarkGray));

    let inner_height = block.inner(area).height as usize;

    let content = if app.preview.is_empty() {
        if app.selected_session_name().is_none() {
            "(no session selected)".to_string()
        } else {
            "(no preview)".to_string()
        }
    } else {
        // Show the bottom N lines to display the latest output
        let lines: Vec<&str> = app.preview.lines().collect();
        if lines.len() > inner_height {
            lines[lines.len() - inner_height..].join("\n")
        } else {
            app.preview.clone()
        }
    };

    let preview = Paragraph::new(content)
        .style(Style::default().fg(Color::Yellow))
        .block(block);

    frame.render_widget(preview, area);
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let hints = match app.mode {
        Mode::Search => vec![
            ("Enter", "confirm"),
            ("Esc", "cancel"),
            ("Type", "to filter"),
        ],
        Mode::Normal => {
            let mut h = vec![
                ("↑↓", "nav"),
                ("Enter", "switch"),
                ("/", "search"),
                ("c", "create"),
                ("x", "kill"),
                ("s", "sort"),
                ("?", "help"),
            ];
            if app.category_focus.is_some() {
                h.push(("Esc", "unfocus"));
            }
            h
        }
        Mode::Help => vec![("Esc/?", "close")],
        Mode::ConfirmKill => vec![("y", "confirm"), ("n/Esc", "cancel")],
        Mode::Creating | Mode::Renaming | Mode::RenamingCategory => {
            vec![("Enter", "confirm"), ("Esc", "cancel")]
        }
    };

    let spans: Vec<Span> = hints
        .iter()
        .enumerate()
        .flat_map(|(i, (key, desc))| {
            let mut s = vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::DarkGray),
                ),
                Span::styled(format!(" {} ", desc), Style::default().fg(Color::DarkGray)),
            ];
            if i < hints.len() - 1 {
                s.push(Span::raw(" "));
            }
            s
        })
        .collect();

    // Search bar
    let content = if app.mode == Mode::Search {
        let search_line = Line::from(vec![
            Span::styled(" / ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(&app.search.query),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]);

        let footer_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(area);

        frame.render_widget(Paragraph::new(search_line), footer_chunks[0]);
        frame.render_widget(Paragraph::new(Line::from(spans)), footer_chunks[1]);
        return;
    } else {
        Line::from(spans)
    };

    let footer = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(footer, area);
}

fn render_help_popup(frame: &mut Frame) {
    let area = centered_rect(60, 80, frame.area());
    frame.render_widget(Clear, area);

    let help_text = vec![
        Line::from(vec![Span::styled(
            " Keybindings ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        help_line("↑/↓ k/j", "Navigate up/down"),
        help_line("Enter", "Switch session / Focus category"),
        help_line("/", "Search sessions"),
        help_line("Tab/←/→", "Toggle collapse"),
        help_line("r", "Refresh sessions"),
        help_line("c", "Create session (auto-prefix)"),
        help_line("x", "Kill session/category"),
        help_line("R", "Rename session"),
        help_line("C", "Rename category (batch)"),
        help_line("s", "Cycle sort mode"),
        help_line("J/K", "Reorder (Custom sort)"),
        help_line("Esc", "Cancel / Unfocus"),
        help_line("Ctrl+x", "Quit"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Press ? or Esc to close ",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let popup = Paragraph::new(help_text).block(
        Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(popup, area);
}

fn render_confirm_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area);

    let target = app.kill_target.as_deref().unwrap_or("?");
    let msg = if let Some(cat) = target.strip_prefix("category:") {
        format!("Kill all sessions in '{}'?", cat)
    } else {
        format!("Kill session '{}'?", target)
    };

    let popup = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            msg,
            Style::default().fg(Color::Red),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" y ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw("es  "),
            Span::styled(" n ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("o"),
        ]),
    ])
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .title(" Confirm ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Red)),
    );
    frame.render_widget(popup, area);
}

fn render_input_popup(frame: &mut Frame, title: &str, buffer: &str) {
    let area = centered_rect(60, 20, frame.area());
    frame.render_widget(Clear, area);

    let popup = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::raw(" "),
            Span::raw(buffer),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]),
    ])
    .block(
        Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(popup, area);
}

fn help_line<'a>(key: &'a str, desc: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("  {:14}", key),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(desc),
    ])
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
