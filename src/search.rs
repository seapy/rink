use crate::tmux::Session;

#[derive(Debug, Default, Clone)]
pub struct SearchState {
    pub query: String,
    pub active: bool,
}

impl SearchState {
    pub fn push(&mut self, c: char) {
        self.query.push(c);
    }

    pub fn pop(&mut self) {
        self.query.pop();
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.active = false;
    }

    pub fn activate(&mut self) {
        self.active = true;
        self.query.clear();
    }

    pub fn filter<'a>(&self, sessions: &'a [Session]) -> Vec<&'a Session> {
        if self.query.is_empty() {
            return sessions.iter().collect();
        }
        let query_lower = self.query.to_lowercase();
        sessions
            .iter()
            .filter(|s| s.name.to_lowercase().contains(&query_lower))
            .collect()
    }
}
