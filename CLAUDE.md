# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A collection of production-ready agents, skills, hooks, commands, rules, and MCP configurations for software development using Claude Code. Core CLI is implemented in Rust (Hexagonal Architecture, DDD, Clean Code).

## Running Tests

```bash
cargo test              # Run all tests (1190 tests)
cargo clippy -- -D warnings  # Lint with zero warnings
cargo build --release   # Build release binary
npm run lint            # Lint all Markdown files
```

## Architecture

```
crates/          Rust crates (hexagonal architecture)
  ecc-domain/    Pure business logic — zero I/O
  ecc-ports/     Trait definitions (FileSystem, ShellExecutor, Environment, TerminalIO)
  ecc-app/       Use cases — orchestrates domain + ports
  ecc-infra/     Adapters (OS filesystem, process executor, terminal)
  ecc-cli/       CLI entry point (`ecc` command)
  ecc-test-support/  Test doubles | ecc-integration-tests/  Integration tests
bin/             Shell shims | docs/  Guides & reference | examples/  CLAUDE.md templates
agents/          Subagents (architect, uncle-bob, planner, code-reviewer, spec-adversary, solution-adversary, drift-checker, ...)
commands/        Slash commands (audit-*, plan-*, solution, implement, verify, review, ...)
skills/          Domain knowledge | rules/  Always-follow guidelines
hooks/           Automations | contexts/  Prompt injection | mcp-configs/  MCP servers
```

## CLI Commands

```
ecc version          Show version
ecc install          Install ECC config to ~/.claude/
ecc init             Initialize ECC in current project
ecc audit            Audit ECC configuration health
ecc hook <id> [profiles]  Run a hook by ID
ecc validate <target>     Validate content files (agents|commands|hooks|skills|rules|paths)
ecc dev on|off|status     Toggle ECC config on/off
ecc claw                  NanoClaw interactive REPL
ecc completion <shell>    Generate shell completions
```

## Slash Commands

### Audit Commands

| Command | Domain |
|---------|--------|
| `/audit-full` | All domains — parallel run with cross-domain correlation |
| `/audit-archi` | Boundary integrity, dependency direction, DDD compliance |
| `/audit-code` | SOLID, clean code, naming, complexity |
| `/audit-convention` | Naming patterns, style consistency |
| `/audit-doc` | Coverage, staleness, drift |
| `/audit-errors` | Swallowed errors, taxonomy, boundary translation |
| `/audit-evolution` | Git hotspots, churn, bus factor, complexity trends |
| `/audit-observability` | Logging, metrics, tracing, health endpoints |
| `/audit-security` | OWASP top 10, secrets, attack surface |
| `/audit-test` | Coverage, classification, fixture ratios, E2E matrix |

### Spec-Driven Pipeline (Doc-First)

`/spec` → `/spec-dev`, `/spec-fix`, `/spec-refactor` → `/design` → `/implement`

- `/spec` auto-classifies intent (dev/fix/refactor) and delegates to the matching `/spec-*` command
- Each `/spec-*` runs web research, grill-me interview, enters **Plan Mode** for doc-first review (spec draft + upper-level doc preview), adversarial review, then outputs the spec
- `/design` produces the technical design, enters **Plan Mode** for architecture preview (arch docs, diagrams, bounded contexts), then adversarial review
- `/implement` enters **Plan Mode** for implementation steps, then executes TDD loops with TaskCreate tracking and mandatory doc updates
- All three phases use Plan Mode so the user reviews artifacts before execution
- Old names (`/plan`, `/solution`) still work as aliases
- State machine in `.claude/workflow/` enforces phase ordering via hooks

### Side Commands

| Command | Purpose |
|---------|---------|
| `/verify` | Build + tests + lint + code review + architecture review + drift check |
| `/review` | Robert professional conscience check |
| `/backlog` | Capture and manage implementation ideas |
| `/build-fix` | Fix build/type errors reactively |
| `/ecc-test-mode` | Isolated worktree for testing ECC config changes |

## Command Workflows

Slash command workflows defined in `commands/` are mandatory. Follow every phase and step exactly as specified — do not skip, reorder, or modify phases. The spec-driven pipeline is enforced by `.claude/workflow/state.json` and hook-based gates.

## Doc Hierarchy

`CLAUDE.md` (onboarding) → `docs/getting-started.md` (human setup) → `docs/ARCHITECTURE.md` (system design) → `docs/adr/` (decisions) → `docs/domain/bounded-contexts.md` (domain model) → `docs/runbooks/` (ops) → `docs/MODULE-SUMMARIES.md` (per-crate reference). Information lives at the lowest layer that serves its audience; CLAUDE.md stays terse.

## Dual-Mode Development

- **Spec-driven** (`/spec` or `/spec-*` → `/design` → `/implement`): for features, fixes, and refactors requiring design review. Old names (`/plan`, `/solution`) work as aliases.
- **Direct** (edit + `/verify`): for small, well-understood changes
- Use `/audit-*` independently at any time for health checks
- Use `/review` as a final craft conscience gate before shipping

## Gotchas

- `ecc-domain` crate must have zero I/O imports — pure business logic only (enforced by hook)
- Agent frontmatter `model` field controls which Claude model runs the agent — wrong value silently degrades quality
- `hooks.json` lives in `hooks/`, not the project root
- Skill directory name must match the `name` field in its frontmatter
- Test count in CLAUDE.md (currently 1180) must be updated after adding or removing tests
- ECC hooks are bypassed by default via `.envrc` (`ECC_WORKFLOW_BYPASS=1`) — to test the pipeline: `ECC_WORKFLOW_BYPASS=0 claude` or use `/ecc-test-mode`

## Development Notes

- Source is Rust, organized as a Cargo workspace with 7 crates
- Hexagonal architecture: domain → ports → infra → app → CLI
- All I/O is abstracted behind port traits, enabling full in-memory testing
- Agent/skill/hook format: Markdown with YAML frontmatter (see `agents/`, `skills/`, `hooks/`)
- File naming: lowercase with hyphens (e.g., `python-reviewer.md`, `tdd-workflow.md`)
