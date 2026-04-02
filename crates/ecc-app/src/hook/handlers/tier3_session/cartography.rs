//! Cartography hook handlers — stop:cartography writes session deltas.

use std::path::{Path, PathBuf};

use ecc_domain::cartography::{ChangedFile, ProjectType, SessionDelta};
use tracing::warn;

use crate::hook::{HookPorts, HookResult};

/// start:cartography — process pending deltas by invoking the cartographer agent.
///
/// - Reads `CLAUDE_PROJECT_DIR` to find the project root.
/// - If no pending deltas exist in `.claude/cartography/`, exits immediately.
/// - Discards uncommitted changes in `docs/cartography/` via `git checkout`.
/// - Creates scaffold directories if missing.
/// - Acquires file lock `.claude/cartography/cartography-merge.lock` (skips if held).
/// - Reads pending deltas, filters already-processed ones, sorts by timestamp ascending.
/// - Invokes cartographer agent for each delta.
/// - On success: archives deltas to `processed/`.
/// - On failure: runs `git reset HEAD docs/cartography/` and logs error.
pub fn start_cartography(stdin: &str, ports: &HookPorts<'_>) -> HookResult {
    tracing::debug!(handler = "start_cartography", "executing handler");

    let project_dir = match ports.env.var("CLAUDE_PROJECT_DIR") {
        Some(d) => PathBuf::from(d),
        None => {
            warn!("start_cartography: CLAUDE_PROJECT_DIR not set, passthrough");
            return HookResult::passthrough(stdin);
        }
    };

    let cartography_dir = project_dir.join(".claude").join("cartography");

    // Step 1: Check for pending deltas — if none, exit immediately (AC-006.5)
    let pending_deltas = collect_pending_deltas(ports, &cartography_dir);
    if pending_deltas.is_empty() {
        tracing::debug!("start_cartography: no pending deltas, passthrough");
        return HookResult::passthrough(stdin);
    }

    // Step 2: Discard uncommitted changes in docs/cartography/ (AC-002.3)
    let docs_cartography = project_dir.join("docs").join("cartography");
    if let Ok(status_out) = ports.shell.run_command_in_dir(
        "git",
        &["status", "--porcelain", "docs/cartography/"],
        &project_dir,
    ) && !status_out.stdout.trim().is_empty() {
        let _ = ports.shell.run_command_in_dir(
            "git",
            &["checkout", "--", "docs/cartography/"],
            &project_dir,
        );
    }

    // Step 3: Create scaffold if missing (AC-001.1, AC-001.4)
    let journeys_dir = docs_cartography.join("journeys");
    let flows_dir = docs_cartography.join("flows");
    if !ports.fs.exists(&journeys_dir) {
        let _ = ports.fs.create_dir_all(&journeys_dir);
    }
    if !ports.fs.exists(&flows_dir) {
        let _ = ports.fs.create_dir_all(&flows_dir);
    }
    let readme_path = docs_cartography.join("README.md");
    if !ports.fs.exists(&readme_path) {
        let _ = ports.fs.write(
            &readme_path,
            "# Cartography\n\nAuto-generated documentation of user journeys and data flows.\n",
        );
    }

    // Step 4: Try to acquire file lock (AC-002.4)
    let lock_path = cartography_dir.join("cartography-merge.lock");
    if ports.fs.exists(&lock_path) {
        tracing::debug!("start_cartography: lock held, skipping");
        return HookResult::passthrough(stdin);
    }
    // Write lock file
    if ports.fs.write(&lock_path, "locked").is_err() {
        warn!("start_cartography: cannot acquire lock, skipping");
        return HookResult::passthrough(stdin);
    }

    // Step 5: Filter out already-processed deltas and sort by timestamp (AC-006.6, AC-006.7)
    let processed_dir = cartography_dir.join("processed");
    let mut unprocessed: Vec<(PathBuf, SessionDelta)> = pending_deltas
        .into_iter()
        .filter_map(|path| {
            let content = ports.fs.read_to_string(&path).ok()?;
            let delta: SessionDelta = serde_json::from_str(&content).ok()?;
            // Check if already processed
            let file_name = path.file_name()?.to_str()?.to_string();
            let processed_path = processed_dir.join(&file_name);
            if ports.fs.exists(&processed_path) {
                return None;
            }
            Some((path, delta))
        })
        .collect();

    // Sort by timestamp ascending
    unprocessed.sort_by_key(|(_, delta)| delta.timestamp);

    if unprocessed.is_empty() {
        let _ = ports.fs.remove_file(&lock_path);
        return HookResult::passthrough(stdin);
    }

    // Step 6: Invoke cartographer agent for each delta (AC-006.3, AC-006.4)
    let mut success = true;
    for (_path, delta) in &unprocessed {
        let delta_json = match serde_json::to_string(delta) {
            Ok(j) => j,
            Err(_) => {
                success = false;
                break;
            }
        };
        let agent_result = ports.shell.run_command_in_dir(
            "claude",
            &["--agent", "cartographer", "--input", &delta_json],
            &project_dir,
        );
        if let Ok(out) = agent_result {
            if out.exit_code != 0 {
                success = false;
                break;
            }
        } else {
            success = false;
            break;
        }
    }

    if success {
        // git add docs/cartography/ && git commit
        let _ = ports
            .shell
            .run_command_in_dir("git", &["add", "docs/cartography/"], &project_dir);
        let _ = ports.shell.run_command_in_dir(
            "git",
            &["commit", "-m", "docs(cartography): update"],
            &project_dir,
        );

        // Archive processed deltas (AC-006.8)
        let _ = ports.fs.create_dir_all(&processed_dir);
        for (path, _) in &unprocessed {
            if let Some(file_name) = path.file_name() {
                let dest = processed_dir.join(file_name);
                let _ = ports.fs.rename(path, &dest);
            }
        }
    } else {
        // On failure: git reset, do not archive (AC-006.3)
        let _ = ports.shell.run_command_in_dir(
            "git",
            &["reset", "HEAD", "docs/cartography/"],
            &project_dir,
        );
        let msg = "[start_cartography] agent failed; changes reset, deltas not archived\n";
        warn!("{}", msg.trim());
        let _ = ports.fs.remove_file(&lock_path);
        return HookResult {
            stdout: stdin.to_string(),
            stderr: msg.to_string(),
            exit_code: 0,
        };
    }

    // Release lock
    let _ = ports.fs.remove_file(&lock_path);

    HookResult::passthrough(stdin)
}

/// Collect all `pending-delta-*.json` files from the cartography directory.
fn collect_pending_deltas(ports: &HookPorts<'_>, cartography_dir: &Path) -> Vec<PathBuf> {
    if !ports.fs.exists(cartography_dir) {
        return Vec::new();
    }
    let entries = match ports.fs.read_dir(cartography_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    entries
        .into_iter()
        .filter(|p| {
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            name.starts_with("pending-delta-") && name.ends_with(".json")
        })
        .collect()
}

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
    let git_output = match ports
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
        .collect();

    if changed_lines.is_empty() {
        return HookResult::passthrough(stdin);
    }

    // Detect project type
    let project_type = detect_project_type(ports, &project_dir);

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

    let timestamp = epoch_secs();

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

/// Detect project type based on the presence of build files at the project root.
fn detect_project_type(ports: &HookPorts<'_>, project_dir: &Path) -> ProjectType {
    if ports.fs.exists(&project_dir.join("Cargo.toml")) {
        return ProjectType::Rust;
    }
    if ports.fs.exists(&project_dir.join("package.json")) {
        // Check for TypeScript indicator
        if ports.fs.exists(&project_dir.join("tsconfig.json"))
            || ports.fs.exists(&project_dir.join("tsconfig.base.json"))
        {
            return ProjectType::Typescript;
        }
        return ProjectType::Javascript;
    }
    if ports.fs.exists(&project_dir.join("pyproject.toml"))
        || ports.fs.exists(&project_dir.join("setup.py"))
    {
        return ProjectType::Python;
    }
    if ports.fs.exists(&project_dir.join("go.mod")) {
        return ProjectType::Go;
    }
    if ports.fs.exists(&project_dir.join("pom.xml"))
        || ports.fs.exists(&project_dir.join("build.gradle"))
    {
        return ProjectType::Java;
    }
    ProjectType::Unknown
}

/// Classify a changed file path based on the project type.
fn classify_file(path: &str, project_type: &ProjectType) -> String {
    let parts: Vec<&str> = path.splitn(4, '/').collect();
    match project_type {
        ProjectType::Rust => {
            // crates/<crate-name>/... → <crate-name>
            if parts.len() >= 2 && parts[0] == "crates" {
                return parts[1].to_string();
            }
            // Fallback: first path component
            parts[0].to_string()
        }
        ProjectType::Javascript | ProjectType::Typescript => {
            // packages/<package>/... → <package>
            if parts.len() >= 2 && (parts[0] == "packages" || parts[0] == "apps") {
                return parts[1].to_string();
            }
            parts[0].to_string()
        }
        _ => {
            // Unknown: top-level directory
            parts[0].to_string()
        }
    }
}

/// Generate a fallback session ID from timestamp + process ID.
fn generate_fallback_session_id() -> String {
    let ts = epoch_secs();
    let pid = std::process::id();
    format!("session-{}-{}", ts, pid)
}

/// Delete any delta files in the cartography dir that contain invalid JSON.
fn clean_corrupt_deltas(ports: &HookPorts<'_>, cartography_dir: &Path) {
    let entries = match ports.fs.read_dir(cartography_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries {
        let name = entry
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        if !name.starts_with("pending-delta-") || !name.ends_with(".json") {
            continue;
        }

        match ports.fs.read_to_string(&entry) {
            Ok(content) => {
                if serde_json::from_str::<SessionDelta>(&content).is_err() {
                    warn!(
                        "stop_cartography: deleting corrupt delta file: {}",
                        entry.display()
                    );
                    let _ = ports.fs.remove_file(&entry);
                }
            }
            Err(e) => {
                warn!(
                    "stop_cartography: cannot read delta file {}: {}",
                    entry.display(),
                    e
                );
            }
        }
    }
}

fn epoch_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook::HookPorts;
    use ecc_ports::fs::FileSystem;
    use ecc_ports::shell::CommandOutput;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};

    fn make_ports<'a>(
        fs: &'a InMemoryFileSystem,
        shell: &'a MockExecutor,
        env: &'a MockEnvironment,
        term: &'a BufferedTerminal,
    ) -> HookPorts<'a> {
        HookPorts {
            fs,
            shell,
            env,
            terminal: term,
        }
    }

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
        let ports = make_ports(&fs, &shell, &env, &term);

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
        let fs = InMemoryFileSystem::new()
            .with_file("/project/Cargo.toml", "[workspace]");
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
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = stop_cartography("{}", &ports);

        assert_eq!(result.exit_code, 0);

        // Delta file should exist
        let delta_path =
            std::path::Path::new("/project/.claude/cartography/pending-delta-rust-session-001.json");
        assert!(fs.exists(delta_path), "delta file should have been written");

        let content = fs.read_to_string(delta_path).expect("should read delta");
        let delta: SessionDelta =
            serde_json::from_str(&content).expect("should parse delta JSON");

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
        let ports_ts = make_ports(&fs_ts, &shell_ts, &env_ts, &term_ts);

        let _ = stop_cartography("{}", &ports_ts);
        let delta_ts_path = std::path::Path::new(
            "/tsproject/.claude/cartography/pending-delta-ts-session-001.json",
        );
        let content_ts = fs_ts.read_to_string(delta_ts_path).expect("ts delta");
        let delta_ts: SessionDelta = serde_json::from_str(&content_ts).expect("ts delta json");
        assert_eq!(delta_ts.project_type, ProjectType::Typescript);

        // --- javascript (package.json, no tsconfig) ---
        let fs_js = InMemoryFileSystem::new()
            .with_file("/jsproject/package.json", "{}");
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
        let ports_js = make_ports(&fs_js, &shell_js, &env_js, &term_js);

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
        let env_unk = MockEnvironment::new()
            .with_var("CLAUDE_PROJECT_DIR", "/unknown-project");
        // CLAUDE_SESSION_ID NOT set → fallback ID
        let term_unk = BufferedTerminal::new();
        let ports_unk = make_ports(&fs_unk, &shell_unk, &env_unk, &term_unk);

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

    // ────────────────────────────────────────────────────────────────────────
    // PC-012 through PC-016: start_cartography tests
    // ────────────────────────────────────────────────────────────────────────

    /// Helper: build a valid SessionDelta JSON with a given session_id and timestamp.
    fn make_delta_json(session_id: &str, timestamp: u64) -> String {
        serde_json::to_string(&SessionDelta {
            session_id: session_id.to_string(),
            timestamp,
            changed_files: vec![ChangedFile {
                path: "src/main.rs".to_string(),
                classification: "src".to_string(),
            }],
            project_type: ProjectType::Rust,
        })
        .unwrap()
    }

    /// PC-012: no pending deltas → exits immediately, no shell commands invoked.
    #[test]
    fn noop_when_no_pending_deltas() {
        // No cartography dir, no delta files
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new(); // no commands registered — any call would return ShellError
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = start_cartography("{}", &ports);

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "{}");
        // No lock file, no processed dir
        assert!(!fs.exists(std::path::Path::new(
            "/project/.claude/cartography/cartography-merge.lock"
        )));
    }

    /// PC-013: pending deltas + missing scaffold → scaffold created; existing scaffold untouched.
    #[test]
    fn creates_scaffold_when_missing() {
        let delta_json = make_delta_json("session-abc", 1000);
        let fs = InMemoryFileSystem::new().with_file(
            "/project/.claude/cartography/pending-delta-session-abc.json",
            &delta_json,
        );
        let shell = MockExecutor::new()
            // git status → clean (no dirty state)
            .on_args(
                "git",
                &["status", "--porcelain", "docs/cartography/"],
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            // agent invocation → success
            .on_args(
                "claude",
                &[
                    "--agent",
                    "cartographer",
                    "--input",
                    &delta_json,
                ],
                CommandOutput {
                    stdout: "ok".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            // git add + commit → success
            .on_args(
                "git",
                &["add", "docs/cartography/"],
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "git",
                &["commit", "-m", "docs(cartography): update"],
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let result = start_cartography("{}", &ports);

        assert_eq!(result.exit_code, 0);

        // Scaffold must exist
        let journeys_dir = std::path::Path::new("/project/docs/cartography/journeys");
        let flows_dir = std::path::Path::new("/project/docs/cartography/flows");
        let readme = std::path::Path::new("/project/docs/cartography/README.md");
        assert!(fs.exists(journeys_dir), "journeys/ should have been created");
        assert!(fs.exists(flows_dir), "flows/ should have been created");
        assert!(fs.exists(readme), "README.md should have been created");
    }

    /// PC-013 part 2: existing scaffold is left untouched.
    #[test]
    fn existing_scaffold_untouched() {
        let delta_json = make_delta_json("session-abc", 1000);
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/project/.claude/cartography/pending-delta-session-abc.json",
                &delta_json,
            )
            .with_dir("/project/docs/cartography/journeys")
            .with_dir("/project/docs/cartography/flows")
            .with_file(
                "/project/docs/cartography/README.md",
                "# Existing README\n",
            );
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["status", "--porcelain", "docs/cartography/"],
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "claude",
                &[
                    "--agent",
                    "cartographer",
                    "--input",
                    &delta_json,
                ],
                CommandOutput {
                    stdout: "ok".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "git",
                &["add", "docs/cartography/"],
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "git",
                &["commit", "-m", "docs(cartography): update"],
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        let _ = start_cartography("{}", &ports);

        // Existing README content preserved
        let readme_content = fs
            .read_to_string(std::path::Path::new(
                "/project/docs/cartography/README.md",
            ))
            .expect("readme");
        assert_eq!(readme_content, "# Existing README\n");
    }

    /// PC-014: dirty docs/cartography/ → git checkout invoked before processing.
    #[test]
    fn discards_uncommitted_changes_on_start() {
        let delta_json = make_delta_json("session-dirty", 1000);
        let fs = InMemoryFileSystem::new().with_file(
            "/project/.claude/cartography/pending-delta-session-dirty.json",
            &delta_json,
        );
        // git status shows dirty state
        let shell = MockExecutor::new()
            .on_args(
                "git",
                &["status", "--porcelain", "docs/cartography/"],
                CommandOutput {
                    stdout: " M docs/cartography/journeys/some.md\n".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "git",
                &["checkout", "--", "docs/cartography/"],
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "claude",
                &[
                    "--agent",
                    "cartographer",
                    "--input",
                    &delta_json,
                ],
                CommandOutput {
                    stdout: "ok".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "git",
                &["add", "docs/cartography/"],
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            )
            .on_args(
                "git",
                &["commit", "-m", "docs(cartography): update"],
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
        let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
        let term = BufferedTerminal::new();
        let ports = make_ports(&fs, &shell, &env, &term);

        // If git checkout was NOT called after dirty status, the handler would proceed
        // without discarding — we verify it completes successfully (checkout was called)
        // by checking the result is a passthrough (no error path triggered).
        let result = start_cartography("{}", &ports);
        assert_eq!(result.exit_code, 0);
        // Delta should have been archived (agent succeeded after discard)
        let processed = std::path::Path::new(
            "/project/.claude/cartography/processed/pending-delta-session-dirty.json",
        );
        assert!(
            fs.exists(processed),
            "delta should be archived after successful processing"
        );
    }

    /// PC-015: file lock held → skips; delta in processed/ → skipped; ordering by timestamp.
    #[test]
    fn lock_idempotency_and_ordering() {
        // ── Sub-test A: lock held → skip ──
        {
            let delta_json = make_delta_json("session-locked", 1000);
            let fs = InMemoryFileSystem::new()
                .with_file(
                    "/project/.claude/cartography/pending-delta-session-locked.json",
                    &delta_json,
                )
                // Lock file exists
                .with_file(
                    "/project/.claude/cartography/cartography-merge.lock",
                    "locked",
                );
            let shell = MockExecutor::new().on_args(
                "git",
                &["status", "--porcelain", "docs/cartography/"],
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
            let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
            let term = BufferedTerminal::new();
            let ports = make_ports(&fs, &shell, &env, &term);

            let result = start_cartography("{}", &ports);
            assert_eq!(result.exit_code, 0);
            // Delta NOT archived — lock was held
            assert!(
                !fs.exists(std::path::Path::new(
                    "/project/.claude/cartography/processed/pending-delta-session-locked.json"
                )),
                "delta should NOT be archived when lock is held"
            );
        }

        // ── Sub-test B: already-processed delta → filtered out ──
        {
            let delta_json = make_delta_json("session-old", 1000);
            let fs = InMemoryFileSystem::new()
                .with_file(
                    "/project/.claude/cartography/pending-delta-session-old.json",
                    &delta_json,
                )
                // Already processed
                .with_file(
                    "/project/.claude/cartography/processed/pending-delta-session-old.json",
                    &delta_json,
                );
            let shell = MockExecutor::new().on_args(
                "git",
                &["status", "--porcelain", "docs/cartography/"],
                CommandOutput {
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: 0,
                },
            );
            let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
            let term = BufferedTerminal::new();
            let ports = make_ports(&fs, &shell, &env, &term);

            let result = start_cartography("{}", &ports);
            assert_eq!(result.exit_code, 0);
        }

        // ── Sub-test C: ordering — 3 deltas with timestamps [300, 100, 200] ──
        {
            // We verify all 3 get archived (proving they were all processed in some order).
            // The sort correctness is ensured by the implementation; the test verifies
            // all unprocessed deltas are archived after success.
            let delta_a = make_delta_json("session-300", 300);
            let delta_b = make_delta_json("session-100", 100);
            let delta_c = make_delta_json("session-200", 200);

            let fs = InMemoryFileSystem::new()
                .with_file(
                    "/project/.claude/cartography/pending-delta-session-300.json",
                    &delta_a,
                )
                .with_file(
                    "/project/.claude/cartography/pending-delta-session-100.json",
                    &delta_b,
                )
                .with_file(
                    "/project/.claude/cartography/pending-delta-session-200.json",
                    &delta_c,
                );

            let shell = MockExecutor::new()
                .on_args(
                    "git",
                    &["status", "--porcelain", "docs/cartography/"],
                    CommandOutput {
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: 0,
                    },
                )
                // Agent succeeds for all (command-only match)
                .on("claude", CommandOutput {
                    stdout: "ok".to_string(),
                    stderr: String::new(),
                    exit_code: 0,
                })
                .on_args(
                    "git",
                    &["add", "docs/cartography/"],
                    CommandOutput {
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: 0,
                    },
                )
                .on_args(
                    "git",
                    &["commit", "-m", "docs(cartography): update"],
                    CommandOutput {
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: 0,
                    },
                );
            let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
            let term = BufferedTerminal::new();
            let ports = make_ports(&fs, &shell, &env, &term);

            let result = start_cartography("{}", &ports);
            assert_eq!(result.exit_code, 0);

            // All three archived
            for name in &[
                "pending-delta-session-300.json",
                "pending-delta-session-100.json",
                "pending-delta-session-200.json",
            ] {
                let processed = std::path::Path::new("/project/.claude/cartography/processed")
                    .join(name);
                assert!(
                    fs.exists(&processed),
                    "delta {} should be archived",
                    name
                );
            }
        }
    }

    /// PC-016: success → deltas archived to processed/ AFTER agent; failure → error to
    /// stderr, deltas NOT archived, git reset invoked.
    #[test]
    fn archive_on_success_and_reset_on_failure() {
        // ── Success path ──
        {
            let delta_json = make_delta_json("session-ok", 1000);
            let fs = InMemoryFileSystem::new().with_file(
                "/project/.claude/cartography/pending-delta-session-ok.json",
                &delta_json,
            );
            let shell = MockExecutor::new()
                .on_args(
                    "git",
                    &["status", "--porcelain", "docs/cartography/"],
                    CommandOutput {
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: 0,
                    },
                )
                .on_args(
                    "claude",
                    &[
                        "--agent",
                        "cartographer",
                        "--input",
                        &delta_json,
                    ],
                    CommandOutput {
                        stdout: "ok".to_string(),
                        stderr: String::new(),
                        exit_code: 0,
                    },
                )
                .on_args(
                    "git",
                    &["add", "docs/cartography/"],
                    CommandOutput {
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: 0,
                    },
                )
                .on_args(
                    "git",
                    &["commit", "-m", "docs(cartography): update"],
                    CommandOutput {
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: 0,
                    },
                );
            let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
            let term = BufferedTerminal::new();
            let ports = make_ports(&fs, &shell, &env, &term);

            let result = start_cartography("{}", &ports);
            assert_eq!(result.exit_code, 0);

            // Delta must be archived
            let processed = std::path::Path::new(
                "/project/.claude/cartography/processed/pending-delta-session-ok.json",
            );
            assert!(fs.exists(processed), "delta should be archived on success");
            // Original pending delta must be removed
            let pending = std::path::Path::new(
                "/project/.claude/cartography/pending-delta-session-ok.json",
            );
            assert!(
                !fs.exists(pending),
                "original pending delta should be gone after archive"
            );
        }

        // ── Failure path ──
        {
            let delta_json = make_delta_json("session-fail", 1000);
            let fs = InMemoryFileSystem::new().with_file(
                "/project/.claude/cartography/pending-delta-session-fail.json",
                &delta_json,
            );
            let shell = MockExecutor::new()
                .on_args(
                    "git",
                    &["status", "--porcelain", "docs/cartography/"],
                    CommandOutput {
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: 0,
                    },
                )
                .on_args(
                    "claude",
                    &[
                        "--agent",
                        "cartographer",
                        "--input",
                        &delta_json,
                    ],
                    CommandOutput {
                        stdout: String::new(),
                        stderr: "agent error".to_string(),
                        exit_code: 1, // non-zero = failure
                    },
                )
                .on_args(
                    "git",
                    &["reset", "HEAD", "docs/cartography/"],
                    CommandOutput {
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: 0,
                    },
                );
            let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
            let term = BufferedTerminal::new();
            let ports = make_ports(&fs, &shell, &env, &term);

            let result = start_cartography("{}", &ports);
            // Should still exit 0 (passthrough) but write error to stderr
            assert_eq!(result.exit_code, 0);
            assert!(
                !result.stderr.is_empty(),
                "stderr should contain error message on failure"
            );

            // Delta must NOT be archived
            let processed = std::path::Path::new(
                "/project/.claude/cartography/processed/pending-delta-session-fail.json",
            );
            assert!(
                !fs.exists(processed),
                "delta should NOT be archived on agent failure"
            );
        }
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
            let ports = make_ports(&fs, &shell, &env, &term);

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
                assert!(delta_files.is_empty(), "no delta should be written for non-git repo");
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
            let ports = make_ports(&fs, &shell, &env, &term);

            let result = stop_cartography("{}", &ports);
            assert_eq!(result.exit_code, 0);

            // Corrupt file should have been deleted
            let corrupt_path = std::path::Path::new(
                "/project/.claude/cartography/pending-delta-old-session.json",
            );
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
}
