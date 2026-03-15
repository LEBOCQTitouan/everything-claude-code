# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A collection of production-ready agents, skills, hooks, commands, rules, and MCP configurations for software development using Claude Code. Core CLI is implemented in Rust (Hexagonal Architecture, DDD, Clean Code).

## Running Tests

```bash
# Run all tests (999 tests)
cargo test

# Run clippy
cargo clippy -- -D warnings

# Build release binary
cargo build --release
```

## Architecture

```
Cargo.toml       Rust workspace root
Cargo.lock       Dependency lock file
crates/          Rust crates (hexagonal architecture)
  ecc-domain/    Pure business logic — zero I/O
  ecc-ports/     Trait definitions (FileSystem, ShellExecutor, Environment, TerminalIO)
  ecc-app/       Application use cases — orchestrates domain + ports
  ecc-infra/     Production adapters (OS filesystem, process executor, terminal)
  ecc-cli/       CLI binary entry point (`ecc` command)
  ecc-test-support/  Test doubles (InMemoryFileSystem, MockExecutor, MockEnvironment)
bin/             Shell shims (ecc-hook, ecc-shell-hook.sh)
docs/            Diagrams, guides, and reference documentation
examples/        CLAUDE.md templates for real-world stacks
agents/          Specialized subagents (architect, uncle-bob, planner, code-reviewer, ...)
commands/        Slash commands (/plan, /build-fix, /verify, /e2e, /doc-suite, /audit)
skills/          Domain knowledge (tdd-workflow, security-review, backend-patterns, ...)
rules/           Always-follow guidelines (common/ + typescript/ + python/ + golang/)
hooks/           Trigger-based automations (hooks.json)
contexts/        Dynamic system prompt injection
mcp-configs/     MCP server configurations
schemas/         JSON schemas (hooks, package-manager)
```

## CLI Commands

```
ecc version          Show version
ecc install          Install ECC config to ~/.claude/
ecc init             Initialize ECC in current project
ecc audit            Audit ECC configuration health
ecc hook <id> [profiles]  Run a hook by ID
ecc validate <target>     Validate content files (agents|commands|hooks|skills|rules|paths)
ecc completion <shell>    Generate shell completions
```

## Slash Commands

6 commands cover the entire coding workflow:

| Command | Purpose |
|---------|---------|
| `/plan` | Plan, TDD (with commits per iteration), E2E if needed. Modes: `stories` (default), `refactor`, `security` |
| `/build-fix` | Fix build/type errors reactively |
| `/verify` | Build + tests + lint + code review + architecture review + coverage + dead code scan |
| `/e2e` | Generate and run E2E tests with Playwright |
| `/doc-suite` | Plan-first documentation pipeline |
| `/audit` | Codebase health audit |

## Scripts

| Command | Description |
|---------|-------------|
| `cargo test` | Run all 999 Rust tests |
| `cargo clippy -- -D warnings` | Lint with zero warnings |
| `cargo build --release` | Build release binary |
| `npm run lint` | Lint all Markdown files with markdownlint |

## Development Notes

- Source is Rust, organized as a Cargo workspace with 6 crates
- Hexagonal architecture: domain (pure logic) → ports (traits) → infra (adapters) → app (use cases) → CLI
- All I/O is abstracted behind port traits, enabling full in-memory testing
- Agent format: Markdown with YAML frontmatter (name, description, tools, model)
- Skill format: Markdown with clear sections for when to use, how it works, examples
- Hook format: JSON with matcher conditions and command/notification hooks
- File naming: lowercase with hyphens (e.g., `python-reviewer.md`, `tdd-workflow.md`)
