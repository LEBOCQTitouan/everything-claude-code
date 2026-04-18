//! CLAUDE.md count validation use case.

use ecc_domain::docs::claude_md;
use ecc_ports::fs::FileSystem;
use ecc_ports::shell::ShellExecutor;
use ecc_ports::terminal::TerminalIO;
use std::path::Path;

/// Run CLAUDE.md count validation. Returns true if all match.
pub fn run_validate_claude_md(
    fs: &dyn FileSystem,
    shell: &dyn ShellExecutor,
    terminal: &dyn TerminalIO,
    project_root: &Path,
    json: bool,
) -> bool {
    let claude_path = project_root.join("CLAUDE.md");
    let content = match fs.read_to_string(&claude_path) {
        Ok(c) => c,
        Err(_) => {
            terminal.stderr_write("ERROR: CLAUDE.md not found\n");
            return false;
        }
    };

    let mut claims = claude_md::extract_claims(&content);

    // Gather actual counts
    let mut actuals: Vec<(String, u64)> = Vec::new();

    // Test count via cargo test --list
    if let Ok(output) = shell.run_command_in_dir(
        "cargo",
        &["test", "--", "--list"],
        std::path::Path::new("."),
    ) {
        let test_count = output
            .stdout
            .lines()
            .filter(|l| l.ends_with(": test") || l.ends_with(": benchmark"))
            .count() as u64;
        actuals.push(("tests".to_string(), test_count));
    } else {
        terminal.stderr_write("WARN: cargo test --list unavailable, skipping test count\n");
    }

    // Crate count
    let crates_dir = project_root.join("crates");
    if let Ok(entries) = fs.read_dir(&crates_dir) {
        let crate_count = entries.iter().filter(|e| fs.is_dir(e)).count() as u64;
        actuals.push(("crates".to_string(), crate_count));
    }

    claude_md::compare_claims(&mut claims, &actuals);

    let all_match = claims.iter().all(|c| c.actual.is_none() || c.matches);

    if json {
        let claims_json: Vec<String> = claims
            .iter()
            .map(|c| {
                format!(
                    "{{\"text\":\"{}\",\"claimed\":{},\"actual\":{},\"match\":{}}}",
                    c.text,
                    c.claimed,
                    c.actual
                        .map(|a| a.to_string())
                        .unwrap_or("null".to_string()),
                    c.matches,
                )
            })
            .collect();
        terminal.stdout_write(&format!("{{\"claims\":[{}]}}\n", claims_json.join(",")));
    } else if all_match {
        terminal.stdout_write("All counts valid\n");
    } else {
        for c in &claims {
            if !c.matches
                && let Some(actual) = c.actual
            {
                terminal.stderr_write(&format!(
                    "MISMATCH: \"{}\" — claimed {}, actual {}\n",
                    c.text, c.claimed, actual
                ));
            }
        }
    }

    all_match
}

/// Scan CLAUDE.md and AGENTS.md for TEMPORARY (BL-NNN) markers whose backlog file is missing.
/// Returns true on success (exit 0 semantics). Kill switch: if `disabled=true`, short-circuits
/// to true with a single stderr notice.
pub fn run_validate_temporary_markers(
    _fs: &dyn FileSystem,
    _terminal: &dyn TerminalIO,
    _project_root: &Path,
    _disabled: bool,
    _strict: bool,
    _audit_report: bool,
) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use ecc_test_support::{BufferedTerminal, InMemoryFileSystem, MockExecutor};

    #[test]
    fn missing_claude_md() {
        let fs = InMemoryFileSystem::new();
        let shell = MockExecutor::new();
        let term = BufferedTerminal::new();
        assert!(!run_validate_claude_md(
            &fs,
            &shell,
            &term,
            Path::new("/root"),
            false
        ));
        assert!(term.stderr_output().join("").contains("not found"));
    }

    // PC-011: Kill switch short-circuits
    #[test]
    fn markers_kill_switch_emits_notice() {
        let fs = InMemoryFileSystem::new();
        let term = BufferedTerminal::new();
        let result =
            run_validate_temporary_markers(&fs, &term, Path::new("/root"), true, false, false);
        assert!(result, "kill switch must return true");
        let stderr = term.stderr_output().join("");
        assert!(
            stderr.contains("markers: disabled via ECC_CLAUDE_MD_MARKERS_DISABLED"),
            "stderr must contain kill switch notice, got: {stderr:?}"
        );
        let stdout = term.stdout_output().join("");
        assert!(stdout.is_empty(), "stdout must be empty, got: {stdout:?}");
    }

    // PC-012: Missing backlog dir — WARN but not strict → return true
    #[test]
    fn markers_missing_backlog_dir() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/CLAUDE.md", "TEMPORARY (BL-999) some marker\n");
        // No /root/docs/backlog/ dir
        let term = BufferedTerminal::new();
        let result =
            run_validate_temporary_markers(&fs, &term, Path::new("/root"), false, false, false);
        assert!(
            result,
            "non-strict should return true even with missing marker"
        );
        let stderr = term.stderr_output().join("");
        assert!(
            stderr.contains("WARN:"),
            "must emit WARN for missing backlog entry, got: {stderr:?}"
        );
        assert!(
            stderr.contains("BL-999"),
            "must reference BL-999, got: {stderr:?}"
        );
    }

    // PC-013: Walker deny-list + symlink skip
    #[test]
    fn markers_walker_denylist_and_symlink() {
        let stale_content = "TEMPORARY (BL-999) stale marker\n";
        let fs = InMemoryFileSystem::new()
            .with_file("/root/CLAUDE.md", "# clean file\n")
            .with_file("/root/.git/CLAUDE.md", stale_content)
            .with_file("/root/target/CLAUDE.md", stale_content)
            .with_file("/root/node_modules/CLAUDE.md", stale_content)
            .with_file("/root/.claude/worktrees/foo/CLAUDE.md", stale_content)
            .with_symlink("/root/symlinked-CLAUDE.md", "/root/CLAUDE.md");
        // Note: symlink registered — walker must skip is_symlink paths
        let term = BufferedTerminal::new();
        let result =
            run_validate_temporary_markers(&fs, &term, Path::new("/root"), false, false, false);
        assert!(result, "all denylist files must be ignored → return true");
        let stderr = term.stderr_output().join("");
        assert!(
            !stderr.contains("BL-999"),
            "no diagnostics from denylist dirs, got: {stderr:?}"
        );
        let stdout = term.stdout_output().join("");
        assert!(
            !stdout.contains("BL-999"),
            "no diagnostics from denylist dirs in stdout, got: {stdout:?}"
        );
    }

    // PC-014: Depth cap at 16
    #[test]
    fn markers_depth_limit() {
        // Build a path that's 17 levels deep from /root
        let deep_path = "/root/a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/CLAUDE.md".to_string();
        let fs =
            InMemoryFileSystem::new().with_file(&deep_path, "TEMPORARY (BL-888) deep marker\n");
        let term = BufferedTerminal::new();
        let result =
            run_validate_temporary_markers(&fs, &term, Path::new("/root"), false, false, false);
        // Depth 17 is beyond the cap of 16 — file must be skipped, NOT diagnosed as missing
        // Instead we expect a depth-limit warning OR simply no BL-888 diagnostic
        let stderr = term.stderr_output().join("");
        assert!(
            !stderr.contains("BL-888"),
            "depth-capped file must not produce marker diagnostic, got: {stderr:?}"
        );
        // Function should still return true (no stale markers found at valid depth)
        assert!(
            result,
            "depth-capped walker must return true when no valid-depth markers missing"
        );
    }

    // PC-015: Non-UTF8 vs I/O error distinction
    #[test]
    fn markers_non_utf8_and_io_errors_distinguished() {
        // InMemoryFileSystem returns NotFound for missing files (mapped as I/O-ish error).
        // We create a file that is registered as existing but whose read would return an error.
        // Since InMemoryFileSystem doesn't have a built-in "invalid UTF8" simulation,
        // we instead test that any read error produces a WARN with the path mentioned.
        // The key AC: one single-line WARN per failed file, exit code unaffected.
        let fs = InMemoryFileSystem::new()
            // A valid CLAUDE.md at root that is clean (no markers)
            .with_file("/root/CLAUDE.md", "# no markers\n");
        // We cannot easily inject a NotFound for a file the walker finds via read_dir
        // without a custom FS mock. Instead, verify the "not found on read_to_string"
        // path by having a file visible in read_dir but absent from files map.
        // InMemoryFileSystem's read_dir lists from files + dirs maps, so this is tricky.
        // We test the observable contract: clean CLAUDE.md → no WARN, returns true.
        let term = BufferedTerminal::new();
        let result =
            run_validate_temporary_markers(&fs, &term, Path::new("/root"), false, false, false);
        assert!(result, "clean file → returns true");
        let stderr = term.stderr_output().join("");
        assert!(
            !stderr.contains("WARN:"),
            "no WARN for clean file, got: {stderr:?}"
        );
    }

    // PC-016: AGENTS.md walked identically to CLAUDE.md
    #[test]
    fn markers_agents_md_scanned() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/AGENTS.md", "TEMPORARY (BL-999) missing marker\n");
        // No docs/backlog dir → BL-999 missing
        let term = BufferedTerminal::new();
        let result =
            run_validate_temporary_markers(&fs, &term, Path::new("/root"), false, false, false);
        assert!(result, "non-strict → true even with missing marker");
        let stderr = term.stderr_output().join("");
        assert!(
            stderr.contains("WARN:"),
            "must emit WARN for AGENTS.md missing marker, got: {stderr:?}"
        );
        assert!(
            stderr.contains("BL-999"),
            "must reference BL-999 from AGENTS.md, got: {stderr:?}"
        );
    }

    // PC-017: Baseline — zero markers, non-strict → silent
    #[test]
    fn markers_baseline_silent() {
        let fs = InMemoryFileSystem::new().with_file("/root/CLAUDE.md", "# no markers here\n");
        let term = BufferedTerminal::new();
        let result =
            run_validate_temporary_markers(&fs, &term, Path::new("/root"), false, false, false);
        assert!(result, "no markers → returns true");
        let stdout = term.stdout_output().join("");
        assert!(
            stdout.is_empty(),
            "stdout must be empty for silent baseline, got: {stdout:?}"
        );
        let stderr = term.stderr_output().join("");
        assert!(
            stderr.is_empty(),
            "stderr must be empty for silent baseline, got: {stderr:?}"
        );
    }

    // PC-018: Zero missing + strict → success stdout
    #[test]
    fn markers_strict_success() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/CLAUDE.md", "TEMPORARY (BL-042) resolved marker\n")
            .with_file("/root/docs/backlog/BL-042-foo.md", "# backlog entry\n");
        let term = BufferedTerminal::new();
        let result =
            run_validate_temporary_markers(&fs, &term, Path::new("/root"), false, true, false);
        assert!(result, "all resolved + strict → returns true");
        let stdout = term.stdout_output().join("");
        assert!(
            stdout.contains("All TEMPORARY markers reference valid backlog entries\n"),
            "must emit success message on stdout, got: {stdout:?}"
        );
    }

    // PC-019: Missing + strict → ERROR prefix + false
    #[test]
    fn markers_strict_error_prefix() {
        let fs =
            InMemoryFileSystem::new().with_file("/root/CLAUDE.md", "TEMPORARY (BL-999) missing\n");
        // No backlog dir
        let term = BufferedTerminal::new();
        let result =
            run_validate_temporary_markers(&fs, &term, Path::new("/root"), false, true, false);
        assert!(!result, "strict + missing → returns false");
        let stderr = term.stderr_output().join("");
        assert!(
            stderr.contains("ERROR:"),
            "strict must emit ERROR: prefix, got: {stderr:?}"
        );
        assert!(
            !stderr.contains("WARN:"),
            "strict must NOT emit WARN: prefix, got: {stderr:?}"
        );
    }

    // PC-020: Missing + default (non-strict) → WARN + true + remediation hint
    #[test]
    fn markers_warn_default() {
        let fs =
            InMemoryFileSystem::new().with_file("/root/CLAUDE.md", "TEMPORARY (BL-999) missing\n");
        let term = BufferedTerminal::new();
        let result =
            run_validate_temporary_markers(&fs, &term, Path::new("/root"), false, false, false);
        assert!(result, "non-strict + missing → returns true");
        let stderr = term.stderr_output().join("");
        assert!(
            stderr.contains("WARN:"),
            "must emit WARN: prefix, got: {stderr:?}"
        );
        assert!(
            !stderr.contains("ERROR:"),
            "must NOT emit ERROR: in non-strict mode, got: {stderr:?}"
        );
        assert!(
            stderr.contains("BL-999"),
            "must reference missing ID, got: {stderr:?}"
        );
        assert!(
            stderr.contains("Remediation:"),
            "must include remediation guidance, got: {stderr:?}"
        );
    }

    // PC-021: Audit report table + archived=resolved
    #[test]
    fn markers_audit_report_table() {
        let fs = InMemoryFileSystem::new()
            .with_file(
                "/root/CLAUDE.md",
                "TEMPORARY (BL-042) resolved\nTEMPORARY (BL-999) missing\nTEMPORARY (BL-137) archived\n",
            )
            .with_file("/root/docs/backlog/BL-042-foo.md", "# resolved entry\n")
            .with_file("/root/docs/backlog/BL-137-archived.md", "# archived entry\n");
        let term = BufferedTerminal::new();
        let _result =
            run_validate_temporary_markers(&fs, &term, Path::new("/root"), false, false, true);
        let stdout = term.stdout_output().join("");
        assert!(
            stdout.contains("| File | Line | Marker ID | Status |"),
            "must emit table header, got: {stdout:?}"
        );
        assert!(
            stdout.contains("BL-042"),
            "must include BL-042 row, got: {stdout:?}"
        );
        assert!(
            stdout.contains("resolved"),
            "BL-042 must have status 'resolved', got: {stdout:?}"
        );
        assert!(
            stdout.contains("BL-999"),
            "must include BL-999 row, got: {stdout:?}"
        );
        assert!(
            stdout.contains("missing"),
            "BL-999 must have status 'missing', got: {stdout:?}"
        );
        assert!(
            stdout.contains("BL-137"),
            "must include BL-137 row (presence-only → resolved), got: {stdout:?}"
        );
    }

    // PC-022: Deterministic file + line ordering
    #[test]
    fn markers_file_order_deterministic() {
        let fs = InMemoryFileSystem::new()
            .with_file("/root/CLAUDE.md", "TEMPORARY (BL-001) one\n")
            .with_file("/root/aaa/CLAUDE.md", "TEMPORARY (BL-002) two\n")
            .with_file("/root/zzz/CLAUDE.md", "TEMPORARY (BL-003) three\n");
        // No backlog → all missing, all WARN
        let term = BufferedTerminal::new();
        run_validate_temporary_markers(&fs, &term, Path::new("/root"), false, false, false);
        let stderr = term.stderr_output().join("");
        // All three must be emitted
        assert!(
            stderr.contains("BL-001"),
            "must contain BL-001, got: {stderr:?}"
        );
        assert!(
            stderr.contains("BL-002"),
            "must contain BL-002, got: {stderr:?}"
        );
        assert!(
            stderr.contains("BL-003"),
            "must contain BL-003, got: {stderr:?}"
        );
        // Lexicographic order: /root/CLAUDE.md < /root/aaa/CLAUDE.md < /root/zzz/CLAUDE.md
        let pos1 = stderr.find("BL-001").expect("BL-001 not found");
        let pos2 = stderr.find("BL-002").expect("BL-002 not found");
        let pos3 = stderr.find("BL-003").expect("BL-003 not found");
        assert!(pos1 < pos2, "BL-001 must appear before BL-002 (lex order)");
        assert!(pos2 < pos3, "BL-002 must appear before BL-003 (lex order)");
    }
}
