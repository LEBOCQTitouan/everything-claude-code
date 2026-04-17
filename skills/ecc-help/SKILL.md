---
name: ecc-help
description: Quick reference for Everything Claude Code tooling — slash commands, pipeline flow, CLI commands, and key concepts. Load when users ask how ECC works or what commands are available.
origin: ECC
---

# ECC Help

## When to Activate

- User asks "what commands are available?" or "how does ECC work?"
- New session needs orientation on ECC tooling
- User references a slash command or CLI command by name

## Pipeline Overview

ECC follows a spec-driven pipeline: `/spec` → `/design` → `/implement`.

- **`/spec`** classifies intent (dev/fix/refactor), runs requirements analysis, architecture review, grill-me interview, adversarial review. Outputs `docs/specs/<date>-<slug>/spec.md`.
- **`/design`** produces file changes table, pass conditions, TDD order. Runs SOLID/security/oath reviews + adversarial review. Outputs `design.md`.
- **`/implement`** executes TDD loop via tdd-executor subagents (RED→GREEN→REFACTOR), code review, doc updates. Outputs `implement-done.md`.

All three phases use **Plan Mode** for user review before execution. **Worktree isolation** keeps each session's writes on a dedicated branch.

## Slash Commands

| Command | Purpose |
|---------|---------|
| `/spec` | Auto-classify and delegate to `/spec-dev`, `/spec-fix`, or `/spec-refactor` |
| `/design` | Technical design from spec — file changes, pass conditions, TDD order |
| `/implement` | TDD execution loop with subagent dispatch |
| `/verify` | Build + lint + test + code review + architecture review |
| `/audit-full` | Full codebase health audit (delegates to domain-specific `/audit-*`) |
| `/audit-*` | Domain audits: archi, code, doc, errors, evolution, observability, security, test |
| `/backlog` | Capture, list, promote, archive implementation ideas |
| `/party` | Multi-agent round-table discussion (BMAD-style) |
| `/commit` | Auto-generated conventional commit message |
| `/build-fix` | Incrementally fix build/type errors |
| `/review` | Robert professional conscience check |
| `/doc-suite` | Full documentation pipeline |
| `/catchup` | Session resumption summary |
| `/project-foundation` | Bootstrap project-level PRD + architecture docs |

## CLI Commands

```
ecc validate agents|commands|hooks|skills|rules|teams  Validate ECC components
ecc workflow init|transition|status|reset               Workflow state machine
ecc backlog next-id|reindex|list                        Backlog operations
ecc worktree gc|status                                  Worktree lifecycle
ecc status [--json]                                     Diagnostic snapshot
ecc drift check [--json]                                Spec-vs-implementation drift
ecc commit lint --staged                                Atomic commit validator
ecc dev on|off|switch                                   Toggle ECC config
```

## Key Concepts

- **Worktree isolation**: Each `/spec`→`/implement` session runs in a git worktree. Write-guard hook blocks edits on main.
- **Workflow state**: `ecc-workflow` binary manages `state.json` phases (idle→plan→solution→implement→done). Phase-gate hook enforces ordering.
- **Tool-set manifest**: `manifest/tool-manifest.yaml` declares atomic tools + named presets. Agents reference presets via `tool-set:` frontmatter; expanded at install time.
- **Party panel**: `/party` assembles multi-agent round-table from BMAD roles + ECC agents + project-domain agents for cross-perspective analysis.
- **Fix-round budget**: Max 2 fix attempts per pass condition before user escalation via AskUserQuestion.
- **Campaign file**: `campaign.md` in each spec dir tracks grill-me decisions and adversary history across sessions.
