//! Session storage — load, save, list, branch, clear sessions via FileSystem port.

use super::error::ClawError;
use ecc_domain::claw::session_name::{is_valid_session_name, session_name_from_path};
use ecc_domain::claw::turn::{Turn, format_turns, parse_turns};
use ecc_domain::paths::{claw_dir, claw_session_path};
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// Validate session name at the storage boundary.
fn validate_name(session_name: &str) -> Result<(), ClawError> {
    if !is_valid_session_name(session_name) {
        return Err(ClawError::InvalidSessionName {
            name: session_name.to_string(),
        });
    }
    Ok(())
}

/// Load turns from a session file. Returns empty Vec if not found.
pub fn load_session(home: &Path, session_name: &str, fs: &dyn FileSystem) -> Vec<Turn> {
    if validate_name(session_name).is_err() {
        return Vec::new();
    }
    let path = claw_session_path(home, session_name);
    match fs.read_to_string(&path) {
        Ok(content) => parse_turns(&content),
        Err(_) => Vec::new(),
    }
}

/// Save turns to a session file.
pub fn save_session(
    home: &Path,
    session_name: &str,
    turns: &[Turn],
    fs: &dyn FileSystem,
) -> Result<(), ClawError> {
    validate_name(session_name)?;
    let path = claw_session_path(home, session_name);
    let sessions_dir = claw_dir(home).join("sessions");
    fs.create_dir_all(&sessions_dir).map_err(|e| ClawError::CreateSessionDir {
        reason: e.to_string(),
    })?;
    let content = format_turns(turns);
    fs.write(&path, &content).map_err(|e| ClawError::SaveSession {
        name: session_name.to_string(),
        reason: e.to_string(),
    })
}

/// List all session names.
pub fn list_sessions(home: &Path, fs: &dyn FileSystem) -> Vec<String> {
    let sessions_dir = claw_dir(home).join("sessions");
    match fs.read_dir(&sessions_dir) {
        Ok(entries) => entries
            .iter()
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .is_some_and(|e| e == "md")
            })
            .filter_map(|p| session_name_from_path(p))
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Branch: copy current session into a new session name.
pub fn branch_session(
    home: &Path,
    source_name: &str,
    target_name: &str,
    turns: &[Turn],
    fs: &dyn FileSystem,
) -> Result<(), ClawError> {
    validate_name(target_name)?;
    save_session(home, target_name, turns, fs).map_err(|e| ClawError::BranchSession {
        source_name: source_name.to_string(),
        target_name: target_name.to_string(),
        reason: e.to_string(),
    })
}

/// Clear a session by removing its file.
pub fn clear_session(home: &Path, session_name: &str, fs: &dyn FileSystem) -> Result<(), ClawError> {
    validate_name(session_name)?;
    let path = claw_session_path(home, session_name);
    if fs.exists(&path) {
        fs.remove_file(&path).map_err(|e| ClawError::ClearSession {
            name: session_name.to_string(),
            reason: e.to_string(),
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::Path;

    fn home() -> &'static Path {
        Path::new("/home/test")
    }

    #[test]
    fn load_session_returns_empty_when_missing() {
        let fs = InMemoryFileSystem::new();
        let turns = load_session(home(), "nonexistent", &fs);
        assert!(turns.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let fs = InMemoryFileSystem::new();
        let turns = vec![ecc_domain::claw::turn::Turn {
            timestamp: "ts".to_string(),
            role: ecc_domain::claw::turn::Role::User,
            content: "hello".to_string(),
        }];

        save_session(home(), "test", &turns, &fs).unwrap();
        let loaded = load_session(home(), "test", &fs);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].content, "hello");
    }

    #[test]
    fn save_session_creates_dir() {
        let fs = InMemoryFileSystem::new();
        let turns = vec![ecc_domain::claw::turn::Turn {
            timestamp: "ts".to_string(),
            role: ecc_domain::claw::turn::Role::User,
            content: "test".to_string(),
        }];

        let result = save_session(home(), "test", &turns, &fs);
        assert!(result.is_ok());
    }

    #[test]
    fn list_sessions_empty() {
        let fs = InMemoryFileSystem::new();
        let names = list_sessions(home(), &fs);
        assert!(names.is_empty());
    }

    #[test]
    fn list_sessions_finds_md_files() {
        let fs = InMemoryFileSystem::new()
            .with_file("/home/test/.claude/claw/sessions/alpha.md", "content")
            .with_file("/home/test/.claude/claw/sessions/beta.md", "content");
        let names = list_sessions(home(), &fs);
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"alpha".to_string()));
        assert!(names.contains(&"beta".to_string()));
    }

    #[test]
    fn list_sessions_ignores_non_md() {
        let fs = InMemoryFileSystem::new()
            .with_file("/home/test/.claude/claw/sessions/alpha.md", "content")
            .with_file("/home/test/.claude/claw/sessions/readme.txt", "content");
        let names = list_sessions(home(), &fs);
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "alpha");
    }

    #[test]
    fn branch_session_creates_copy() {
        let fs = InMemoryFileSystem::new();
        let turns = vec![ecc_domain::claw::turn::Turn {
            timestamp: "ts".to_string(),
            role: ecc_domain::claw::turn::Role::User,
            content: "original".to_string(),
        }];

        branch_session(home(), "main", "branch1", &turns, &fs).unwrap();
        let loaded = load_session(home(), "branch1", &fs);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].content, "original");
    }

    #[test]
    fn clear_session_removes_file() {
        let fs = InMemoryFileSystem::new()
            .with_file("/home/test/.claude/claw/sessions/test.md", "content");

        assert!(fs.exists(Path::new("/home/test/.claude/claw/sessions/test.md")));
        clear_session(home(), "test", &fs).unwrap();
        assert!(!fs.exists(Path::new("/home/test/.claude/claw/sessions/test.md")));
    }

    #[test]
    fn clear_session_nonexistent_is_ok() {
        let fs = InMemoryFileSystem::new();
        let result = clear_session(home(), "nonexistent", &fs);
        assert!(result.is_ok());
    }

    #[test]
    fn save_session_overwrites() {
        let fs = InMemoryFileSystem::new();
        let turns1 = vec![ecc_domain::claw::turn::Turn {
            timestamp: "ts".to_string(),
            role: ecc_domain::claw::turn::Role::User,
            content: "first".to_string(),
        }];
        let turns2 = vec![ecc_domain::claw::turn::Turn {
            timestamp: "ts".to_string(),
            role: ecc_domain::claw::turn::Role::User,
            content: "second".to_string(),
        }];

        save_session(home(), "test", &turns1, &fs).unwrap();
        save_session(home(), "test", &turns2, &fs).unwrap();
        let loaded = load_session(home(), "test", &fs);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].content, "second");
    }

    #[test]
    fn load_session_with_multiple_turns() {
        let content = "### [ts1] User\nhello\n---\n\n### [ts2] Assistant\nworld\n---";
        let fs = InMemoryFileSystem::new()
            .with_file("/home/test/.claude/claw/sessions/multi.md", content);
        let turns = load_session(home(), "multi", &fs);
        assert_eq!(turns.len(), 2);
    }

    #[test]
    fn save_rejects_invalid_name() {
        let fs = InMemoryFileSystem::new();
        let result = save_session(home(), "../etc/passwd", &[], &fs);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("invalid session name") || err_msg.contains("Invalid session name"), "got: {err_msg}");
    }

    #[test]
    fn load_returns_empty_for_invalid_name() {
        let fs = InMemoryFileSystem::new();
        let turns = load_session(home(), "../etc/passwd", &fs);
        assert!(turns.is_empty());
    }

    #[test]
    fn clear_rejects_invalid_name() {
        let fs = InMemoryFileSystem::new();
        let result = clear_session(home(), "bad/name", &fs);
        assert!(result.is_err());
    }

    #[test]
    fn branch_rejects_invalid_target_name() {
        let fs = InMemoryFileSystem::new();
        let result = branch_session(home(), "src", "bad name!", &[], &fs);
        assert!(result.is_err());
    }

    #[test]
    fn save_empty_turns() {
        let fs = InMemoryFileSystem::new();
        let result = save_session(home(), "empty", &[], &fs);
        assert!(result.is_ok());
    }

    #[test]
    fn branch_preserves_all_turns() {
        let turns = vec![
            ecc_domain::claw::turn::Turn {
                timestamp: "ts1".to_string(),
                role: ecc_domain::claw::turn::Role::User,
                content: "a".to_string(),
            },
            ecc_domain::claw::turn::Turn {
                timestamp: "ts2".to_string(),
                role: ecc_domain::claw::turn::Role::Assistant,
                content: "b".to_string(),
            },
        ];
        let fs = InMemoryFileSystem::new();
        branch_session(home(), "src", "dst", &turns, &fs).unwrap();
        let loaded = load_session(home(), "dst", &fs);
        assert_eq!(loaded.len(), 2);
    }
}
