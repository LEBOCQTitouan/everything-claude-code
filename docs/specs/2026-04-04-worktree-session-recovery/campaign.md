# Campaign: Worktree Session Recovery

## Bug Description

When a worktree is deleted (e.g., after session-end merge), if the Claude Code session's CWD was still bound to that worktree directory, all shell operations fail because the directory no longer exists. This is a session-level limitation — the user must restart Claude Code from the main repo directory.

## Detected Toolchain

- Test: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Build: `cargo build`

## Agent Outputs

### Investigation (code-reviewer)
Root cause: session_merge.rs and merge.rs delete worktree directory while Claude Code's CWD is bound to it. POSIX invariant prevents child processes from changing parent CWD. Two code paths: zero-commit cleanup (line 41) and merge cleanup (cleanup_worktree). No existing tests cover the orphaned-CWD scenario.

### Blast Radius (architect)
4 files affected across adapter/application layers. No port/domain changes needed. Recommended: deferred cleanup + session-start gc. ~50-80 lines. Key risk: worktree accumulation if gc fails.

### Web Research
- git worktree remove fails if CWD is inside worktree (POSIX)
- Claude Code issue #29260: hooks fail with ENOENT when CWD deleted
- Established pattern: parent must validate CWD before spawning hooks
- Signaling alternatives (temp file, stdout, OSC 7) all insufficient

## Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Root cause vs symptom | Root cause — ECC deletes worktree while session CWD is bound | Recommended |
| 2 | Minimal vs proper fix | Deferred cleanup: remove deletion from merge/hook, add session-start gc | Recommended |
| 3 | Missing tests | All 4 scenarios: deferred merge, deferred empty, session-start gc, gc skip active | Recommended |
| 4 | Regression risk | Accept architect's list: accumulation, startup perf, PID race, message wording | Recommended |
| 5 | Related audit findings | Fix CONV-002 (anyhow leak in worktree.rs) alongside | User |
| 6 | Reproducibility | Derive from code: EnterWorktree → commit → session end → hook deletes → ENOENT | Recommended |
| 7 | Data impact | No data migration needed — purely process CWD issue | Recommended |
