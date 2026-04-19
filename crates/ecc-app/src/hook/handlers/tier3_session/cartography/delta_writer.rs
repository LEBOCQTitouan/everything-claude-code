//! stop:cartography hook — detects changed files and writes a pending delta.

use std::path::PathBuf;

use ecc_domain::cartography::{ChangedFile, ProjectType, SessionDelta, classify_file};
use tracing::warn;

use crate::hook::{HookPorts, HookResult};

use super::delta_helpers::clean_corrupt_deltas;

/// stop:cartography — detect changed files and write a pending delta.
///
/// - Reads `CLAUDE_PROJECT_DIR` to find the project root.
/// - Runs `git diff --name-only HEAD` to detect changed files.
/// - Detects project type from build files (Cargo.toml → rust, package.json → js/ts).
/// - Writes `.claude/cartography/pending-delta-<session_id>.json`.
/// - Cleans up corrupt existing delta files before writing.
pub fn stop_cartography(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "stop_cartography", "executing handler");

    let project_dir = match ports.env.var("CLAUDE_PROJECT_DIR") {
        Some(d) => PathBuf::from(d),
        None => {
            warn!("stop_cartography: CLAUDE_PROJECT_DIR not set, passthrough");
            return HookResult::passthrough(stdin);
        }
    };

    // Run git diff --name-only HEAD in the project dir
    let git_output =
        match ports
            .shell
            .run_command_in_dir("git", &["diff", "--name-only", "HEAD"], &project_dir)
        {
            Ok(out) => out,
            Err(e) => {
                warn!("stop_cartography: git command error: {}", e);
                return HookResult::passthrough(stdin);
            }
        };

    // Non-zero exit: check for "not a git repository"
    if git_output.exit_code != 0 {
        let combined = format!("{} {}", git_output.stdout, git_output.stderr);
        if combined.to_lowercase().contains("not a git repo") {
            warn!("stop_cartography: project is not a git repository, passthrough");
            return HookResult::passthrough(stdin);
        }
    }

    // No changed files → passthrough, no delta written
    let changed_lines: Vec<&str> = git_output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter(|l| !ecc_domain::cartography::is_noise_path(l))
        .collect();

    if changed_lines.is_empty() {
        return HookResult::passthrough(stdin);
    }

    // Detect project type via detection framework, map to cartography enum
    let detected = crate::detection::framework::detect_project_type(ports.fs, &project_dir);
    let project_type = ProjectType::from(&detected);

    // Get session ID
    let session_id = ports
        .env
        .var("CLAUDE_SESSION_ID")
        .unwrap_or_else(generate_fallback_session_id);

    // Classify changed files
    let changed_files: Vec<ChangedFile> = changed_lines
        .iter()
        .map(|path| {
            let classification = classify_file(path, &project_type);
            ChangedFile {
                path: (*path).to_string(),
                classification,
            }
        })
        .collect();

    let timestamp = super::super::epoch_secs();

    let delta = SessionDelta {
        session_id: session_id.clone(),
        timestamp,
        changed_files,
        project_type,
    };

    let cartography_dir = project_dir.join(".claude").join("cartography");

    // Ensure the cartography directory exists
    if let Err(e) = ports.fs.create_dir_all(&cartography_dir) {
        warn!("stop_cartography: cannot create cartography dir: {}", e);
        return HookResult::passthrough(stdin);
    }

    // Clean up any corrupt delta files
    clean_corrupt_deltas(ports, &cartography_dir);

    // Serialize and write delta
    let delta_json = match serde_json::to_string_pretty(&delta) {
        Ok(j) => j,
        Err(e) => {
            warn!("stop_cartography: failed to serialize delta: {}", e);
            return HookResult::passthrough(stdin);
        }
    };

    let delta_path = cartography_dir.join(format!("pending-delta-{}.json", session_id));
    if let Err(e) = ports.fs.write(&delta_path, &delta_json) {
        warn!("stop_cartography: failed to write delta file: {}", e);
        return HookResult::passthrough(stdin);
    }

    tracing::debug!(
        session_id = %session_id,
        path = %delta_path.display(),
        "stop_cartography: delta written"
    );

    HookResult::passthrough(stdin)
}

/// Generate a fallback session ID from timestamp + process ID.
pub(super) fn generate_fallback_session_id() -> String {
    let ts = super::super::epoch_secs();
    let pid = std::process::id();
    format!("session-{}-{}", ts, pid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use ecc_ports::fs::FileSystem;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};
    /// PC-008: zero committed changes → passthrough, no delta file written.
    #[test]
    fn no_delta_when_no_changes() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new().on_args(
            "git",
            &["diff", "--name-only", "HEAD"],
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/project")
            .with_var("CLAUDE_SESSION_ID", "test-session-001");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = stop_cartography("{}", &ports);

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");

        // No delta file should have been written — either dir doesn't exist or has no pending-delta files
        let cartography_dir = std::path::Path::new("/project/.claude/cartography");
        let no_delta = if fs.exists(cartography_dir) {
            fs.read_dir(cartography_dir)
                .map(|entries| {
                    !entries.iter().any(|p| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .starts_with("pending-delta-")
                    })
                })
                .unwrap_or(true)
        } else {
            true
        };
        assert!(no_delta, "no delta file should have been written");
    }

    /// PC-009: Cargo.toml at root + changed files → delta JSON with project_type="rust"
    /// and crate classification.
    #[test]
    fn writes_delta_rust_project() {
        let fs = InMemoryFileSystem::new().with_file("/project/Cargo.toml", "[workspace]");
        let shell = MockExecutor::new().on_args(
            "git",
            &["diff", "--name-only", "HEAD"],
            CommandOutput {
                stdout: "crates/ecc-domain/src/lib.rs\ncrates/ecc-app/src/main.rs\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/project")
            .with_var("CLAUDE_SESSION_ID", "rust-session-001");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = stop_cartography("{}", &ports);

        assert_eq!(result.exit_code, 0);

        // Delta file should exist
        let delta_path = std::path::Path::new(
            "/project/.claude/cartography/pending-delta-rust-session-001.json",
        );
        assert!(fs.exists(delta_path), "delta file should have been written");

        let content = fs.read_to_string(delta_path).expect("should read delta");
        let delta: SessionDelta = serde_json::from_str(&content).expect("should parse delta JSON");

        assert_eq!(delta.session_id, "rust-session-001");
        assert_eq!(delta.project_type, ProjectType::Rust);
        assert_eq!(delta.changed_files.len(), 2);

        // crates/ecc-domain/src/lib.rs → classification: ecc-domain
        let domain_file = delta
            .changed_files
            .iter()
            .find(|f| f.path == "crates/ecc-domain/src/lib.rs")
            .expect("ecc-domain file should be present");
        assert_eq!(domain_file.classification, "ecc-domain");

        // crates/ecc-app/src/main.rs → classification: ecc-app
        let app_file = delta
            .changed_files
            .iter()
            .find(|f| f.path == "crates/ecc-app/src/main.rs")
            .expect("ecc-app file should be present");
        assert_eq!(app_file.classification, "ecc-app");
    }

    /// PC-010: project-type variants (package.json→typescript/javascript; no build file→unknown)
    /// + CLAUDE_SESSION_ID absent → fallback ID.
    #[test]
    fn project_type_variants_and_fallback_id() {
        // --- typescript (package.json + tsconfig.json) ---
        let fs_ts = InMemoryFileSystem::new()
            .with_file("/tsproject/package.json", "{}")
            .with_file("/tsproject/tsconfig.json", "{}");
        let shell_ts = MockExecutor::new().on_args(
            "git",
            &["diff", "--name-only", "HEAD"],
            CommandOutput {
                stdout: "src/index.ts\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env_ts = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/tsproject")
            .with_var("CLAUDE_SESSION_ID", "ts-session-001");
        let term_ts = BufferedTerminal::new();
        let ports_ts = HookPorts::test_default(&fs_ts, &shell_ts, &env_ts, &term_ts);

        let _ = stop_cartography("{}", &ports_ts);
        let delta_ts_path = std::path::Path::new(
            "/tsproject/.claude/cartography/pending-delta-ts-session-001.json",
        );
        let content_ts = fs_ts.read_to_string(delta_ts_path).expect("ts delta");
        let delta_ts: SessionDelta = serde_json::from_str(&content_ts).expect("ts delta json");
        assert_eq!(delta_ts.project_type, ProjectType::Typescript);

        // --- javascript (package.json, no tsconfig) ---
        let fs_js = InMemoryFileSystem::new().with_file("/jsproject/package.json", "{}");
        let shell_js = MockExecutor::new().on_args(
            "git",
            &["diff", "--name-only", "HEAD"],
            CommandOutput {
                stdout: "src/index.js\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env_js = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/jsproject")
            .with_var("CLAUDE_SESSION_ID", "js-session-001");
        let term_js = BufferedTerminal::new();
        let ports_js = HookPorts::test_default(&fs_js, &shell_js, &env_js, &term_js);

        let _ = stop_cartography("{}", &ports_js);
        let delta_js_path = std::path::Path::new(
            "/jsproject/.claude/cartography/pending-delta-js-session-001.json",
        );
        let content_js = fs_js.read_to_string(delta_js_path).expect("js delta");
        let delta_js: SessionDelta = serde_json::from_str(&content_js).expect("js delta json");
        assert_eq!(delta_js.project_type, ProjectType::Javascript);

        // --- unknown (no recognized build file) + top-level directory classification ---
        let fs_unk = InMemoryFileSystem::new();
        let shell_unk = MockExecutor::new().on_args(
            "git",
            &["diff", "--name-only", "HEAD"],
            CommandOutput {
                stdout: "src/main.rb\ndocs/guide.md\n".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env_unk = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/unknown-project");
        // CLAUDE_SESSION_ID NOT set → fallback ID
        let term_unk = BufferedTerminal::new();
        let ports_unk = HookPorts::test_default(&fs_unk, &shell_unk, &env_unk, &term_unk);

        let _ = stop_cartography("{}", &ports_unk);

        // Find whatever pending-delta file was written (fallback ID)
        let cart_dir = std::path::Path::new("/unknown-project/.claude/cartography");
        let entries = fs_unk
            .read_dir(cart_dir)
            .expect("cartography dir should exist");
        let delta_files: Vec<_> = entries
            .iter()
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .starts_with("pending-delta-")
            })
            .collect();
        assert_eq!(delta_files.len(), 1, "exactly one delta file should exist");

        let content_unk = fs_unk
            .read_to_string(delta_files[0])
            .expect("unknown delta");
        let delta_unk: SessionDelta =
            serde_json::from_str(&content_unk).expect("unknown delta json");
        assert_eq!(delta_unk.project_type, ProjectType::Unknown);

        // Fallback ID format: session-<timestamp>-<pid>
        assert!(
            delta_unk.session_id.starts_with("session-"),
            "fallback session ID should start with 'session-', got: {}",
            delta_unk.session_id
        );

        // Files classified by top-level directory
        let main_rb = delta_unk
            .changed_files
            .iter()
            .find(|f| f.path == "src/main.rb")
            .expect("src/main.rb should be present");
        assert_eq!(main_rb.classification, "src");

        let guide = delta_unk
            .changed_files
            .iter()
            .find(|f| f.path == "docs/guide.md")
            .expect("docs/guide.md should be present");
        assert_eq!(guide.classification, "docs");
    }

    /// PC-011: no git repo → passthrough + warning; corrupt JSON → deleted + warning,
    /// current delta written.
    #[test]
    fn edge_cases_no_git_and_corrupt_delta() {
        // --- no git repo: git diff returns non-zero with "not a git repository" ---
        {
            let fs = InMemoryFileSystem::new().with_file("/norepo/Cargo.toml", "[workspace]");
            let shell = MockExecutor::new().on_args(
                "git",
                &["diff", "--name-only", "HEAD"],
                CommandOutput {
                    stdout: String::new(),
                    stderr: "fatal: not a git repository (or any of the parent directories): .git"
                        .to_string(),
                    exit_code: 128,
                },
            );
            let env = MockEnvironment::new()
                .with_var("CLAUDE_PROJECT_DIR", "/norepo")
                .with_var("CLAUDE_SESSION_ID", "norepo-session");
            let term = BufferedTerminal::new();
            let ports = HookPorts::test_default(&fs, &shell, &env, &term);

            let result = stop_cartography("{}", &ports);

            assert_eq!(result.exit_code, 0);
            assert_eq!(result.stdout, "{}");

            // No delta file should have been written
            let cart_dir = std::path::Path::new("/norepo/.claude/cartography");
            if fs.exists(cart_dir) {
                let entries = fs.read_dir(cart_dir).unwrap_or_default();
                let delta_files: Vec<_> = entries
                    .iter()
                    .filter(|p| {
                        p.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .starts_with("pending-delta-")
                    })
                    .collect();
                assert!(
                    delta_files.is_empty(),
                    "no delta should be written for non-git repo"
                );
            }
        }

        // --- corrupt JSON: existing delta file with invalid JSON is deleted, current one written ---
        {
            let fs = InMemoryFileSystem::new()
                .with_file("/project/Cargo.toml", "[workspace]")
                .with_file(
                    "/project/.claude/cartography/pending-delta-old-session.json",
                    "{not valid json",
                );
            let shell = MockExecutor::new().on_args(
                "git",
                &["diff", "--name-only", "HEAD"],
                CommandOutput {
                    stdout: "crates/ecc-app/src/lib.rs\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
            let env = MockEnvironment::new()
                .with_var("CLAUDE_PROJECT_DIR", "/project")
                .with_var("CLAUDE_SESSION_ID", "new-session-001");
            let term = BufferedTerminal::new();
            let ports = HookPorts::test_default(&fs, &shell, &env, &term);

            let result = stop_cartography("{}", &ports);
            assert_eq!(result.exit_code, 0);

            // Corrupt file should have been deleted
            let corrupt_path =
                std::path::Path::new("/project/.claude/cartography/pending-delta-old-session.json");
            assert!(
                !fs.exists(corrupt_path),
                "corrupt delta file should have been deleted"
            );

            // Current session's delta should have been written
            let new_delta_path = std::path::Path::new(
                "/project/.claude/cartography/pending-delta-new-session-001.json",
            );
            assert!(
                fs.exists(new_delta_path),
                "current session delta should have been written"
            );

            let content = fs.read_to_string(new_delta_path).expect("new delta");
            let delta: SessionDelta = serde_json::from_str(&content).expect("new delta json");
            assert_eq!(delta.session_id, "new-session-001");
            assert_eq!(delta.project_type, ProjectType::Rust);
        }
    }

    /// PC-010: workflow-only session (`.claude/workflow/` files only) → passthrough, no delta written.
    #[test]
    fn filters_workflow_only_session() {
        let fs = InMemoryFileSystem::new().with_file("/project/Cargo.toml", "[workspace]");
        let shell = MockExecutor::new().on_args(
            "git",
            &["diff", "--name-only", "HEAD"],
            CommandOutput {
                stdout: ".claude/workflow/state.json\n.claude/workflow/implement-done.md\n"
                    .to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/project")
            .with_var("CLAUDE_SESSION_ID", "workflow-only-001");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = stop_cartography("{}", &ports);

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");

        // No delta should be written — all changed files are workflow noise
        let delta_path = std::path::Path::new(
            "/project/.claude/cartography/pending-delta-workflow-only-001.json",
        );
        assert!(
            !fs.exists(delta_path),
            "workflow-only paths must be filtered — no delta should be written"
        );
    }

    /// PC-011: spec-only session (`docs/specs/<slug>/spec.md`, `design.md`, `tasks.md`) →
    /// passthrough, no delta written.
    #[test]
    fn filters_spec_only_session() {
        let fs = InMemoryFileSystem::new().with_file("/project/Cargo.toml", "[workspace]");
        let shell = MockExecutor::new().on_args(
            "git",
            &["diff", "--name-only", "HEAD"],
            CommandOutput {
                stdout: "docs/specs/2026-04-19-foo/spec.md\ndocs/specs/2026-04-19-foo/design.md\ndocs/specs/2026-04-19-foo/tasks.md\n"
                    .to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/project")
            .with_var("CLAUDE_SESSION_ID", "spec-only-001");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = stop_cartography("{}", &ports);

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");

        // No delta should be written — all changed files are spec noise
        let delta_path = std::path::Path::new(
            "/project/.claude/cartography/pending-delta-spec-only-001.json",
        );
        assert!(
            !fs.exists(delta_path),
            "spec-only paths must be filtered — no delta should be written"
        );
    }

    /// Only `.claude/` paths in git diff → passthrough, no delta written (self-referential filter).
    #[test]
    fn filters_out_dot_claude_paths() {
        let fs = InMemoryFileSystem::new().with_file("/project/Cargo.toml", "[workspace]");
        let shell = MockExecutor::new().on_args(
            "git",
            &["diff", "--name-only", "HEAD"],
            CommandOutput {
                stdout: ".claude/cartography/pending-delta-old.json\n.claude/workflow/state.json\n"
                    .to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
        let env = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/project")
            .with_var("CLAUDE_SESSION_ID", "self-ref-001");
        let term = BufferedTerminal::new();
        let ports = HookPorts::test_default(&fs, &shell, &env, &term);

        let result = stop_cartography("{}", &ports);

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");

        // No delta should be written — all changed files were filtered out
        let delta_path =
            std::path::Path::new("/project/.claude/cartography/pending-delta-self-ref-001.json");
        assert!(
            !fs.exists(delta_path),
            ".claude/ paths must be filtered — no delta should be written"
        );
    }
}
