# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A collection of production-ready agents, skills, hooks, commands, rules, and MCP configurations for software development using Claude Code. Core CLI is implemented in Rust (Hexagonal Architecture, DDD, Clean Code).

## Running Tests

```bash
cargo test              # Run all Rust tests (1562 tests)
bats tests/statusline/  # Run statusline Bats tests (16 tests)
cargo clippy -- -D warnings  # Lint with zero warnings
cargo build --release   # Build release binary
```

## Architecture

Hexagonal architecture: domain → ports → app → infra → CLI (9 crates). `ecc-workflow` is a standalone binary for workflow state management. `ecc-flock` is a shared POSIX flock utility. See `docs/ARCHITECTURE.md` for full structure.

## CLI Commands

```
ecc version          Show version
ecc install          Install ECC config to ~/.claude/
ecc init             Initialize ECC in current project
ecc audit            Audit ECC configuration health
ecc hook <id> [profiles]  Run a hook by ID
ecc validate <target>     Validate content files (agents|commands|hooks|skills|rules|paths)
ecc validate spec <path>  Validate spec artifact (AC numbering, sequential IDs, no gaps)
ecc validate design <path> [--spec <spec-path>]  Validate design artifact (PC table, AC coverage, dependency order)
ecc dev on|off|status     Toggle ECC config on/off
ecc dev switch dev|default [--dry-run]  Instant config switching via symlinks
ecc validate statusline   Verify statusline installation
ecc backlog next-id       Next sequential BL-NNN ID
ecc backlog check-duplicates <title> [--tags t1,t2]  Check for duplicate entries
ecc backlog reindex [--dry-run]  Regenerate BACKLOG.md from files
ecc worktree gc [--force]  Clean up stale session worktrees
ecc claw                  NanoClaw interactive REPL
ecc completion <shell>    Generate shell completions
```

## Slash Commands

Audit commands (`/audit-full`, `/audit-archi`, `/audit-code`, `/audit-convention`, `/audit-doc`, `/audit-errors`, `/audit-evolution`, `/audit-observability`, `/audit-security`, `/audit-test`, `/audit-web`) and side commands (`/verify`, `/review`, `/backlog`, `/build-fix`, `/catchup`, `/commit`, `/ecc-test-mode`): see `docs/commands-reference.md`.

### Spec-Driven Pipeline (Doc-First)

`/spec` → `/spec-dev`, `/spec-fix`, `/spec-refactor` → `/design` → `/implement`

- `/spec` auto-classifies intent (dev/fix/refactor) and delegates to the matching `/spec-*` command
- Each `/spec-*` runs web research, grill-me interview, enters **Plan Mode** for doc-first review (spec draft + upper-level doc preview), adversarial review, then outputs the spec
- `/design` produces the technical design, enters **Plan Mode** for architecture preview (arch docs, diagrams, bounded contexts), then adversarial review
- `/implement` enters **Plan Mode** for implementation steps, then executes TDD loops with TaskCreate tracking and mandatory doc updates
- All three phases use Plan Mode so the user reviews artifacts before execution
- State machine in `.claude/workflow/` enforces phase ordering via `ecc-workflow` binary

## Command Workflows

Slash command workflows defined in `commands/` are mandatory. Follow every phase and step exactly as specified — do not skip, reorder, or modify phases. The spec-driven pipeline is enforced by `.claude/workflow/state.json` and hook-based gates.

## Doc Hierarchy

`CLAUDE.md` (onboarding) → `docs/getting-started.md` (human setup) → `docs/ARCHITECTURE.md` (system design) → `docs/adr/` (decisions) → `docs/specs/` (persisted spec+design artifacts per work item) → `docs/domain/bounded-contexts.md` (domain model) → `docs/runbooks/` (ops) → `docs/MODULE-SUMMARIES.md` (per-crate reference). Information lives at the lowest layer that serves its audience; CLAUDE.md stays terse.

## Dual-Mode Development

- **Spec-driven** (`/spec` or `/spec-*` → `/design` → `/implement`): for features, fixes, and refactors requiring design review.
- **Direct** (edit + `/verify`): for small, well-understood changes
- Use `/audit-*` independently at any time for health checks
- Use `/review` as a final craft conscience gate before shipping

## Gotchas

- `ecc-domain` crate must have zero I/O imports — pure business logic only (enforced by hook)
- Agent frontmatter `model` field controls which Claude model runs the agent — wrong value silently degrades quality
- `hooks.json` lives in `hooks/`, not the project root
- Skill directory name must match the `name` field in its frontmatter
- Test count in CLAUDE.md (currently 1562) must be updated after adding or removing tests
- ECC hooks are bypassed by default via `.envrc` (`ECC_WORKFLOW_BYPASS=1`) — to test the pipeline: `ECC_WORKFLOW_BYPASS=0 claude` or use `/ecc-test-mode`

## Development Notes

- Source is Rust, organized as a Cargo workspace with 8 crates
- Hexagonal architecture: domain → ports → infra → app → CLI
- All I/O is abstracted behind port traits, enabling full in-memory testing
- Agent/skill/hook format: Markdown with YAML frontmatter (see `agents/`, `skills/`, `hooks/`)
- File naming: lowercase with hyphens (e.g., `python-reviewer.md`, `tdd-workflow.md`)
