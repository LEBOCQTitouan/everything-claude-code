# Getting Started

## Requirements

- Claude Code CLI v2.1.0+
- macOS or Linux (x64 / arm64)

## Installation

```bash
curl -fsSL https://raw.githubusercontent.com/LEBOCQTitouan/everything-claude-code/main/scripts/get-ecc.sh | bash
```

This downloads the prebuilt Rust binary to `~/.ecc/bin/ecc` and adds it to your PATH.

## Shell Autocompletion

Supports bash, zsh, fish, and PowerShell.

| Shell | Setup command                                               | Reload                              |
|-------|-------------------------------------------------------------|-------------------------------------|
| zsh   | `eval "$(ecc completion zsh)"`  (add to `~/.zshrc`)        | `source ~/.zshrc`                   |
| bash  | `eval "$(ecc completion bash)"` (add to `~/.bashrc`)       | `source ~/.bashrc`                  |
| fish  | `ecc completion fish > ~/.config/fish/completions/ecc.fish` | `source ~/.config/fish/config.fish` |

After reloading, `ecc <TAB>` completes commands, languages, and templates automatically.

## Usage

### Global install — sets up `~/.claude/` with agents, commands, skills, rules, hooks

```bash
ecc install typescript          # common + TypeScript rules
ecc install typescript python   # multiple stacks
```

| What | Where |
|------|-------|
| Agents | `~/.claude/agents/` |
| Commands | `~/.claude/commands/` |
| Skills | `~/.claude/skills/` |
| Rules (common + language) | `~/.claude/rules/` |
| Hooks | merged into `~/.claude/settings.json` |

### Per-project setup — run from any repo

```bash
cd /your/project
ecc init                                        # auto-detect language & template
ecc init golang                                 # specify language
ecc init --template go-microservice golang
```

Creates:
- `CLAUDE.md` — project instructions, pre-filled from the nearest matching template
- `.claude/settings.json` — project-local hooks merged non-destructively

Available templates: `saas-nextjs`, `go-microservice`, `django-api`, `rust-api`

### (Optional) Configure MCPs

Copy desired entries from `mcp-configs/mcp-servers.json` to your `~/.claude.json`. Replace `YOUR_*_HERE` placeholders with actual API keys.

## Repository Structure

```
everything-claude-code/
│
├── agents/                          # Specialized subagents for delegation
│   ├── architect.md                 # ★ Hexagonal Architecture + DDD enforcer (system-level)
│   ├── architect-module.md          # ★ Module-level design within hexagonal boundaries
│   ├── uncle-bob.md                 # ★ Clean Architecture + Clean Code consultant
│   ├── planner.md                   # Feature planning, risk assessment, phase breakdown
│   ├── code-reviewer.md             # Security, quality, and Clean Code review
│   ├── tdd-guide.md                 # Test-driven development workflow
│   ├── security-reviewer.md         # OWASP / vulnerability analysis
│   ├── refactor-cleaner.md          # Dead code detection and safe removal
│   ├── doc-updater.md               # Documentation sync
│   ├── audit-orchestrator.md        # Codebase health audit orchestrator
│   ├── evolution-analyst.md         # Git history mining (hotspots, bus factor)
│   ├── test-auditor.md              # Test architecture quality analyst
│   ├── observability-auditor.md     # Logging/monitoring consistency audit
│   ├── error-handling-auditor.md    # Error handling architecture analyst
│   └── convention-auditor.md        # Naming/pattern consistency analyst
│
├── commands/                        # Slash commands
│   ├── audit-full.md               # All domains — parallel run with cross-domain correlation
│   ├── audit-archi.md              # Boundary integrity, dependency direction, DDD compliance
│   ├── audit-code.md               # SOLID, clean code, naming, complexity
│   ├── audit-convention.md         # Naming patterns, style consistency
│   ├── audit-doc.md                # Coverage, staleness, drift
│   ├── audit-errors.md             # Swallowed errors, taxonomy, boundary translation
│   ├── audit-evolution.md          # Git hotspots, churn, bus factor, complexity trends
│   ├── audit-observability.md      # Logging, metrics, tracing, health endpoints
│   ├── audit-security.md           # OWASP top 10, secrets, attack surface
│   ├── audit-test.md               # Coverage, classification, fixture ratios, E2E matrix
│   ├── spec-dev.md / spec-fix.md / spec-refactor.md  # Spec-driven planning
│   ├── design.md                   # Technical approach from spec
│   ├── implement.md                # TDD execution with doc updates
│   ├── verify.md                   # Build + tests + lint + review gate
│   ├── review.md                   # Robert professional conscience check
│   ├── build-fix.md                # Fix build/type errors reactively
│   └── backlog.md                  # Capture and manage implementation ideas
│
├── skills/                          # Domain knowledge invoked by agents or commands
│   ├── tdd-workflow/
│   ├── security-review/
│   ├── backend-patterns/
│   ├── frontend-patterns/
│   └── ...                          # 110+ domain skills
│
├── rules/                           # Always-follow guidelines (copy to ~/.claude/rules/)
│   ├── common/                      # Language-agnostic — always install
│   ├── typescript/ / python/ / golang/ / rust/
│
├── hooks/                           # Trigger-based automations
│   └── hooks.json                   # PreToolUse, PostToolUse, Stop events
│
├── contexts/                        # Dynamic system prompt injection
├── mcp-configs/                     # MCP server configurations
├── examples/                        # CLAUDE.md templates for real-world stacks
├── docs/                            # Documentation and reference material
│
├── crates/                          # Rust workspace (hexagonal architecture)
│   ├── ecc-domain/                  #   Pure business logic — zero I/O
│   ├── ecc-ports/                   #   Trait definitions (FileSystem, ShellExecutor, etc.)
│   ├── ecc-app/                     #   Application use cases
│   ├── ecc-infra/                   #   Production adapters
│   ├── ecc-cli/                     #   CLI binary entry point
│   ├── ecc-workflow/                #   Standalone binary — workflow state management
│   ├── ecc-flock/                   #   Shared POSIX flock utility
│   ├── ecc-test-support/            #   Test doubles (InMemoryFileSystem, MockExecutor)
│   └── ecc-integration-tests/       #   Binary-spawning integration tests
│
└── scripts/                         # Install/uninstall scripts
    └── get-ecc.sh                   #   curl installer
```

> ★ = added or heavily modified in this fork

## Agent Orchestration

See the full diagrams in [`docs/diagrams/`](docs/diagrams/):

| Diagram | Description |
|---------|-------------|
| [agent-orchestration.md](diagrams/agent-orchestration.md) | Full development flow and architecture agent chain |
| [feature-development.md](diagrams/feature-development.md) | Feature lifecycle sequence diagram |
| [tdd-workflow.md](diagrams/tdd-workflow.md) | TDD loop with uncle-bob quality gate |
| [security-review.md](diagrams/security-review.md) | Code review split across security, quality, Clean Code |
| [refactoring.md](diagrams/refactoring.md) | Safe refactoring loop with test verification |

### Agent Responsibilities

| Agent | Scope | Enforces |
|-------|-------|----------|
| **architect** | System-wide | Hexagonal Architecture, DDD (bounded contexts, aggregates, ports) |
| **architect-module** | Single layer/module | Module internals, pattern selection, code efficiency |
| **uncle-bob** | Design + code | SOLID, Clean Architecture dependency rule, Clean Code |
| **planner** | Feature scope | Implementation phases, risk assessment |
| **code-reviewer** | Changed code | Security, quality, regressions |

## Key Concepts

### Agents

Subagents handle delegated tasks with limited scope. Defined as Markdown with YAML frontmatter:

```markdown
---
name: architect
description: Strategic architect enforcing Hexagonal Architecture and DDD...
tools: ["Read", "Grep", "Glob", "Agent"]
model: opus
---
```

### Skills

Domain knowledge invoked by commands or agents. Each skill lives in its own directory with a `SKILL.md` file.

### Hooks

Automated triggers on tool events (`PreToolUse`, `PostToolUse`, `Stop`). Defined in `hooks/hooks.json`.

### Rules

Always-follow guidelines, installed to `~/.claude/rules/`. Organized by language with a common set that always applies.

## Commands

### Spec-Driven Pipeline

`/spec-dev`, `/spec-fix`, `/spec-refactor` -> `/design` -> `/implement`

Each `/spec-*` runs a grill-me interview, then writes `.claude/workflow/plan.md`. `/design` designs the technical approach. `/implement` executes TDD loops with mandatory doc updates.

### Audit Commands

`/audit-full` runs all domains in parallel. Individual audits: `/audit-archi`, `/audit-code`, `/audit-convention`, `/audit-doc`, `/audit-errors`, `/audit-evolution`, `/audit-observability`, `/audit-security`, `/audit-test`.

### Side Commands

| Command | Purpose |
|---------|---------|
| `/verify` | Build + tests + lint + code review + architecture review |
| `/review` | Robert professional conscience check |
| `/build-fix` | Fix build/type errors reactively |
| `/backlog` | Capture and manage implementation ideas |

**Distinction**: `/verify` = "Is this ready to ship?" (fast, change-scoped, pass/fail). `/audit-*` = "What is the long-term health?" (deep, codebase-wide, git-history-aware, report-generating).

## Running Tests

```bash
cargo test                    # run all Rust tests
cargo clippy -- -D warnings   # lint with zero warnings
cargo build --release         # build release binary
```

## Build Acceleration

Optional tools to speed up Rust compilation during development. None are required — ECC builds without them.

### sccache (11-14% faster repeated builds)

[sccache](https://github.com/mozilla/sccache) caches compilation artifacts so unchanged crates skip recompilation across clean builds.

```bash
# Install
cargo install sccache          # or: brew install sccache (macOS)

# Enable per-shell session
export RUSTC_WRAPPER=sccache

# Or persist in your user-level cargo config (~/.cargo/config.toml):
# [build]
# rustc-wrapper = "sccache"

# Verify it's working
sccache --show-stats
```

Expected speedup: **11-14%** on repeated test builds (source: web-radar-2026-03-29).

### mold linker (Linux only)

[mold](https://github.com/rui314/mold) is a drop-in replacement linker that is significantly faster than the default `ld`. ECC's `.cargo/config.toml` already configures mold for Linux targets — just install it:

```bash
# Ubuntu/Debian
sudo apt install mold

# Fedora
sudo dnf install mold
```

On macOS, the mold config sections are ignored (target-specific to `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`).

### Cranelift backend (30% faster compile, slower runtime)

The Cranelift codegen backend trades runtime performance for significantly faster compilation. Useful for rapid iteration during development, but **not recommended for benchmarks or release builds**.

```toml
# Add to your ~/.cargo/config.toml (NOT the project config):
# [unstable]
# codegen-backend = true
#
# [profile.dev]
# codegen-backend = "cranelift"
```

Expected speedup: **30%** faster compilation (source: web-radar-2026-03-29). Requires nightly Rust (`rustup default nightly`).
