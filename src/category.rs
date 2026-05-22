use crate::tmux::Session;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct SessionGroup {
    pub category: String,
    pub sessions: Vec<Session>,
}

/// Extract the category prefix from a session name using the separator.
/// e.g., "work-project" with separator "-" -> "work"
/// e.g., "standalone" with separator "-" -> "" (no category)
pub fn extract_category(session_name: &str, separator: &str) -> String {
    if let Some(pos) = session_name.find(separator) {
        session_name[..pos].to_string()
    } else {
        String::new()
    }
}

/// Extract the session name part after the category prefix.
/// e.g., "work-project" with separator "-" -> "project"
/// e.g., "standalone" with separator "-" -> "standalone"
pub fn extract_session_suffix(session_name: &str, separator: &str) -> String {
    if let Some(pos) = session_name.find(separator) {
        session_name[pos + separator.len()..].to_string()
    } else {
        session_name.to_string()
    }
}

/// Group sessions by their category prefix.
/// Sessions without a category go into the "" (empty) group.
pub fn group_sessions(sessions: &[Session], separator: &str) -> Vec<SessionGroup> {
    let mut groups: BTreeMap<String, Vec<Session>> = BTreeMap::new();

    for session in sessions {
        let category = extract_category(&session.name, separator);
        groups.entry(category).or_default().push(session.clone());
    }

    groups
        .into_iter()
        .map(|(category, sessions)| SessionGroup { category, sessions })
        .collect()
}

/// Filter sessions by a search query (case-insensitive substring match).
pub fn filter_sessions(sessions: &[Session], query: &str) -> Vec<Session> {
    if query.is_empty() {
        return sessions.to_vec();
    }
    let query_lower = query.to_lowercase();
    sessions
        .iter()
        .filter(|s| s.name.to_lowercase().contains(&query_lower))
        .cloned()
        .collect()
}
