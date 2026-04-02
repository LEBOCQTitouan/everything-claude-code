# ADR 0038: Scaffold Command Distribution

## Status
Accepted

## Context
Workflow templates need to be installed into a project's `.github/workflows/` directory (project-local), but `ecc install` targets `~/.claude/` (global). Options considered: extend `ecc init --with-workflows` (Rust changes in 3 crates, ~270 lines), new `ecc scaffold` subcommand (same scope), or a slash command (content-only, no Rust changes).

## Decision
Use a `/scaffold-workflows` slash command for v1 distribution. The command prompts users to select templates via `AskUserQuestion`, copies them verbatim to `.github/workflows/`, handles overwrite warnings, and supports `--dry-run`. No Rust code changes required.

## Consequences
- Zero Rust changes — pure content addition, fastest to ship
- Leverages Claude Code's existing file-writing capabilities
- Users get interactive template selection with overwrite protection
- Trade-off: no offline CLI support (requires active Claude Code session)
- Future: may graduate to `ecc init --with-workflows` in a later backlog item for offline support
