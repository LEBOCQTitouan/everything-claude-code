use std::path::{Path, PathBuf};

/// The Claude config directory name.
const CLAUDE_DIR_NAME: &str = ".claude";

/// The sessions subdirectory name.
const SESSIONS_DIR_NAME: &str = "sessions";

/// The manifest filename.
const MANIFEST_FILENAME: &str = ".ecc-manifest.json";

/// The settings filename.
const SETTINGS_FILENAME: &str = "settings.json";

/// The aliases filename.
const ALIASES_FILENAME: &str = "session-aliases.json";

/// Get the Claude config directory path from a home directory.
pub fn claude_dir(home: &Path) -> PathBuf {
    home.join(CLAUDE_DIR_NAME)
}

/// Get the sessions directory path from a home directory.
pub fn sessions_dir(home: &Path) -> PathBuf {
    claude_dir(home).join(SESSIONS_DIR_NAME)
}

/// Get the learned skills directory path from a home directory.
pub fn learned_skills_dir(home: &Path) -> PathBuf {
    claude_dir(home).join("skills").join("learned")
}

/// Get the manifest file path from a Claude config directory.
pub fn manifest_path(claude_dir: &Path) -> PathBuf {
    claude_dir.join(MANIFEST_FILENAME)
}

/// Get the settings file path from a Claude config directory.
pub fn settings_path(claude_dir: &Path) -> PathBuf {
    claude_dir.join(SETTINGS_FILENAME)
}

/// Get the aliases file path from a Claude config directory.
pub fn aliases_path(claude_dir: &Path) -> PathBuf {
    claude_dir.join(ALIASES_FILENAME)
}

/// Return the manifest filename constant.
pub fn manifest_filename() -> &'static str {
    MANIFEST_FILENAME
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn claude_dir_appends_dot_claude() {
        let home = Path::new("/home/user");
        assert_eq!(claude_dir(home), PathBuf::from("/home/user/.claude"));
    }

    #[test]
    fn sessions_dir_under_claude() {
        let home = Path::new("/home/user");
        assert_eq!(sessions_dir(home), PathBuf::from("/home/user/.claude/sessions"));
    }

    #[test]
    fn learned_skills_dir_path() {
        let home = Path::new("/home/user");
        assert_eq!(
            learned_skills_dir(home),
            PathBuf::from("/home/user/.claude/skills/learned")
        );
    }

    #[test]
    fn manifest_path_in_claude_dir() {
        let dir = Path::new("/home/user/.claude");
        assert_eq!(
            manifest_path(dir),
            PathBuf::from("/home/user/.claude/.ecc-manifest.json")
        );
    }

    #[test]
    fn settings_path_in_claude_dir() {
        let dir = Path::new("/home/user/.claude");
        assert_eq!(
            settings_path(dir),
            PathBuf::from("/home/user/.claude/settings.json")
        );
    }

    #[test]
    fn aliases_path_in_claude_dir() {
        let dir = Path::new("/home/user/.claude");
        assert_eq!(
            aliases_path(dir),
            PathBuf::from("/home/user/.claude/session-aliases.json")
        );
    }

    #[test]
    fn manifest_filename_constant() {
        assert_eq!(manifest_filename(), ".ecc-manifest.json");
    }

    #[test]
    fn claude_dir_windows_style_home() {
        let home = Path::new("C:/Users/test");
        assert_eq!(claude_dir(home), PathBuf::from("C:/Users/test/.claude"));
    }

    #[test]
    fn claude_dir_root() {
        let home = Path::new("/");
        assert_eq!(claude_dir(home), PathBuf::from("/.claude"));
    }

    #[test]
    fn sessions_dir_windows_style() {
        let home = Path::new("C:/Users/test");
        assert_eq!(
            sessions_dir(home),
            PathBuf::from("C:/Users/test/.claude/sessions")
        );
    }

    #[test]
    fn paths_are_independent_of_each_other() {
        let dir = Path::new("/home/user/.claude");
        let m = manifest_path(dir);
        let s = settings_path(dir);
        let a = aliases_path(dir);
        assert_ne!(m, s);
        assert_ne!(m, a);
        assert_ne!(s, a);
    }

    #[test]
    fn relative_home_dir() {
        let home = Path::new("relative/home");
        assert_eq!(claude_dir(home), PathBuf::from("relative/home/.claude"));
    }

    #[test]
    fn empty_path_home() {
        let home = Path::new("");
        assert_eq!(claude_dir(home), PathBuf::from(".claude"));
    }

    #[test]
    fn learned_skills_three_levels_deep() {
        let home = Path::new("/home/user");
        let path = learned_skills_dir(home);
        // Should be home/.claude/skills/learned
        assert!(path.ends_with("skills/learned"));
        assert!(path.starts_with("/home/user/.claude"));
    }
}
