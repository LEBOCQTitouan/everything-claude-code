---
description: Web-based upgrade scan with Technology Radar output
allowed-tools: [Task, Read, Grep, Glob, WebSearch, Write, TodoWrite, AskUserQuestion]
---

# Web-Based Upgrade Audit

> **MANDATORY**: Follow every phase exactly. Narrate per `skills/narrative-conventions/SKILL.md`.

Scans web for upgrades, tools, patterns, and feature ideas. Produces Technology Radar in `docs/audits/`.

Scope: $ARGUMENTS (or all dimensions if none). **Read-only — DO NOT modify source code.**

> **Tracking**: TodoWrite checklist below. If unavailable, proceed without tracking.

TodoWrite: Phase 0-5 items.

## Arguments

- `--focus=<dims>` — comma-separated: `deps`, `arch`, `tools`, `ecosystem`, `competitors`, `user-requests`, `blogs`, `research`
- `--setup` — force guided setup

### Focus-to-Agent Mapping

| Focus | Agent(s) |
|-------|----------|
| `deps` | dep-scanner |
| `arch` | arch-scout |
| `tools` | tool-scout |
| `ecosystem` | ecosystem-scout |
| `competitors` | competitor-scout |
| `user-requests` | user-request-miner |
| `blogs` | blog-miner |
| `research` | research-scout |
| `features` | competitor-scout + user-request-miner + blog-miner + research-scout |

No `--focus` → all 8 agents.

## Phase 0: GUIDED SETUP

Check `docs/audits/audit-web-profile.yaml`:
- **Exists, no `--setup`**: Load silently. Flag stale dimensions.
- **Exists + `--setup`**: Regenerate interactively.
- **Absent**: Interactive setup: scan codebase characteristics, generate suggested profile (standard 8 + custom dims), present via AskUserQuestion, write profile.

Non-interactive fallback: default profile with standard dimensions.

## Phase 1: INVENTORY

Scan project root for dependency manifests (Cargo.toml, package.json, pyproject.toml, requirements.txt, go.mod). Extract direct deps + versions. Extract architectural patterns from CLAUDE.md/ARCHITECTURE.md.

**Dep cap**: >100 deps → group by category, cap queries at 50/category.

**Cost consent gate**: Display scope (deps, patterns, tool areas, estimated searches) via AskUserQuestion. If declined: exit.

## Phase 2: LANDSCAPE SCAN

Delegate to `web-scout` orchestrator with inventory context + focus filter.

8 parallel subagents (one per active dimension). `web-research-strategy` skill governs query formulation. Source triangulation: 3+ independent sources per finding. Concurrency cap: 4 simultaneous. WebSearch fallback: exa-web-search MCP. Agent failure: proceed with available results.

Each subagent returns condensed findings only.

## Phase 3: EVALUATE

Score each finding on 3 dimensions (0-5): **strategic fit**, **maturity**, **migration effort**.

### Quadrants

Techniques | Tools | Platforms | Languages & Frameworks | Feature Opportunities

### Ring Classification

| Ring | Criteria |
|------|---------|
| **Adopt** | fit≥4, maturity≥4, effort≤2 |
| **Trial** | Promising but not fully proven |
| **Assess** | Worth watching |
| **Hold** | Avoid for new work |

Low maturity (<2) → Assess or Hold. Merge duplicate findings.

## Phase 4: SYNTHESIZE

### 4a. Write Report

Write to `docs/audits/web-radar-YYYY-MM-DD.md`. Create dir if needed. Same-day rerun: `-r2` suffix.

Structure: Project Profile, then per-quadrant sections (Adopt/Trial/Assess/Hold findings with title, ring, scores, rationale, 3+ source citations). Close with Next Steps.

No findings → "No upgrade opportunities identified."

### 4b. Terminal Summary

Top 10 findings by impact/effort ratio (fit minus effort). Show report path + totals.

## Phase 5: SELF-IMPROVEMENT

### Coverage Gap Analysis
Check each quadrant/dimension for findings. Zero findings → suggest removing or refining.

### Suggest New Dimensions
Based on inventory + findings: suggest custom dimensions for uncovered characteristics.

### Threshold Adjustment
If distributions skewed, suggest adjustments.

Present suggestions via AskUserQuestion. Persist accepted to profile `improvement_history`. If profile modified, commit with report.

**STOP. DO NOT modify source code.** Say: "To act on findings, run `/backlog add` or `/spec`."

## Sources Re-interrogation

If `docs/sources.md` exists: `ecc sources check` for reachability, report stale (WARN >90d, ERROR >180d), update flags. Skip if absent.

## Related Agents

- `agents/web-scout.md` — parallel search orchestrator
- `agents/web-radar-analyst.md` — scoring per category
