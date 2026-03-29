# ADR-030: Task State Source of Truth

## Status

Accepted (2026-03-29)

## Context

The `/implement` command maintained task state in three parallel systems: tasks.md (file), TodoWrite (UI checklist), and TaskCreate/TaskUpdate (spinner + progress). The LLM manually constructed all three, causing drift when context compacted or sessions resumed. Status transitions were unvalidated.

## Decision

- **tasks.md is the single source of truth.** TodoWrite and TaskCreate are derived from it via `ecc-workflow tasks sync`.
- **Status transitions are validated** by a finite state machine in `ecc-domain::task::status` (pending→red→green→done, with failed branch and PostTdd shortcut).
- **Subcommands live in `ecc-workflow`**, not `ecc-cli`, because task management is a workflow-internal concern invoked by the `/implement` pipeline.
- **Domain types are pure** — all parsing, validation, and update functions in `ecc-domain::task` take `&str` and return `Result<T>`, with zero I/O imports.
- **Atomic writes** use `ecc-flock` with lock name `"tasks"` and tempfile+rename pattern.
- **Path traversal protection** uses `canonicalize` + `starts_with(project_dir)`.

## Consequences

- Eliminates three-way drift between tasks.md, TodoWrite, and TaskCreate
- Re-entry resumes from tasks.md state (no manual reconciliation needed)
- Invalid transitions are caught immediately at the CLI level
- Domain logic is fully testable with pure string inputs
