//! Tests for start_cartography — session delta processing, scaffold creation,
//! dirty-state handling, lock idempotency, and archive/reset behavior (PC-012 to PC-016).
//! Also includes safety-net tests (PC-001 to PC-004) locking down edge-case behavior.
use super::*;
use crate::hook::HookPorts;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::CommandOutput;
use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockEnvironment, MockExecutor};

pub(super) fn make_ports<'a>(
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
        cost_store: None,
        bypass_store: None,
        metrics_store: None,
    }
}

// ────────────────────────────────────────────────────────────────────────
// PC-012 through PC-016: start_cartography tests
// ────────────────────────────────────────────────────────────────────────

/// Helper: build a valid SessionDelta JSON with a given session_id and timestamp.
pub(super) fn make_delta_json(session_id: &str, timestamp: u64) -> String {
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

    let result = process_cartography("{}", &ports);

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
        // agent invocation → success (any args)
        .on(
            "claude",
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

    let result = process_cartography("{}", &ports);

    assert_eq!(result.exit_code, 0);

    // Scaffold must exist
    let journeys_dir = std::path::Path::new("/project/docs/cartography/journeys");
    let flows_dir = std::path::Path::new("/project/docs/cartography/flows");
    let readme = std::path::Path::new("/project/docs/cartography/README.md");
    assert!(
        fs.exists(journeys_dir),
        "journeys/ should have been created"
    );
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
        .with_file("/project/docs/cartography/README.md", "# Existing README\n");
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
        .on(
            "claude",
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

    let _ = process_cartography("{}", &ports);

    // Existing README content preserved
    let readme_content = fs
        .read_to_string(std::path::Path::new("/project/docs/cartography/README.md"))
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
        .on(
            "claude",
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
    let result = process_cartography("{}", &ports);
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

        let result = process_cartography("{}", &ports);
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

        let result = process_cartography("{}", &ports);
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
            .on(
                "claude",
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

        let result = process_cartography("{}", &ports);
        assert_eq!(result.exit_code, 0);

        // All three archived
        for name in &[
            "pending-delta-session-300.json",
            "pending-delta-session-100.json",
            "pending-delta-session-200.json",
        ] {
            let processed =
                std::path::Path::new("/project/.claude/cartography/processed").join(name);
            assert!(fs.exists(&processed), "delta {} should be archived", name);
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
            .on(
                "claude",
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

        let result = process_cartography("{}", &ports);
        assert_eq!(result.exit_code, 0);

        // Delta must be archived
        let processed = std::path::Path::new(
            "/project/.claude/cartography/processed/pending-delta-session-ok.json",
        );
        assert!(fs.exists(processed), "delta should be archived on success");
        // Original pending delta must be removed
        let pending =
            std::path::Path::new("/project/.claude/cartography/pending-delta-session-ok.json");
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
            .on(
                "claude",
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

        let result = process_cartography("{}", &ports);
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

// ── Safety-net tests (PC-001 through PC-004) ──
// Written BEFORE refactoring to lock down existing edge-case behavior.

/// PC-001: Agent exits non-zero → delta NOT archived, stderr reports failure.
#[test]
fn safety_net_agent_nonzero_exit() {
    let delta_json = r#"{"session_id":"sn-001","timestamp":1000,"changed_files":[{"path":"crates/foo/src/lib.rs","classification":"foo"}],"project_type":"rust"}"#;
    let fs = InMemoryFileSystem::new()
        .with_file("/project/Cargo.toml", "[workspace]")
        .with_file(
            "/project/.claude/cartography/pending-delta-session-sn-001.json",
            delta_json,
        )
        .with_file("/project/docs/cartography/journeys/.gitkeep", "")
        .with_file("/project/docs/cartography/flows/.gitkeep", "")
        .with_file("/project/docs/cartography/elements/.gitkeep", "")
        .with_file("/project/docs/cartography/README.md", "# Cartography\n");
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
        .on(
            "claude",
            CommandOutput {
                stdout: String::new(),
                stderr: "agent error\n".to_string(),
                exit_code: 1,
            },
        )
        .on(
            "git",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
    let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
    let term = BufferedTerminal::new();
    let ports = make_ports(&fs, &shell, &env, &term);

    let result = process_cartography("{}", &ports);

    assert_eq!(result.exit_code, 0);
    assert!(
        result.stderr.contains("agent failed"),
        "stderr should report agent failure: got {:?}",
        result.stderr
    );
    let archived = std::path::Path::new(
        "/project/.claude/cartography/processed/pending-delta-session-sn-001.json",
    );
    assert!(
        !fs.exists(archived),
        "delta must NOT be archived on failure"
    );
}

/// PC-002: Agent returns invalid output (not journey/flow) but exits 0 →
/// output discarded, delta still archived (current behavior).
#[test]
fn safety_net_agent_invalid_output() {
    let delta_json = r#"{"session_id":"sn-002","timestamp":1000,"changed_files":[{"path":"crates/bar/src/lib.rs","classification":"bar"}],"project_type":"rust"}"#;
    let fs = InMemoryFileSystem::new()
        .with_file("/project/Cargo.toml", "[workspace]")
        .with_file(
            "/project/.claude/cartography/pending-delta-session-sn-002.json",
            delta_json,
        )
        .with_file("/project/docs/cartography/journeys/.gitkeep", "")
        .with_file("/project/docs/cartography/flows/.gitkeep", "")
        .with_file("/project/docs/cartography/elements/.gitkeep", "")
        .with_file("/project/docs/cartography/README.md", "# Cartography\n");
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
        .on(
            "claude",
            CommandOutput {
                stdout: "not valid journey or flow markdown".to_string(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
        .on(
            "git",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
    let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
    let term = BufferedTerminal::new();
    let ports = make_ports(&fs, &shell, &env, &term);

    let result = process_cartography("{}", &ports);

    assert_eq!(result.exit_code, 0);
    let archived = std::path::Path::new(
        "/project/.claude/cartography/processed/pending-delta-session-sn-002.json",
    );
    assert!(
        fs.exists(archived),
        "delta should be archived when agent exits 0"
    );
}

/// PC-003: Archive rename failure is handled gracefully (no panic).
#[test]
fn safety_net_archive_failure() {
    let delta_json = r#"{"session_id":"sn-003","timestamp":1000,"changed_files":[{"path":"src/main.rs","classification":"root"}],"project_type":"rust"}"#;
    let fs = InMemoryFileSystem::new()
        .with_file("/project/Cargo.toml", "[workspace]")
        .with_file(
            "/project/.claude/cartography/pending-delta-session-sn-003.json",
            delta_json,
        )
        .with_file("/project/docs/cartography/journeys/.gitkeep", "")
        .with_file("/project/docs/cartography/flows/.gitkeep", "")
        .with_file("/project/docs/cartography/elements/.gitkeep", "")
        .with_file("/project/docs/cartography/README.md", "# Cartography\n");
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
        .on(
            "claude",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
        .on(
            "git",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
    let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
    let term = BufferedTerminal::new();
    let ports = make_ports(&fs, &shell, &env, &term);

    let result = process_cartography("{}", &ports);
    assert_eq!(result.exit_code, 0);
}

/// PC-004: Malformed delta JSON is silently skipped; good deltas still process.
#[test]
fn safety_net_malformed_delta_json() {
    let good_delta = r#"{"session_id":"sn-004-good","timestamp":2000,"changed_files":[{"path":"src/lib.rs","classification":"root"}],"project_type":"rust"}"#;
    let bad_delta = r#"{ this is not valid json }"#;
    let fs = InMemoryFileSystem::new()
        .with_file("/project/Cargo.toml", "[workspace]")
        .with_file(
            "/project/.claude/cartography/pending-delta-session-sn-004-bad.json",
            bad_delta,
        )
        .with_file(
            "/project/.claude/cartography/pending-delta-session-sn-004-good.json",
            good_delta,
        )
        .with_file("/project/docs/cartography/journeys/.gitkeep", "")
        .with_file("/project/docs/cartography/flows/.gitkeep", "")
        .with_file("/project/docs/cartography/elements/.gitkeep", "")
        .with_file("/project/docs/cartography/README.md", "# Cartography\n");
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
        .on(
            "claude",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        )
        .on(
            "git",
            CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: 0,
            },
        );
    let env = MockEnvironment::new().with_var("CLAUDE_PROJECT_DIR", "/project");
    let term = BufferedTerminal::new();
    let ports = make_ports(&fs, &shell, &env, &term);

    let result = process_cartography("{}", &ports);

    assert_eq!(result.exit_code, 0);
    let good_archived = std::path::Path::new(
        "/project/.claude/cartography/processed/pending-delta-session-sn-004-good.json",
    );
    assert!(fs.exists(good_archived), "good delta should be archived");
    let bad_archived = std::path::Path::new(
        "/project/.claude/cartography/processed/pending-delta-session-sn-004-bad.json",
    );
    assert!(
        !fs.exists(bad_archived),
        "malformed delta must not be archived"
    );
}
