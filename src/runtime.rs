use std::path::PathBuf;

/// Per-user runtime directory for transient rink files.
///
/// Prefer XDG_RUNTIME_DIR on Linux because it is private to the user and cleaned up
/// by the session manager. Fall back to a user-specific directory under /tmp so
/// different users on the same machine do not fight over /tmp/rink permissions.
pub fn runtime_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("RINK_RUNTIME_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir).join("rink");
        }
    }

    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir).join("rink");
        }
    }

    let user = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    std::env::temp_dir().join(format!("rink-{}", sanitize_component(&user)))
}

fn sanitize_component(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_user_component() {
        assert_eq!(sanitize_component("seapy"), "seapy");
        assert_eq!(
            sanitize_component("bad/name with spaces"),
            "bad_name_with_spaces"
        );
        assert_eq!(sanitize_component(""), "unknown");
    }
}
