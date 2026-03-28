use super::error::MergeError;
use ecc_ports::fs::FileSystem;
use std::path::Path;

pub(super) fn read_json(fs: &dyn FileSystem, path: &Path) -> Result<serde_json::Value, MergeError> {
    let content = fs.read_to_string(path).map_err(|e| MergeError::ReadFile {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;
    serde_json::from_str(&content).map_err(|e| MergeError::InvalidJson {
        path: path.display().to_string(),
        reason: e.to_string(),
    })
}

pub(super) fn read_json_or_default(fs: &dyn FileSystem, path: &Path) -> serde_json::Value {
    read_json(fs, path).unwrap_or_else(|_| serde_json::json!({}))
}

pub(super) fn copy_dir_recursive(
    fs: &dyn FileSystem,
    src: &Path,
    dest: &Path,
) -> Result<(), MergeError> {
    fs.create_dir_all(dest).map_err(|e| MergeError::CreateDir {
        path: dest.display().to_string(),
        reason: e.to_string(),
    })?;

    let entries = fs
        .read_dir_recursive(src)
        .map_err(|e| MergeError::ReadDir {
            path: src.display().to_string(),
            reason: e.to_string(),
        })?;

    for entry in entries {
        if let Ok(relative) = entry.strip_prefix(src) {
            let dest_path = dest.join(relative);
            if fs.is_dir(&entry) {
                fs.create_dir_all(&dest_path)
                    .map_err(|e| MergeError::CreateDir {
                        path: dest_path.display().to_string(),
                        reason: e.to_string(),
                    })?;
            } else {
                if let Some(parent) = dest_path.parent() {
                    fs.create_dir_all(parent)
                        .map_err(|e| MergeError::CreateDir {
                            path: parent.display().to_string(),
                            reason: e.to_string(),
                        })?;
                }
                fs.copy(&entry, &dest_path)
                    .map_err(|e| MergeError::CopyFile {
                        path: entry.display().to_string(),
                        reason: e.to_string(),
                    })?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use ecc_domain::config::merge::{self, FileToReview};
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};
    use std::path::Path;

    fn no_color_env() -> MockEnvironment {
        MockEnvironment::new().with_var("NO_COLOR", "1")
    }

    // --- prompt_file_review ---

    #[test]
    fn prompt_accept() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new content")
            .with_file("/dest/agent.md", "old content");
        let terminal = BufferedTerminal::new().with_input("a");
        let env = no_color_env();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };

        let (choice, apply_all) = prompt_file_review(&terminal, &fs, &env, &file, "[1/2]").unwrap();
        assert_eq!(choice, ReviewChoice::Accept);
        assert!(!apply_all);
    }

    #[test]
    fn prompt_keep() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new")
            .with_file("/dest/agent.md", "old");
        let terminal = BufferedTerminal::new().with_input("k");
        let env = no_color_env();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };

        let (choice, _) = prompt_file_review(&terminal, &fs, &env, &file, "[1/1]").unwrap();
        assert_eq!(choice, ReviewChoice::Keep);
    }

    #[test]
    fn prompt_smart_merge() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new content")
            .with_file("/dest/agent.md", "old content");
        let terminal = BufferedTerminal::new().with_input("s");
        let env = no_color_env();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };

        let (choice, apply_all) = prompt_file_review(&terminal, &fs, &env, &file, "[1/1]").unwrap();
        assert_eq!(choice, ReviewChoice::SmartMerge);
        assert!(!apply_all);
    }

    #[test]
    fn prompt_accept_all() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new")
            .with_file("/dest/agent.md", "old");
        let terminal = BufferedTerminal::new().with_input("A");
        let env = no_color_env();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };

        let (choice, apply_all) = prompt_file_review(&terminal, &fs, &env, &file, "[1/2]").unwrap();
        assert_eq!(choice, ReviewChoice::Accept);
        assert!(apply_all);
    }

    #[test]
    fn prompt_keep_all() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new")
            .with_file("/dest/agent.md", "old");
        let terminal = BufferedTerminal::new().with_input("K");
        let env = no_color_env();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };

        let (choice, apply_all) = prompt_file_review(&terminal, &fs, &env, &file, "[1/2]").unwrap();
        assert_eq!(choice, ReviewChoice::Keep);
        assert!(apply_all);
    }

    #[test]
    fn prompt_new_file_shows_preview() {
        let fs = InMemoryFileSystem::new().with_file("/src/agent.md", "line1\nline2\nline3");
        let terminal = BufferedTerminal::new().with_input("a");
        let env = no_color_env();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: true,
        };

        let result = prompt_file_review(&terminal, &fs, &env, &file, "[1/1]");
        assert!(result.is_ok());
        let output = terminal.stdout_output();
        assert!(output.iter().any(|s| s.contains("NEW")));
    }

    #[test]
    fn prompt_unknown_input_defaults_to_accept() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new")
            .with_file("/dest/agent.md", "old");
        let terminal = BufferedTerminal::new().with_input("z");
        let env = no_color_env();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };

        let (choice, _) = prompt_file_review(&terminal, &fs, &env, &file, "[1/1]").unwrap();
        assert_eq!(choice, ReviewChoice::Accept);
    }

    // --- apply_review_choice ---

    #[test]
    fn apply_accept_copies_file() {
        let fs = InMemoryFileSystem::new().with_file("/src/agent.md", "new content");
        let shell = MockExecutor::new();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: true,
        };
        let mut report = merge::empty_report();

        apply_review_choice(&fs, &shell, ReviewChoice::Accept, &file, false, &mut report);

        assert!(report.errors.is_empty());
        assert!(report.added.contains(&"agent.md".to_string()));
    }

    #[test]
    fn apply_keep_skips_file() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };
        let mut report = merge::empty_report();

        apply_review_choice(&fs, &shell, ReviewChoice::Keep, &file, false, &mut report);

        assert!(report.skipped.contains(&"agent.md".to_string()));
        assert!(report.updated.is_empty());
    }

    #[test]
    fn apply_smart_merge_success() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new content")
            .with_file("/dest/agent.md", "old content");
        let shell = MockExecutor::new().with_command("claude").on(
            "claude",
            CommandOutput {
                stdout: "merged content".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: false,
        };
        let mut report = merge::empty_report();

        apply_review_choice(
            &fs,
            &shell,
            ReviewChoice::SmartMerge,
            &file,
            false,
            &mut report,
        );

        assert!(report.errors.is_empty(), "errors: {:?}", report.errors);
        assert!(report.smart_merged.contains(&"agent.md".to_string()));
    }

    #[test]
    fn apply_accept_dry_run_does_not_copy() {
        let fs = InMemoryFileSystem::new().with_file("/src/agent.md", "new content");
        let shell = MockExecutor::new();
        let file = FileToReview {
            filename: "agent.md".into(),
            src_path: "/src/agent.md".into(),
            dest_path: "/dest/agent.md".into(),
            is_new: true,
        };
        let mut report = merge::empty_report();

        apply_review_choice(&fs, &shell, ReviewChoice::Accept, &file, true, &mut report);

        assert!(report.errors.is_empty());
        // Dry run: file should NOT be copied
        assert!(!fs.exists(Path::new("/dest/agent.md")));
    }

    // --- merge_directory ---

    #[test]
    fn merge_directory_with_force_skips_prompt() {
        let fs = InMemoryFileSystem::new()
            .with_file("/src/agent.md", "new content")
            .with_file("/dest/agent.md", "old content");
        let terminal = BufferedTerminal::new();
        let env = no_color_env();
        let shell = MockExecutor::new();
        let ctx = MergeContext {
            fs: &fs,
            terminal: &terminal,
            env: &env,
            shell: &shell,
        };
        let mut options = MergeOptions {
            force: true,
            ..Default::default()
        };

        let report = merge_directory(
            &ctx,
            Path::new("/src"),
            Path::new("/dest"),
            "Agents",
            ".md",
            &mut options,
        );
        assert!(report.errors.is_empty());
        assert!(!report.updated.is_empty() || !report.added.is_empty());
    }

    // --- merge_hooks ---

    #[test]
    fn merge_hooks_adds_new_hooks() {
        let hooks_json = r#"{
            "hooks": {
                "PreToolUse": [
                    {"description": "ECC format", "hooks": [{"command": "ecc-hook format", "type": "command"}]}
                ]
            }
        }"#;
        let settings_json = r#"{}"#;

        let fs = InMemoryFileSystem::new()
            .with_file("/ecc/hooks.json", hooks_json)
            .with_file("/claude/settings.json", settings_json);

        let result = merge_hooks(
            &fs,
            Path::new("/ecc/hooks.json"),
            Path::new("/claude/settings.json"),
            false,
        );

        assert!(result.is_ok(), "err: {:?}", result);
        let (added, existing, _legacy) = result.unwrap();
        assert_eq!(added, 1);
        assert_eq!(existing, 0);
    }

    #[test]
    fn merge_hooks_returns_error_for_missing_hooks_json() {
        let fs = InMemoryFileSystem::new();
        let result = merge_hooks(
            &fs,
            Path::new("/missing/hooks.json"),
            Path::new("/claude/settings.json"),
            false,
        );
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("merge_hooks"), "got: {msg}");
    }

    #[test]
    fn merge_hooks_dry_run_no_write() {
        let hooks_json = r#"{
            "hooks": {
                "PreToolUse": [
                    {"description": "ECC format", "hooks": [{"command": "ecc-hook format", "type": "command"}]}
                ]
            }
        }"#;
        let fs = InMemoryFileSystem::new().with_file("/ecc/hooks.json", hooks_json);

        let result = merge_hooks(
            &fs,
            Path::new("/ecc/hooks.json"),
            Path::new("/claude/settings.json"),
            true, // dry_run
        );

        assert!(result.is_ok());
        // settings.json should NOT be created
        assert!(!fs.exists(std::path::Path::new("/claude/settings.json")));
    }
}
