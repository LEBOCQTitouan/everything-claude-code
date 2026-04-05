# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A collection of production-ready agents, skills, hooks, commands, rules, and MCP configurations for software development using Claude Code. Core CLI is implemented in Rust (Hexagonal Architecture, DDD, Clean Code).

## Running Tests

```bash
cargo test              # Run all Rust tests (997 tests)
cargo nextest run       # Faster test runner (~60% speedup, per-test isolation)
bats tests/statusline/  # Run statusline Bats tests (22 tests)
cargo clippy -- -D warnings  # Lint with zero warnings
cargo deny check        # Supply chain audit (licenses + advisories)
cargo llvm-cov --workspace   # Coverage report (works on macOS)
cargo build --release   # Build release binary
cargo mutants -p ecc-domain   # Mutation testing (domain crate)
cargo xtask mutants            # Structured mutation testing (all scoped crates)
cargo xtask mutants --in-diff  # Diff-scoped mutation testing
cargo dist build        # Local release build test (cargo-dist, requires cargo-dist installed)
```

## Architecture

Hexagonal architecture: domain → ports → app → infra → CLI (9 crates). `ecc-workflow` is a standalone binary for workflow state management. `ecc-flock` is a shared POSIX flock utility. `workflow-templates/` contains installable GitHub Actions YAML templates. See `docs/ARCHITECTURE.md` for full structure.

## CLI Commands

```
ecc analyze changelog [--since <tag|date>]  Generate conventional commit changelog
ecc analyze hotspots [--top N] [--since <tag|date>]  Show most frequently changed files
ecc analyze coupling [--threshold 0.7] [--since <tag|date>]  Show co-change file pairs
ecc analyze bus-factor [--top N] [--since <tag|date>]  Show files with single-author risk
ecc version          Show version
ecc install          Install ECC config to ~/.claude/
ecc init             Initialize ECC in current project
ecc audit            Audit ECC configuration health
ecc hook <id> [profiles]  Run a hook by ID
ecc validate <target>     Validate content files (agents|commands|hooks|skills|rules|paths|patterns)
ecc validate spec <path>  Validate spec artifact (AC numbering, sequential IDs, no gaps)
ecc validate design <path> [--spec <spec-path>]  Validate design artifact (PC table, AC coverage, dependency order)
ecc dev on|off|status     Toggle ECC config on/off
ecc dev switch dev|default [--dry-run]  Instant config switching via symlinks
ecc validate statusline   Verify statusline installation
ecc validate conventions  Validate naming, values, placement, and cross-references
ecc validate cartography [--coverage]  Validate cartography schema, staleness, and coverage
ecc status               Show workflow state, versions, component counts
ecc config set <key> <value>  Persist CLI preferences (~/.ecc/config.toml)
ecc log tail [--session <id>]  Live-tail current session logs
ecc log search <query> [--session <id>] [--since 2d] [--level warn]  FTS5 search
ecc log prune [--older-than 30d]  Clean up old logs
ecc log export --format json|csv [--since 7d]  Export filtered logs
ecc backlog next-id       Next sequential BL-NNN ID
ecc backlog check-duplicates <title> [--tags t1,t2]  Check for duplicate entries
ecc backlog reindex [--dry-run]  Regenerate BACKLOG.md from files
ecc worktree gc [--force]  Clean up stale session worktrees
ecc sources list          List all configured knowledge sources
ecc sources add <url>     Add a new knowledge source
ecc sources check         Check status of configured sources
ecc sources reindex       Reindex sources for search
ecc memory add --type <tier> --title <t> [--tags t1,t2]  Add memory entry
ecc memory search <query> [--type T] [--tag T]  Search memories (FTS5)
ecc memory list [--type T] [--tag T]  List memory entries
ecc memory delete <id>     Delete a memory entry
ecc memory promote <id>    Promote episodic to semantic
ecc memory migrate         Migrate legacy docs/memory/ to SQLite
ecc memory gc [--dry-run]  Garbage-collect stale memories
ecc memory stats           Show memory store statistics
ecc cost summary [--since 7d] [--model M]  Aggregated cost breakdown by model
ecc cost breakdown --by agent|model [--since 7d]  Per-agent or per-model breakdown
ecc cost compare --before DATE --after DATE  Before/after cost comparison
ecc cost export --format json|csv [--since 7d]  Export cost data
ecc cost prune [--older-than 90d]  Delete old cost records
ecc campaign init <spec-dir>    Create campaign.md for grill-me decision persistence
ecc campaign append-decision --question Q --answer A --source recommended|user  Append decision
ecc campaign show              Output campaign.md as JSON
ecc cost migrate  Import legacy JSONL data into SQLite
ecc audit-web profile init    Generate suggested audit profile from codebase
ecc audit-web profile show    Display current audit profile
ecc audit-web profile validate  Check profile structural correctness
ecc audit-web profile reset   Delete the audit profile
ecc audit-web validate-report <path>  Validate radar report structure
ecc claw                  NanoClaw interactive REPL
ecc completion <shell>    Generate shell completions
ecc status [--json]      Show diagnostic snapshot (versions, phase, components)
ecc update [--version <ver>] [--dry-run] [--pre]  Self-update from GitHub Releases
ecc config set <key> <value>  Set persistent config (e.g., log-level info)
ecc workflow init <concern> <feature>  Initialize workflow state
ecc workflow transition <target>      Advance workflow phase
ecc workflow status                   Show current workflow state
ecc workflow recover                  Archive stuck state and reset to idle
ecc workflow phase-gate               Gate writes during plan/solution phases
ecc workflow tasks sync <path>        Parse tasks.md, output JSON summary
ecc workflow tasks update <path> <id> <status>  Atomically update PC status
ecc workflow tasks init <design> --output <path>  Generate tasks.md from design PCs
ecc-workflow <subcommand>             Legacy alias (thin wrapper for ecc workflow)
cargo xtask deploy [--dry-run]  Full local machine deploy (build, install, completions, RC)
```

## Slash Commands

Audit commands (`/audit-full`, `/audit-archi`, `/audit-backlog`, `/audit-code`, `/audit-convention`, `/audit-doc`, `/audit-errors`, `/audit-evolution`, `/audit-observability`, `/audit-security`, `/audit-test`, `/audit-web`) and side commands (`/doc-suite`, `/verify`, `/review`, `/backlog`, `/build-fix`, `/catchup`, `/commit`, `/create-component`, `/ecc-test-mode`, `/scaffold-workflows`): see `docs/commands-reference.md`.

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

`CLAUDE.md` (onboarding) → `docs/getting-started.md` (human setup) → `docs/ARCHITECTURE.md` (system design) → `docs/adr/` (decisions) → `docs/specs/` (persisted spec+design artifacts per work item) → `docs/domain/bounded-contexts.md` (domain model) → `docs/sources.md` (curated knowledge sources — Technology Radar quadrants) → `docs/runbooks/` (ops) → `docs/MODULE-SUMMARIES.md` (per-crate reference). Information lives at the lowest layer that serves its audience; CLAUDE.md stays terse.

## Dual-Mode Development

- **Spec-driven** (`/spec` or `/spec-*` → `/design` → `/implement`): for features, fixes, and refactors requiring design review.
- **Direct** (edit + `/verify`): for small, well-understood changes
- Use `/audit-*` independently at any time for health checks
- Use `/review` as a final craft conscience gate before shipping

## Gotchas

- Workflow state is worktree-scoped: `resolve_state_dir()` resolves to `<git-dir>/ecc-workflow/state.json` in git repos (per-worktree isolation), falling back to `.claude/workflow/` for non-git dirs. Run `ecc-workflow status` from a worktree to verify isolation. If a stale `state.json` exists at `<main-repo>/.claude/workflow/` from a pre-fix session, delete it manually. Old state at `.claude/workflow/state.json` is auto-migrated on first write.
- `ecc workflow` mirrors `ecc-workflow` — use either during migration; `ecc-workflow` will become a thin wrapper
- `ecc-domain` crate must have zero I/O imports — pure business logic only (enforced by hook)
- Agent frontmatter `model` field controls which Claude model runs the agent — wrong value silently degrades quality
- Agent frontmatter `effort` field (low/medium/high/max) controls thinking budget via SubagentStart hook — must match model tier
- `hooks.json` lives in `hooks/`, not the project root
- Skill directory name must match the `name` field in its frontmatter
- Test count in CLAUDE.md must be updated after adding or removing tests
- `pre:edit-write:workflow-branch-guard` blocks `.github/workflows/` edits on main/master/production — create a feature branch first
- ECC hooks are bypassed by default via `.envrc` (`ECC_WORKFLOW_BYPASS=1`) — to test the pipeline: `ECC_WORKFLOW_BYPASS=0 claude` or use `/ecc-test-mode`
- `pre:write-edit:worktree-guard` blocks Write/Edit/MultiEdit on main branch — Claude must call EnterWorktree first; bypass with `ECC_WORKFLOW_BYPASS=1` (lazy worktree: created on first write, not session start)
- `session:end:worktree-merge` auto-merges worktree to main at session end via `ecc-workflow merge` (rebase + full verify + ff-only) — worktree directory is **not** deleted after merge (deferred to gc to avoid orphaning Claude Code's CWD); if merge fails, worktree preserved; retry with `ecc-workflow merge` or clean up with `ecc worktree gc`
- `session:start` triggers `ecc worktree gc` automatically to clean stale worktrees from previous sessions (best-effort, non-blocking)
- Claude Code's `EnterWorktree` prepends `worktree-` to branch names (e.g., `ecc-session-*` becomes `worktree-ecc-session-*`). ECC handles both forms — `WorktreeName::parse()` strips the prefix automatically
- Glossary: **write-guard** = PreToolUse hook blocking writes outside worktree (exit 2); **lazy worktree** = worktree created on-demand at first write; **session merge** = automatic rebase+verify+ff-merge at session end

## Development Notes

- Source is Rust, organized as a Cargo workspace with 9 crates
- Hexagonal architecture: domain → ports → infra → app → CLI
- All I/O is abstracted behind port traits, enabling full in-memory testing
- Agent/skill/hook format: Markdown with YAML frontmatter (see `agents/`, `skills/`, `hooks/`). Agent frontmatter includes `name`, `description`, `model`, `tools`, `effort`.
- File naming: lowercase with hyphens (e.g., `python-reviewer.md`, `tdd-workflow.md`)
