use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersistentState {
    #[serde(default)]
    pub session_order: Vec<String>,
    #[serde(default)]
    pub category_order: Vec<String>,
    #[serde(default)]
    pub collapsed_categories: Vec<String>,
}

impl PersistentState {
    fn state_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rink")
            .join("state.toml")
    }

    pub fn load() -> Self {
        let path = Self::state_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => toml::from_str(&content).unwrap_or_default(),
                Err(_) => PersistentState::default(),
            }
        } else {
            PersistentState::default()
        }
    }

    pub fn save(&self) {
        let path = Self::state_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(content) = toml::to_string_pretty(self) {
            let _ = std::fs::write(&path, content);
        }
    }

    /// Ensure all sessions are present in the order list.
    /// New sessions are appended at the end.
    pub fn sync_session_order(&mut self, session_names: &[String]) {
        // Remove sessions that no longer exist
        self.session_order
            .retain(|name| session_names.contains(name));
        // Add new sessions
        for name in session_names {
            if !self.session_order.contains(name) {
                self.session_order.push(name.clone());
            }
        }
    }

    /// Ensure all categories are present in the order list.
    pub fn sync_category_order(&mut self, category_names: &[String]) {
        self.category_order
            .retain(|name| category_names.contains(name));
        for name in category_names {
            if !self.category_order.contains(name) {
                self.category_order.push(name.clone());
            }
        }
    }

    pub fn move_session(&mut self, session_name: &str, direction: i32) {
        if let Some(pos) = self.session_order.iter().position(|s| s == session_name) {
            let new_pos =
                (pos as i32 + direction).clamp(0, self.session_order.len() as i32 - 1) as usize;
            if new_pos != pos {
                let item = self.session_order.remove(pos);
                self.session_order.insert(new_pos, item);
                self.save();
            }
        }
    }

    pub fn is_collapsed(&self, category: &str) -> bool {
        self.collapsed_categories.contains(&category.to_string())
    }

    pub fn toggle_collapsed(&mut self, category: &str) {
        if self.is_collapsed(category) {
            self.collapsed_categories.retain(|c| c != category);
        } else {
            self.collapsed_categories.push(category.to_string());
        }
        self.save();
    }

    pub fn collapse_all(&mut self, categories: &[String]) {
        self.collapsed_categories = categories.to_vec();
        self.save();
    }

    pub fn expand_all(&mut self) {
        self.collapsed_categories.clear();
        self.save();
    }
}
