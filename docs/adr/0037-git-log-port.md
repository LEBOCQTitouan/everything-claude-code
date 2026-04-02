# ADR 0037: GitLogPort for Git Log Abstraction

## Status

Accepted

## Context

BL-071 adds four `ecc analyze` subcommands (changelog, hotspots, coupling, bus-factor) that all need git log data. The existing pattern for git operations (`ecc-app/src/worktree.rs`) passes `&dyn ShellExecutor` directly and embeds git CLI argument construction in the application layer. This works for one-off commands but becomes problematic when 4+ use cases all need similar git log queries with different `--format` and `--since` flags.

The architect agent flagged the worktree pattern as a hexagonal architecture violation: the application layer knows git CLI semantics (a leaked infrastructure detail), and testing requires mocking raw shell command strings rather than domain-meaningful operations.

## Decision

Introduce a dedicated `GitLogPort` trait in `ecc-ports` with methods that return domain types (`RawCommit`, `(String, String)` tuples). The `GitLogAdapter` in `ecc-infra` implements the trait by wrapping `ShellExecutor`, owning all git command construction and output parsing.

Methods:
- `log_with_files(repo_dir, since)` -> `Vec<RawCommit>`
- `log_file_authors(repo_dir, since)` -> `Vec<(String, String)>`

## Consequences

- **Positive**: App and domain layers remain pure — no git CLI knowledge leaks upward
- **Positive**: Use case tests inject `MockGitLog` returning canned domain data, not shell output strings
- **Positive**: All git format strings and argument construction are centralized in one adapter
- **Negative**: One more port trait to maintain (but it's small: 2 methods)
- **Precedent**: Future git-dependent features (e.g., blame analysis, commit search) should follow this pattern rather than the worktree shortcut
