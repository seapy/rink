use std::process::Command;

#[derive(Debug, Clone)]
pub struct Session {
    pub name: String,
    pub windows: u32,
    pub attached: bool,
    pub last_attached: Option<u64>,
    pub created: Option<u64>,
}

pub trait TmuxClient {
    fn list_sessions(&self) -> Vec<Session>;
    fn current_session(&self) -> Option<String>;
    fn switch_client(&self, session_name: &str) -> bool;
    fn new_session(&self, session_name: &str) -> bool;
    fn kill_session(&self, session_name: &str) -> bool;
    fn rename_session(&self, old_name: &str, new_name: &str) -> bool;
    fn capture_pane(&self, session_name: &str) -> Option<String>;
}

pub struct RealTmuxClient;

impl RealTmuxClient {
    pub fn new() -> Self {
        Self
    }

    fn run_tmux(&self, args: &[&str]) -> Option<String> {
        let output = Command::new("tmux").args(args).output().ok()?;
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }
}

impl TmuxClient for RealTmuxClient {
    fn list_sessions(&self) -> Vec<Session> {
        let format = "#{session_name}\t#{session_windows}\t#{session_activity}\t#{session_created}";
        let output = self.run_tmux(&["list-sessions", "-F", format]);
        let current = self.current_session();

        match output {
            Some(text) => text
                .lines()
                .filter(|line| !line.is_empty())
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if parts.len() >= 4 {
                        let name = parts[0].to_string();
                        let attached = current.as_deref() == Some(&name);
                        Some(Session {
                            name,
                            windows: parts[1].parse().unwrap_or(1),
                            attached,
                            last_attached: parts[2].parse().ok(),
                            created: parts[3].parse().ok(),
                        })
                    } else {
                        None
                    }
                })
                .collect(),
            None => Vec::new(),
        }
    }

    fn current_session(&self) -> Option<String> {
        // Check which session the right-pane client is viewing
        let tty_path = crate::launcher::client_tty_path();
        if let Ok(tty) = std::fs::read_to_string(&tty_path) {
            let tty = tty.trim();
            if !tty.is_empty() {
                if let Some(output) =
                    self.run_tmux(&["display-message", "-c", tty, "-p", "#{session_name}"])
                {
                    let name = output.trim().to_string();
                    if !name.is_empty() {
                        return Some(name);
                    }
                }
            }
        }
        None
    }

    fn switch_client(&self, session_name: &str) -> bool {
        // Use = prefix for exact session name matching (prevents tmux prefix ambiguity)
        let exact_target = format!("={}", session_name);
        // Read the tty of the right zellij pane (written at startup)
        let tty_path = crate::launcher::client_tty_path();
        if let Ok(tty) = std::fs::read_to_string(&tty_path) {
            let tty = tty.trim();
            if !tty.is_empty() {
                return self
                    .run_tmux(&["switch-client", "-c", tty, "-t", &exact_target])
                    .is_some();
            }
        }
        // Fallback: try without specifying client
        self.run_tmux(&["switch-client", "-t", &exact_target])
            .is_some()
    }

    fn new_session(&self, session_name: &str) -> bool {
        self.run_tmux(&["new-session", "-d", "-s", session_name])
            .is_some()
    }

    fn kill_session(&self, session_name: &str) -> bool {
        let exact_target = format!("={}", session_name);
        self.run_tmux(&["kill-session", "-t", &exact_target])
            .is_some()
    }

    fn rename_session(&self, old_name: &str, new_name: &str) -> bool {
        let exact_target = format!("={}", old_name);
        self.run_tmux(&["rename-session", "-t", &exact_target, new_name])
            .is_some()
    }

    fn capture_pane(&self, session_name: &str) -> Option<String> {
        // Use "session:" format for pane target (= prefix doesn't work with capture-pane)
        let pane_target = format!("{}:", session_name);
        self.run_tmux(&[
            "capture-pane",
            "-pt",
            &pane_target,
            "-S",
            "-100",
        ])
    }
}

#[derive(Default)]
pub struct FakeTmuxClient {
    pub sessions: Vec<Session>,
}

impl FakeTmuxClient {
    pub fn new(sessions: Vec<Session>) -> Self {
        Self { sessions }
    }
}

impl TmuxClient for FakeTmuxClient {
    fn list_sessions(&self) -> Vec<Session> {
        self.sessions.clone()
    }

    fn current_session(&self) -> Option<String> {
        self.sessions.iter().find(|s| s.attached).map(|s| s.name.clone())
    }

    fn switch_client(&self, _session_name: &str) -> bool {
        true
    }

    fn new_session(&self, _session_name: &str) -> bool {
        true
    }

    fn kill_session(&self, _session_name: &str) -> bool {
        true
    }

    fn rename_session(&self, _old_name: &str, _new_name: &str) -> bool {
        true
    }

    fn capture_pane(&self, _session_name: &str) -> Option<String> {
        Some("fake pane content".to_string())
    }
}
