# Everything Claude Code — Personal Fork

> Forked from [affaan-m/everything-claude-code](https://github.com/affaan-m/everything-claude-code) by [@affaanmustafa](https://x.com/affaanmustafa) — Anthropic Hackathon Winner.
> This fork is customized for personal use with Hexagonal Architecture, DDD, and Clean Code enforcement added on top of the original system.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## What This Is

A collection of production-ready agents, skills, hooks, commands, and rules for software development with Claude Code. This fork adds an opinionated architecture layer on top of the upstream:

- **Hexagonal Architecture + DDD** enforced by a strategic architect agent
- **Clean Architecture + Clean Code** enforced by an Uncle Bob consultant agent
- **Module-level design** handled by a dedicated module architect agent

---

## Installation

### Requirements

- Node.js 18+
- Claude Code CLI v2.1.0+

```bash
npm install -g @lebocqtitouan/ecc
```

Works on Mac, Linux, and Windows (PowerShell).

---

## Shell Autocompletion

Supports bash, zsh, fish, and PowerShell.

**Auto-detect (recommended):** Run once after installing — omelette detects your shell and writes to the appropriate rc file:

```bash
ecc completion
```

**Per-shell manual setup:** Output the completion script for your shell and source it:

| Shell      | Setup command                                               | Reload                                    |
|------------|-------------------------------------------------------------|-------------------------------------------|
| zsh        | `eval "$(ecc completion zsh)"`  (add to `~/.zshrc`)        | `source ~/.zshrc`                         |
| bash       | `eval "$(ecc completion bash)"` (add to `~/.bashrc`)       | `source ~/.bashrc`                        |
| fish       | `ecc completion fish > ~/.config/fish/completions/ecc.fish` | `source ~/.config/fish/config.fish`       |
| PowerShell | `ecc completion pwsh \| Out-String \| Invoke-Expression`   (add to `$PROFILE`) | `. $PROFILE`   |

After reloading, `ecc <TAB>` completes commands, languages, and templates automatically.

---

## Usage

### Global install — sets up `~/.claude/` with agents, commands, skills, rules, hooks

```bash
ecc install typescript          # common + TypeScript rules
ecc install typescript python   # multiple stacks
```

| What | Where |
|---|---|
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

---

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
├── commands/                        # Slash commands (6 essential commands)
│   ├── plan.md                     # Plan → TDD → E2E (feature/refactor/security modes)
│   ├── build-fix.md                # Fix build/type errors
│   ├── verify.md                   # Build + tests + code review + arch review
│   ├── e2e.md                      # E2E test generation and execution
│   ├── doc-suite.md                # Full documentation pipeline
│   ├── audit.md                    # Codebase health audit (7 domains)
│   └── _archive/                   # 41 archived commands (still readable)
│
├── skills/                          # Domain knowledge invoked by agents or commands
│   ├── tdd-workflow/
│   ├── security-review/
│   ├── backend-patterns/
│   ├── frontend-patterns/
│   ├── continuous-learning/
│   ├── continuous-learning-v2/
│   ├── autonomous-loops/
│   ├── api-design/
│   ├── database-migrations/
│   ├── deployment-patterns/
│   ├── docker-patterns/
│   ├── e2e-testing/
│   ├── eval-harness/
│   ├── verification-loop/
│   ├── search-first/
│   ├── iterative-retrieval/
│   ├── strategic-compact/
│   ├── coding-standards/
│   ├── plankton-code-quality/
│   ├── security-scan/
│   ├── postgres-patterns/
│   ├── golang-patterns/ + golang-testing/
│   ├── python-patterns/ + python-testing/
│   ├── django-patterns/ + django-tdd/ + ...
│   ├── springboot-patterns/ + springboot-tdd/ + ...
│   ├── java-coding-standards/ + jpa-patterns/
│   ├── cpp-coding-standards/ + cpp-testing/
│   └── swift-*/swiftui-patterns/
│
├── rules/                           # Always-follow guidelines (copy to ~/.claude/rules/)
│   ├── common/                      # Language-agnostic — always install
│   │   ├── coding-style.md
│   │   ├── git-workflow.md
│   │   ├── testing.md
│   │   ├── security.md
│   │   └── agents.md
│   ├── typescript/
│   ├── python/
│   └── golang/
│
├── hooks/                           # Trigger-based automations
│   └── hooks.json                   # PreToolUse, PostToolUse, Stop, SessionStart events
│
├── contexts/                        # Dynamic system prompt injection
│   ├── dev.md
│   ├── review.md
│   └── research.md
│
├── mcp-configs/
│   └── mcp-servers.json             # GitHub, Supabase, Vercel, Railway, ...
│
├── examples/                        # CLAUDE.md templates for real-world stacks
│   ├── saas-nextjs-CLAUDE.md
│   ├── go-microservice-CLAUDE.md
│   └── django-api-CLAUDE.md
│
├── docs/                            # Documentation and reference material
│   ├── diagrams/                    # Architecture and flow diagrams
│   │   ├── agent-orchestration.md
│   │   ├── feature-development.md
│   │   ├── tdd-workflow.md
│   │   ├── security-review.md
│   │   └── refactoring.md
│   ├── shortform-guide.md           # Setup, foundations, philosophy (read first)
│   ├── longform-guide.md            # Token optimization, memory, evals, parallelization
│   ├── security-guide.md            # Security patterns and review
│   └── token-optimization.md
│
├── src/                             # TypeScript source
│   ├── lib/                         #   Core libraries (utils, package-manager, session-manager, ...)
│   ├── hooks/                       #   Hook script implementations (21 hooks)
│   ├── ci/                          #   CI validators (agents, commands, hooks, rules, skills)
│   ├── claw.ts                      #   NanoClaw REPL agent
│   ├── postinstall.ts               #   npm postinstall entry point
│   └── ...                          #   Other standalone scripts
│
├── dist/                            # Compiled JS output (npm run build)
│
└── tests/                           # Test suite (1401 tests, run with tsx)
    └── run-all.js
```

> ★ = added or heavily modified in this fork

---

## Agent Orchestration

See the full diagrams in [`docs/diagrams/`](docs/diagrams/):

| Diagram | Description |
|---|---|
| [agent-orchestration.md](docs/diagrams/agent-orchestration.md) | Full development flow and architecture agent chain |
| [feature-development.md](docs/diagrams/feature-development.md) | Feature lifecycle sequence diagram |
| [tdd-workflow.md](docs/diagrams/tdd-workflow.md) | TDD loop with uncle-bob quality gate |
| [security-review.md](docs/diagrams/security-review.md) | Code review split across security, quality, Clean Code |
| [refactoring.md](docs/diagrams/refactoring.md) | Safe refactoring loop with test verification |

### Agent Responsibilities

| Agent | Scope | Enforces |
|---|---|---|
| **architect** | System-wide | Hexagonal Architecture, DDD (bounded contexts, aggregates, ports) |
| **architect-module** | Single layer/module | Module internals, pattern selection, code efficiency |
| **uncle-bob** | Design + code | SOLID, Clean Architecture dependency rule, Clean Code |
| **planner** | Feature scope | Implementation phases, risk assessment |
| **code-reviewer** | Changed code | Security, quality, regressions |

---

## Commands

6 commands cover the entire coding workflow:

| Command | What it does | Agents involved |
|---|---|---|
| `/plan` | Plan → TDD → E2E. Modes: feature, refactor, security | planner, tdd-guide, architect, security-reviewer |
| `/build-fix` | Fix build/type errors reactively | build-error-resolver |
| `/verify` | Build + tests + lint + code review + arch review | code-reviewer, arch-reviewer, go/python-reviewer |
| `/e2e` | Generate + run E2E tests | e2e-runner |
| `/doc-suite` | Plan-first documentation pipeline (9 phases) | doc-orchestrator, doc-analyzer, doc-generator, doc-validator |
| `/audit` | Codebase health audit (7 domains) | audit-orchestrator, evolution-analyst, test-auditor, observability-auditor, error-handling-auditor, convention-auditor |

```
Got a feature?     →  /plan
Build broken?      →  /build-fix
Ready to ship?     →  /verify
Need E2E?          →  /e2e
Need docs?         →  /doc-suite
Need health check? →  /audit
```

41 archived commands are in `commands/_archive/` for reference. All agents and skills remain available.

**Distinction**: `/verify` = "Is this ready to ship?" (fast, change-scoped, pass/fail). `/audit` = "What is the long-term health?" (deep, codebase-wide, git-history-aware, report-generating).

---

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

Domain knowledge invoked by commands or agents:

```
skills/tdd-workflow/SKILL.md
skills/security-review/SKILL.md
skills/backend-patterns/SKILL.md
```

### Hooks

Automated triggers on tool events (`PreToolUse`, `PostToolUse`, `Stop`, `SessionStart`):

```json
{
  "matcher": "tool == \"Edit\"",
  "hooks": [{
    "type": "command",
    "command": "ecc-hook \"post:edit:format\" \"dist/hooks/post-edit-format.js\" \"standard,strict\""
  }]
}
```

### Rules

Always-follow guidelines, installed to `~/.claude/rules/`:

```
rules/common/          # Language-agnostic (always install)
rules/typescript/      # TS/JS specific
rules/python/          # Python specific
rules/golang/          # Go specific
```

---

## Running Tests

```bash
npm run build              # compile TypeScript to dist/
npx tsx tests/run-all.js   # run all 1401 tests
```

---

## Credits

Original project: **[affaan-m/everything-claude-code](https://github.com/affaan-m/everything-claude-code)** by [@affaanmustafa](https://x.com/affaanmustafa).
Built from an Anthropic Hackathon winner. 50K+ stars, 30+ contributors, 6 languages supported.

Guides from the original author:
- [Shorthand Guide](https://x.com/affaanmustafa/status/2012378465664745795) — setup, foundations, philosophy
- [Longform Guide](https://x.com/affaanmustafa/status/2014040193557471352) — token optimization, memory, evals, parallelization

---

## License

MIT
