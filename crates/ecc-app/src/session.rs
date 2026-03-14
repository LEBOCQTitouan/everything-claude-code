//! Session use case — thin wrapper around domain session manager with port injection.

use ecc_domain::session::manager::{
    self, GetAllSessionsOptions, SessionDetail, SessionListResult,
};
use ecc_ports::fs::FileSystem;
use std::path::Path;

/// List sessions in a directory with pagination and filtering.
pub fn list_sessions(
    fs: &dyn FileSystem,
    sessions_dir: &Path,
    options: &GetAllSessionsOptions,
) -> SessionListResult {
    manager::get_all_sessions(fs, sessions_dir, options)
}

/// Get a single session by ID (short ID, date, or full filename).
pub fn get_session(
    fs: &dyn FileSystem,
    sessions_dir: &Path,
    id: &str,
    include_content: bool,
) -> Option<SessionDetail> {
    manager::get_session_by_id(fs, sessions_dir, id, include_content)
}

/// Delete a session file.
pub fn delete_session(fs: &dyn FileSystem, path: &Path) -> bool {
    manager::delete_session(fs, path)
}

/// Write session content to a file.
pub fn write_session(fs: &dyn FileSystem, path: &Path, content: &str) -> bool {
    manager::write_session_content(fs, path, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::InMemoryFileSystem;
    use std::path::Path;

    fn sessions_fs() -> InMemoryFileSystem {
        InMemoryFileSystem::new()
            .with_dir("/sessions")
            .with_file(
                "/sessions/2026-03-14-abc12345-session.tmp",
                "# Session\n## Tasks\n- [x] Task 1\n- [ ] Task 2\n",
            )
            .with_file(
                "/sessions/2026-03-13-def45678-session.tmp",
                "# Session 2\n## Tasks\n- [x] Done\n",
            )
    }

    #[test]
    fn list_sessions_returns_all() {
        let fs = sessions_fs();
        let options = GetAllSessionsOptions::default();
        let result = list_sessions(&fs, Path::new("/sessions"), &options);
        assert_eq!(result.total, 2);
        assert_eq!(result.sessions.len(), 2);
    }

    #[test]
    fn list_sessions_empty_dir() {
        let fs = InMemoryFileSystem::new().with_dir("/sessions");
        let options = GetAllSessionsOptions::default();
        let result = list_sessions(&fs, Path::new("/sessions"), &options);
        assert_eq!(result.total, 0);
    }

    #[test]
    fn get_session_found() {
        let fs = sessions_fs();
        let session = get_session(&fs, Path::new("/sessions"), "abc12345", true);
        assert!(session.is_some());
        let detail = session.unwrap();
        assert_eq!(detail.short_id, "abc12345");
        assert!(detail.content.is_some());
    }

    #[test]
    fn get_session_not_found() {
        let fs = sessions_fs();
        let session = get_session(&fs, Path::new("/sessions"), "nonexistent", false);
        assert!(session.is_none());
    }

    #[test]
    fn get_session_without_content() {
        let fs = sessions_fs();
        let session = get_session(&fs, Path::new("/sessions"), "abc12345", false);
        assert!(session.is_some());
        let detail = session.unwrap();
        assert!(detail.content.is_none());
    }

    #[test]
    fn delete_session_success() {
        let fs = sessions_fs();
        let path = Path::new("/sessions/2026-03-14-abc12345-session.tmp");
        assert!(delete_session(&fs, path));
        assert!(!fs.exists(path));
    }

    #[test]
    fn delete_session_not_found() {
        let fs = InMemoryFileSystem::new();
        assert!(!delete_session(&fs, Path::new("/sessions/nonexistent.tmp")));
    }

    #[test]
    fn write_session_success() {
        let fs = InMemoryFileSystem::new().with_dir("/sessions");
        let path = Path::new("/sessions/test.tmp");
        assert!(write_session(&fs, path, "# New content"));
        assert_eq!(
            fs.read_to_string(path).unwrap(),
            "# New content"
        );
    }
}
