---
name: prompt-optimizer
description: >-
  Analyze raw prompts, identify intent and gaps, match ECC components
  (skills/commands/agents/hooks), and output a ready-to-paste optimized
  prompt. Advisory role only — never executes the task itself.
  TRIGGER when: user says "optimize prompt", "improve my prompt",
  "how to write a prompt for", "help me prompt", "rewrite this prompt",
  or explicitly asks to enhance prompt quality. Also triggers on Chinese
  equivalents: "优化prompt", "改进prompt", "怎么写prompt", "帮我优化这个指令".
  DO NOT TRIGGER when: user wants the task executed directly, or says
  "just do it" / "直接做". DO NOT TRIGGER when user says "优化代码",
  "优化性能", "optimize performance", "optimize this code" — those are
  refactoring/performance tasks, not prompt optimization.
origin: community
metadata:
  author: YannJY02
  version: "1.0.0"
---

# Prompt Optimizer

**Advisory only — never execute the user's task.** Output analysis + optimized prompt only.

## Pipeline

### Phase 0: Project Detection

Detect tech stack from project files (`package.json`, `go.mod`, `Cargo.toml`, `pyproject.toml`, etc.). Read `CLAUDE.md` for conventions.

### Phase 1: Intent Detection

| Category | Signals |
|----------|---------|
| New Feature | build, create, add, implement |
| Bug Fix | fix, broken, error |
| Refactor | refactor, clean up, restructure |
| Research | how to, explore, investigate |
| Testing | test, coverage, verify |
| Review | review, audit, check |
| Documentation | document, update docs |
| Infrastructure | deploy, CI, docker, database |
| Design | design, architecture, plan |

### Phase 2: Scope Assessment

| Scope | Heuristic | Orchestration |
|-------|-----------|---------------|
| TRIVIAL | Single file, < 50 lines | Direct execution |
| LOW | Single component | Single command/skill |
| MEDIUM | Multiple components | Command chain + /verify |
| HIGH | Cross-domain, 5+ files | /spec first, phased |
| EPIC | Multi-session, architectural | Blueprint skill |

### Phase 2.5: Backlog Cross-Reference

If `docs/backlog/BACKLOG.md` exists, compare prompt against open entries. Report matches with confidence scores.

### Phase 3: ECC Component Matching

**By Intent:**

| Intent | Commands | Skills |
|--------|----------|--------|
| New Feature | /spec, /tdd, /verify | tdd-workflow |
| Bug Fix | /tdd, /build-fix, /verify | tdd-workflow |
| Refactor | /refactor-clean, /verify | verification-loop |
| Testing | /tdd, /e2e | tdd-workflow, e2e-testing |
| Review | /code-review | security-review |
| Design (EPIC) | - | blueprint |

**By Tech Stack:** Match detected stack to language-specific skills (django-*, golang-*, springboot-*, swiftui-*, cpp-*, rust-*, etc.).

### Phase 4: Missing Context Detection

Check for: tech stack, target scope, acceptance criteria, error handling, security, testing, performance, existing patterns, scope boundaries. If 3+ critical items missing, ask up to 3 clarification questions.

### Phase 5: Model Recommendation

| Scope | Model |
|-------|-------|
| TRIVIAL-MEDIUM | Sonnet 4.6 |
| HIGH | Sonnet 4.6 + Opus 4.6 (planning) |
| EPIC | Opus 4.6 (blueprint) + Sonnet 4.6 (execution) |

For HIGH/EPIC: split into sequential prompts (Research+Plan, Implement per phase, Integration test).

## Output Format

### Section 1: Prompt Diagnosis

**Strengths**, **Issues** (table: issue/impact/fix), **Needs Clarification** (questions or auto-detected answers).

### Section 2: Recommended ECC Components

Table: Type | Component | Purpose

### Section 3: Optimized Prompt (Full)

Complete self-contained prompt in fenced code block with: task, tech stack, /commands, acceptance criteria, scope boundaries.

### Section 4: Quick Version

| Intent | Pattern |
|--------|---------|
| Feature | `/spec [feature]. /tdd. /code-review. /verify.` |
| Bug Fix | `/tdd — failing test for [bug]. Fix. /verify.` |
| Refactor | `/refactor-clean [scope]. /code-review. /verify.` |
| EPIC | `blueprint skill for "[objective]". /verify gates.` |

### Section 5: Enhancement Rationale

Table: Enhancement | Reason

> Not what you need? Tell me what to adjust, or make a normal task request.

## Related Components

| Component | When to Reference |
|-----------|------------------|
| `configure-ecc` | User hasn't set up ECC |
| `skill-stocktake` | Audit installed components |
| `blueprint` | EPIC-scope prompts (skill, not command) |
