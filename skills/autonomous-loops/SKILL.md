---
name: autonomous-loops
description: "Patterns and architectures for autonomous Claude Code loops — from simple sequential pipelines to RFC-driven multi-agent DAG systems."
origin: ECC
---

# Autonomous Loops Skill

> Compatibility note (v1.8.0): `autonomous-loops` is retained for one release.
> The canonical skill is now `continuous-agent-loop`.

## When to Use

- Setting up autonomous development workflows
- Choosing loop architecture (simple vs complex)
- Building CI/CD-style continuous development pipelines
- Running parallel agents with merge coordination

## Loop Pattern Spectrum

| Pattern | Complexity | Best For |
|---------|-----------|----------|
| [Sequential Pipeline](#1-sequential-pipeline) | Low | Daily dev steps, scripted workflows |
| [NanoClaw REPL](#2-nanoclaw-repl) | Low | Interactive persistent sessions |
| [Infinite Agentic Loop](#3-infinite-agentic-loop) | Medium | Parallel content generation |
| [Continuous Claude PR Loop](#4-continuous-claude-pr-loop) | Medium | Multi-day iterative projects with CI gates |
| [De-Sloppify Pattern](#5-de-sloppify-pattern) | Add-on | Quality cleanup after any Implementer step |
| [Ralphinho / RFC-Driven DAG](#6-ralphinho--rfc-driven-dag) | High | Large features, multi-unit parallel work |

---

## 1. Sequential Pipeline

Chain `claude -p` calls. Each is isolated with fresh context, building on filesystem state.

```bash
#!/bin/bash
set -e
claude -p "Read docs/auth-spec.md. Implement OAuth2 login with TDD."
claude -p "Review changes. Remove unnecessary type tests and over-defensive checks. Run test suite."
claude -p "Run build, lint, type check, tests. Fix failures."
claude -p "Create conventional commit: 'feat: add OAuth2 login flow'"
```

**Variations**: Model routing (`--model opus` for research, default for implementation), `--allowedTools` restrictions (read-only analysis pass), context via files instead of prompts.

---

## 2. NanoClaw REPL

Session-aware REPL with conversation history persistence.

```bash
node scripts/claw.js
CLAW_SESSION=my-project CLAW_SKILLS=tdd-workflow node scripts/claw.js
```

Loads/saves history from `~/.claude/claw/{session}.md`. Use for interactive exploration; use sequential pipeline for scripted automation.

---

## 3. Infinite Agentic Loop

Two-prompt system: Orchestrator reads spec, assigns unique creative directions + iteration numbers to N parallel sub-agents. Credit: @disler.

**Key insight**: Don't rely on agents to self-differentiate. The orchestrator **assigns** each agent a specific creative direction and iteration number.

| Count | Strategy |
|-------|----------|
| 1-5 | All simultaneously |
| 6-20 | Batches of 5 |
| infinite | Waves of 3-5, progressive sophistication |

---

## 4. Continuous Claude PR Loop

Production shell script: create branch, run claude, commit, push PR, wait for CI, auto-fix failures, merge, repeat. Credit: @AnandChowdhary.

```bash
continuous-claude --prompt "Add unit tests" --max-runs 10
continuous-claude --prompt "Fix linter errors" --max-cost 5.00
continuous-claude --prompt "Add auth" --max-runs 10 --review-prompt "Run npm test && npm run lint"
```

**Cross-iteration context**: `SHARED_TASK_NOTES.md` persists progress across iterations.

**CI failure recovery**: Fetches failed run ID, spawns fix agent with CI logs, re-waits for checks.

**Completion signal**: Agent outputs magic phrase N consecutive times to stop.

| Flag | Purpose |
|------|---------|
| `--max-runs N` | Stop after N iterations |
| `--max-cost $X` | Stop after spending $X |
| `--max-duration 2h` | Stop after time elapsed |
| `--merge-strategy squash` | squash/merge/rebase |
| `--worktree <name>` | Parallel via git worktrees |
| `--review-prompt "..."` | Reviewer pass per iteration |

---

## 5. De-Sloppify Pattern

Add a separate cleanup step after each implementation step. Two focused agents outperform one constrained agent.

```bash
claude -p "Implement feature with full TDD."
claude -p "Review changes. Remove: tests verifying language behavior, redundant type checks, over-defensive error handling, console.log, commented-out code. Run tests after cleanup."
```

Don't add negative instructions to the Implementer (degrades quality unpredictably). Use a separate pass instead.

---

## 6. Ralphinho / RFC-Driven DAG

RFC decomposition into dependency DAG, tiered quality pipelines per unit, merge queue with eviction. Credit: @enitrat.

```
RFC → Decomposition → DAG Layers (parallel per layer) → Merge Queue
```

### Complexity Tiers

| Tier | Pipeline |
|------|----------|
| trivial | implement, test |
| small | implement, test, code-review |
| medium | research, plan, implement, test, PRD-review, code-review, review-fix |
| large | + final-review |

### Key Design Principles
- Each stage in separate context window (eliminates author-bias in review)
- Merge queue: rebase onto main, run tests, land or evict with full context
- Non-overlapping units land in parallel; overlapping land sequentially
- Evicted units re-enter with conflict context
- Full state persisted to SQLite for resumability

---

## Choosing the Right Pattern

```
Single focused change? → Sequential Pipeline or NanoClaw
Written spec/RFC?
  Need parallel implementation? → Ralphinho
  No parallel needed? → Continuous Claude
Many variations of same thing? → Infinite Agentic Loop
Otherwise → Sequential Pipeline + De-Sloppify
```

## Anti-Patterns

1. Infinite loops without exit conditions (always have max-runs/cost/duration)
2. No context bridge between iterations (use SHARED_TASK_NOTES.md)
3. Retrying same failure without error context
4. Negative instructions instead of cleanup passes
5. All agents in one context window
6. Ignoring file overlap in parallel work

## References

| Project | Author |
|---------|--------|
| Ralphinho | @enitrat |
| Infinite Agentic Loop | @disler |
| Continuous Claude | @AnandChowdhary |
| NanoClaw | ECC (`/claw`) |
