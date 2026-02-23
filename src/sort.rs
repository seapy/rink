use crate::category::SessionGroup;
use crate::state::PersistentState;
use crate::tmux::Session;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    #[default]
    Name,
    Recent,
    Windows,
    Custom,
}

impl SortMode {
    pub fn next(self) -> Self {
        match self {
            SortMode::Name => SortMode::Recent,
            SortMode::Recent => SortMode::Windows,
            SortMode::Windows => SortMode::Custom,
            SortMode::Custom => SortMode::Name,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "recent" => SortMode::Recent,
            "windows" => SortMode::Windows,
            "custom" => SortMode::Custom,
            _ => SortMode::Name,
        }
    }
}

impl fmt::Display for SortMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortMode::Name => write!(f, "Name"),
            SortMode::Recent => write!(f, "Recent"),
            SortMode::Windows => write!(f, "Windows"),
            SortMode::Custom => write!(f, "Custom"),
        }
    }
}

pub fn sort_sessions(sessions: &mut [Session], mode: SortMode) {
    match mode {
        SortMode::Name => {
            sessions.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        }
        SortMode::Recent => {
            sessions.sort_by(|a, b| {
                b.last_attached
                    .unwrap_or(0)
                    .cmp(&a.last_attached.unwrap_or(0))
            });
        }
        SortMode::Windows => {
            sessions.sort_by(|a, b| b.windows.cmp(&a.windows));
        }
        SortMode::Custom => {
            // Custom sort is handled by the state module
        }
    }
}

pub fn sort_groups(groups: &mut Vec<SessionGroup>, mode: SortMode, state: &PersistentState) {
    match mode {
        SortMode::Custom => {
            groups.sort_by(|a, b| {
                let a_idx = state
                    .category_order
                    .iter()
                    .position(|c| c == &a.category)
                    .unwrap_or(usize::MAX);
                let b_idx = state
                    .category_order
                    .iter()
                    .position(|c| c == &b.category)
                    .unwrap_or(usize::MAX);
                a_idx.cmp(&b_idx)
            });
        }
        _ => {
            // For non-custom modes, sort groups alphabetically by category name
            // Empty category always comes last
            groups.sort_by(|a, b| {
                if a.category.is_empty() {
                    std::cmp::Ordering::Greater
                } else if b.category.is_empty() {
                    std::cmp::Ordering::Less
                } else {
                    a.category.to_lowercase().cmp(&b.category.to_lowercase())
                }
            });
        }
    }

    // Sort sessions within each group
    if mode != SortMode::Custom {
        for group in groups.iter_mut() {
            sort_sessions(&mut group.sessions, mode);
        }
    } else {
        for group in groups.iter_mut() {
            let order = &state.session_order;
            group.sessions.sort_by(|a, b| {
                let a_idx = order.iter().position(|s| s == &a.name).unwrap_or(usize::MAX);
                let b_idx = order.iter().position(|s| s == &b.name).unwrap_or(usize::MAX);
                a_idx.cmp(&b_idx)
            });
        }
    }
}
