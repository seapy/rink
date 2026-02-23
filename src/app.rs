use crate::category::{self, SessionGroup};
use crate::command::Command;
use crate::config::Config;
use crate::search::SearchState;
use crate::sort::{self, SortMode};
use crate::state::PersistentState;
use crate::status::{self, ClaudeStatus};
use crate::tmux::{Session, TmuxClient};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Search,
    Help,
    ConfirmKill,
    Creating,
    Renaming,
    RenamingCategory,
}

/// Represents a visible item in the session list
#[derive(Debug, Clone)]
pub enum ListItem {
    CategoryHeader {
        category: String,
        session_count: usize,
        collapsed: bool,
    },
    SessionEntry {
        session: Session,
        category: String,
    },
}

/// Actions that the event loop should perform after executing a command
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    None,
    SwitchTmuxSession(String),
    Quit,
    Render,
}

pub struct App {
    pub mode: Mode,
    pub sessions: Vec<Session>,
    pub groups: Vec<SessionGroup>,
    pub visible_items: Vec<ListItem>,
    pub selected: usize,
    pub sort_mode: SortMode,
    pub search: SearchState,
    pub state: PersistentState,
    pub config: Config,
    pub input_buffer: String,
    pub kill_target: Option<String>,
    pub rename_target: Option<String>,
    pub rename_category_target: Option<String>,
    pub claude_statuses: HashMap<String, ClaudeStatus>,
    pub show_standalone: bool,
    pub category_focus: Option<String>,
    pub preview: String,
}

impl App {
    pub fn new(config: Config, standalone: bool) -> Self {
        let state = PersistentState::load();
        Self::with_state(config, standalone, state)
    }

    pub fn with_state(config: Config, standalone: bool, state: PersistentState) -> Self {
        let sort_mode = SortMode::from_str(&config.default_sort);

        Self {
            mode: Mode::Normal,
            sessions: Vec::new(),
            groups: Vec::new(),
            visible_items: Vec::new(),
            selected: 0,
            sort_mode,
            search: SearchState::default(),
            state,
            config,
            input_buffer: String::new(),
            kill_target: None,
            rename_target: None,
            rename_category_target: None,
            claude_statuses: HashMap::new(),
            show_standalone: standalone,
            category_focus: None,
            preview: String::new(),
        }
    }

    pub fn refresh_sessions(&mut self, client: &dyn TmuxClient) {
        self.sessions = client
            .list_sessions()
            .into_iter()
            .filter(|s| !s.name.starts_with("_rink_"))
            .collect();
        self.claude_statuses = status::read_all_statuses();
        self.rebuild_visible_items();
    }

    pub fn rebuild_visible_items(&mut self) {
        let sessions = if self.search.active && !self.search.query.is_empty() {
            category::filter_sessions(&self.sessions, &self.search.query)
        } else {
            self.sessions.clone()
        };

        // Sync state ordering
        let session_names: Vec<String> = sessions.iter().map(|s| s.name.clone()).collect();
        self.state.sync_session_order(&session_names);

        let mut groups = category::group_sessions(&sessions, &self.config.separator);

        let category_names: Vec<String> = groups.iter().map(|g| g.category.clone()).collect();
        self.state.sync_category_order(&category_names);

        sort::sort_groups(&mut groups, self.sort_mode, &self.state);
        self.groups = groups;

        // Build visible items list
        let mut items = Vec::new();

        // If we have a category focus, only show that category
        if let Some(ref focus_cat) = self.category_focus {
            for group in &self.groups {
                if &group.category == focus_cat {
                    items.push(ListItem::CategoryHeader {
                        category: group.category.clone(),
                        session_count: group.sessions.len(),
                        collapsed: false,
                    });
                    for session in &group.sessions {
                        items.push(ListItem::SessionEntry {
                            session: session.clone(),
                            category: group.category.clone(),
                        });
                    }
                    break;
                }
            }
        } else {
            for group in &self.groups {
                let collapsed = self.state.is_collapsed(&group.category);

                items.push(ListItem::CategoryHeader {
                    category: group.category.clone(),
                    session_count: group.sessions.len(),
                    collapsed,
                });

                if !collapsed {
                    for session in &group.sessions {
                        items.push(ListItem::SessionEntry {
                            session: session.clone(),
                            category: group.category.clone(),
                        });
                    }
                }
            }
        }

        self.visible_items = items;

        // Clamp selected index
        if !self.visible_items.is_empty() {
            self.selected = self.selected.min(self.visible_items.len() - 1);
        } else {
            self.selected = 0;
        }
    }

    pub fn execute(&mut self, cmd: Command, client: &dyn TmuxClient) -> Action {
        match cmd {
            Command::Quit => Action::Quit,
            Command::MoveUp => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                Action::Render
            }
            Command::MoveDown => {
                if !self.visible_items.is_empty() && self.selected < self.visible_items.len() - 1 {
                    self.selected += 1;
                }
                Action::Render
            }
            Command::Select => self.handle_select(client),
            Command::Search => {
                self.mode = Mode::Search;
                self.search.activate();
                Action::Render
            }
            Command::ToggleCollapse => self.handle_toggle_collapse(),
            Command::CollapseAll => {
                let categories: Vec<String> =
                    self.groups.iter().map(|g| g.category.clone()).collect();
                self.state.collapse_all(&categories);
                self.rebuild_visible_items();
                Action::Render
            }
            Command::ExpandAll => {
                self.state.expand_all();
                self.rebuild_visible_items();
                Action::Render
            }
            Command::Refresh => {
                self.refresh_sessions(client);
                Action::Render
            }
            Command::CreateSession => {
                self.mode = Mode::Creating;
                // Auto-fill category prefix if a category header or session in a category is selected
                self.input_buffer = self.get_current_category_prefix();
                Action::Render
            }
            Command::KillSession => self.handle_kill_init(),
            Command::RenameSession => self.handle_rename_init(),
            Command::RenameCategory => self.handle_rename_category_init(),
            Command::MoveSessionUp => self.handle_move_session(-1),
            Command::MoveSessionDown => self.handle_move_session(1),
            Command::CycleSortMode => {
                self.sort_mode = self.sort_mode.next();
                self.rebuild_visible_items();
                Action::Render
            }
            Command::ShowHelp => {
                self.mode = if self.mode == Mode::Help {
                    Mode::Normal
                } else {
                    Mode::Help
                };
                Action::Render
            }
            Command::Escape => self.handle_escape(),
            Command::InputChar(c) => self.handle_input_char(c),
            Command::InputBackspace => self.handle_input_backspace(),
            Command::InputConfirm => self.handle_input_confirm(client),
            Command::None => Action::None,
        }
    }

    fn handle_select(&mut self, client: &dyn TmuxClient) -> Action {
        if self.visible_items.is_empty() {
            return Action::None;
        }

        match &self.visible_items[self.selected] {
            ListItem::CategoryHeader { category, .. } => {
                if self.category_focus.as_deref() == Some(category) {
                    // Already focused, unfocus
                    self.category_focus = None;
                } else {
                    self.category_focus = Some(category.clone());
                }
                self.rebuild_visible_items();
                Action::Render
            }
            ListItem::SessionEntry { session, .. } => {
                let name = session.name.clone();
                client.switch_client(&name);
                self.refresh_sessions(client);
                Action::SwitchTmuxSession(name)
            }
        }
    }

    fn handle_toggle_collapse(&mut self) -> Action {
        if self.visible_items.is_empty() {
            return Action::None;
        }

        // Find the category of the currently selected item
        let category = match &self.visible_items[self.selected] {
            ListItem::CategoryHeader { category, .. } => category.clone(),
            ListItem::SessionEntry { category, .. } => category.clone(),
        };

        self.state.toggle_collapsed(&category);
        self.rebuild_visible_items();
        Action::Render
    }

    fn handle_kill_init(&mut self) -> Action {
        if self.visible_items.is_empty() {
            return Action::None;
        }

        match &self.visible_items[self.selected] {
            ListItem::SessionEntry { session, .. } => {
                self.kill_target = Some(session.name.clone());
                self.mode = Mode::ConfirmKill;
                Action::Render
            }
            ListItem::CategoryHeader { category, .. } => {
                // Kill all sessions in category
                self.kill_target = Some(format!("category:{}", category));
                self.mode = Mode::ConfirmKill;
                Action::Render
            }
        }
    }

    fn handle_rename_init(&mut self) -> Action {
        if self.visible_items.is_empty() {
            return Action::None;
        }

        if let ListItem::SessionEntry { session, .. } = &self.visible_items[self.selected] {
            self.rename_target = Some(session.name.clone());
            self.input_buffer = session.name.clone();
            self.mode = Mode::Renaming;
        }
        Action::Render
    }

    fn handle_rename_category_init(&mut self) -> Action {
        if self.visible_items.is_empty() {
            return Action::None;
        }

        let category = match &self.visible_items[self.selected] {
            ListItem::CategoryHeader { category, .. } => category.clone(),
            ListItem::SessionEntry { category, .. } => category.clone(),
        };

        if !category.is_empty() {
            self.rename_category_target = Some(category.clone());
            self.input_buffer = category;
            self.mode = Mode::RenamingCategory;
        }
        Action::Render
    }

    fn handle_move_session(&mut self, direction: i32) -> Action {
        if self.sort_mode != SortMode::Custom {
            return Action::None;
        }

        if let Some(ListItem::SessionEntry { session, .. }) =
            self.visible_items.get(self.selected)
        {
            let name = session.name.clone();
            self.state.move_session(&name, direction);
            // Adjust selection to follow the moved item
            let new_selected = (self.selected as i32 + direction)
                .clamp(0, self.visible_items.len() as i32 - 1)
                as usize;
            self.selected = new_selected;
            self.rebuild_visible_items();
        }
        Action::Render
    }

    fn handle_escape(&mut self) -> Action {
        match self.mode {
            Mode::Search => {
                self.search.clear();
                self.mode = Mode::Normal;
                self.rebuild_visible_items();
            }
            Mode::Help | Mode::ConfirmKill | Mode::Creating | Mode::Renaming
            | Mode::RenamingCategory => {
                self.mode = Mode::Normal;
                self.input_buffer.clear();
                self.kill_target = None;
                self.rename_target = None;
                self.rename_category_target = None;
            }
            Mode::Normal => {
                if self.category_focus.is_some() {
                    self.category_focus = None;
                    self.rebuild_visible_items();
                }
            }
        }
        Action::Render
    }

    fn handle_input_char(&mut self, c: char) -> Action {
        match self.mode {
            Mode::Search => {
                self.search.push(c);
                self.rebuild_visible_items();
            }
            Mode::Creating | Mode::Renaming | Mode::RenamingCategory => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
        Action::Render
    }

    fn handle_input_backspace(&mut self) -> Action {
        match self.mode {
            Mode::Search => {
                self.search.pop();
                self.rebuild_visible_items();
            }
            Mode::Creating | Mode::Renaming | Mode::RenamingCategory => {
                self.input_buffer.pop();
            }
            _ => {}
        }
        Action::Render
    }

    fn handle_input_confirm(&mut self, client: &dyn TmuxClient) -> Action {
        match self.mode {
            Mode::Search => {
                self.mode = Mode::Normal;
                // Keep the search filter active but switch to normal navigation
                return Action::Render;
            }
            Mode::Creating => {
                let name = self.input_buffer.trim().to_string();
                if !name.is_empty() {
                    client.new_session(&name);
                    client.switch_client(&name);
                    self.input_buffer.clear();
                    self.mode = Mode::Normal;
                    self.refresh_sessions(client);
                    return Action::SwitchTmuxSession(name);
                }
                self.mode = Mode::Normal;
            }
            Mode::Renaming => {
                let new_name = self.input_buffer.trim().to_string();
                if let Some(old_name) = self.rename_target.take() {
                    if !new_name.is_empty() && new_name != old_name {
                        client.rename_session(&old_name, &new_name);
                        self.refresh_sessions(client);
                    }
                }
                self.input_buffer.clear();
                self.mode = Mode::Normal;
            }
            Mode::RenamingCategory => {
                let new_category = self.input_buffer.trim().to_string();
                if let Some(old_category) = self.rename_category_target.take() {
                    if !new_category.is_empty() && new_category != old_category {
                        // Rename all sessions in this category
                        let sessions_to_rename: Vec<String> = self
                            .sessions
                            .iter()
                            .filter(|s| {
                                category::extract_category(&s.name, &self.config.separator)
                                    == old_category
                            })
                            .map(|s| s.name.clone())
                            .collect();

                        for session_name in sessions_to_rename {
                            let suffix = category::extract_session_suffix(
                                &session_name,
                                &self.config.separator,
                            );
                            let new_name =
                                format!("{}{}{}", new_category, self.config.separator, suffix);
                            client.rename_session(&session_name, &new_name);
                        }
                        self.refresh_sessions(client);
                    }
                }
                self.input_buffer.clear();
                self.mode = Mode::Normal;
            }
            Mode::ConfirmKill => {
                if let Some(target) = self.kill_target.take() {
                    // Determine what session the right pane is currently viewing
                    let current = client.current_session();

                    // Collect sessions to kill
                    let sessions_to_kill: Vec<String> =
                        if let Some(cat) = target.strip_prefix("category:") {
                            self.sessions
                                .iter()
                                .filter(|s| {
                                    category::extract_category(&s.name, &self.config.separator)
                                        == cat
                                })
                                .map(|s| s.name.clone())
                                .collect()
                        } else {
                            vec![target.clone()]
                        };

                    // Check if the current session is being killed
                    let current_being_killed = current
                        .as_ref()
                        .map(|c| sessions_to_kill.contains(c))
                        .unwrap_or(false);

                    // Find a fallback session before killing
                    let fallback = if current_being_killed {
                        self.sessions
                            .iter()
                            .filter(|s| !s.name.starts_with("_rink_"))
                            .filter(|s| !sessions_to_kill.contains(&s.name))
                            .map(|s| s.name.clone())
                            .next()
                    } else {
                        None
                    };

                    // Switch to fallback BEFORE killing, so the client stays alive
                    if let Some(ref fallback_name) = fallback {
                        client.switch_client(fallback_name);
                    }

                    // Now kill the sessions
                    for name in &sessions_to_kill {
                        client.kill_session(name);
                    }

                    self.refresh_sessions(client);
                    self.mode = Mode::Normal;

                    if let Some(fallback_name) = fallback {
                        return Action::SwitchTmuxSession(fallback_name);
                    }
                }
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Action::Render
    }

    fn get_current_category_prefix(&self) -> String {
        if self.visible_items.is_empty() {
            return String::new();
        }

        let category = match &self.visible_items[self.selected] {
            ListItem::CategoryHeader { category, .. } => category.clone(),
            ListItem::SessionEntry { category, .. } => category.clone(),
        };

        if category.is_empty() {
            String::new()
        } else {
            format!("{}{}", category, self.config.separator)
        }
    }

    pub fn selected_session_name(&self) -> Option<String> {
        if let Some(ListItem::SessionEntry { session, .. }) =
            self.visible_items.get(self.selected)
        {
            Some(session.name.clone())
        } else {
            None
        }
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn update_preview(&mut self, client: &dyn TmuxClient) {
        if let Some(name) = self.selected_session_name() {
            match client.capture_pane(&name) {
                Some(content) => self.preview = content,
                None => self.preview = "(preview unavailable)".to_string(),
            }
        } else {
            self.preview.clear();
        }
    }
}
