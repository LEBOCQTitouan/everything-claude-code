# Everything Claude Code — Personal Fork

> Forked from [affaan-m/everything-claude-code](https://github.com/affaan-m/everything-claude-code) by [@affaanmustafa](https://x.com/affaanmustafa) — Anthropic Hackathon Winner.
> This fork is customized for personal use with Hexagonal Architecture, DDD, and Clean Code enforcement added on top of the original system.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

---

## What This Is

A Claude Code plugin — a collection of production-ready agents, skills, hooks, commands, and rules for software development. This fork adds an opinionated architecture layer on top of the upstream:

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

Works on Mac and Linux.

---

### Usage

#### Global install — sets up `~/.claude/` with agents, commands, skills, rules, hooks

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

#### Per-project setup — run from any repo

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

#### (Optional) Configure MCPs

Copy desired entries from `09-mcp-configs/mcp-servers.json` to your `~/.claude.json`. Replace `YOUR_*_HERE` placeholders with actual API keys.

---

## Repository Structure

```
everything-claude-code/
│
├── 03-agents/                       # Specialized subagents for delegation
│   ├── architect.md                 # ★ Hexagonal Architecture + DDD enforcer (system-level)
│   ├── architect-module.md          # ★ Module-level design within hexagonal boundaries
│   ├── uncle-bob.md                 # ★ Clean Architecture + Clean Code consultant
│   ├── planner.md                   # Feature planning, risk assessment, phase breakdown
│   ├── code-reviewer.md             # Security, quality, and Clean Code review
│   ├── tdd-guide.md                 # Test-driven development workflow
│   ├── security-reviewer.md         # OWASP / vulnerability analysis
│   ├── refactor-cleaner.md          # Dead code detection and safe removal
│   └── doc-updater.md               # Documentation sync
│
├── 04-commands/                     # Slash commands (/plan, /tdd, /code-review, ...)
│   ├── plan.md
│   ├── tdd.md
│   ├── code-review.md
│   ├── build-fix.md
│   ├── e2e.md
│   ├── refactor-clean.md
│   └── ...30+ more
│
├── 05-skills/                       # Domain knowledge invoked by agents or commands
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
├── 06-rules/                        # Always-follow guidelines (copy to ~/.claude/rules/)
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
├── 07-hooks/                        # Trigger-based automations
│   └── hooks.json                   # PreToolUse, PostToolUse, Stop, SessionStart events
│
├── 08-contexts/                     # Dynamic system prompt injection
│   ├── dev.md
│   ├── review.md
│   └── research.md
│
├── 09-mcp-configs/
│   └── mcp-servers.json             # GitHub, Supabase, Vercel, Railway, ...
│
├── 02-examples/                     # CLAUDE.md templates for real-world stacks
│   ├── saas-nextjs-CLAUDE.md
│   ├── go-microservice-CLAUDE.md
│   └── django-api-CLAUDE.md
│
├── 01-docs/                         # Documentation and reference material
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
├── 10-scripts/                      # Cross-platform Node.js hook implementations
│   ├── lib/
│   │   ├── utils.js
│   │   └── package-manager.js
│   └── hooks/
│       ├── session-start.js
│       ├── session-end.js
│       └── evaluate-session.js
│
└── 11-tests/                        # Test suite
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

## Key Commands

| Command | What it does | Agents involved |
|---|---|---|
| `/plan` | Implementation plan, risks, phases | planner |
| `/tdd` | Test-driven development workflow | tdd-guide |
| `/code-review` | Security + quality review | code-reviewer + uncle-bob |
| `/build-fix` | Fix build errors | build-error-resolver |
| `/e2e` | Generate + run E2E tests | e2e-runner |
| `/refactor-clean` | Remove dead code safely | refactor-cleaner |
| `/verify` | Run verification loop | — |
| `/learn` | Extract patterns from session | — |

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
    "command": "node scripts/hooks/post-edit.js"
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
node tests/run-all.js
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
