---
name: spec-pipeline-shared
description: Shared sections for spec-dev, spec-fix, and spec-refactor commands — project detection, adversarial review + verdict handling, and spec output schema.
origin: ECC
---

# Spec Pipeline Shared

Canonical shared content for all `/spec-*` commands. Each command references this skill instead of inlining duplicated sections.

## Project Detection

Detect the project's test, lint, and build commands using auto-detection:

- **Test**: `cargo test` (Rust) → `npm test` (Node) → `go test ./...` (Go) → `pytest` (Python)
- **Lint**: `cargo clippy -- -D warnings` → `npm run lint` → `golangci-lint run` → `ruff check .`
- **Build**: `cargo build` → `npm run build` → `go build ./...`

Persist detected commands to state.json via `toolchain-persist.sh`:

```
!bash .claude/hooks/toolchain-persist.sh "<test_cmd>" "<lint_cmd>" "<build_cmd>"
```

On re-entry, read toolchain from `state.json` instead of re-detecting.

> **Shared**: Use the grill-me skill for all interview phases. See `skills/grill-me/SKILL.md` for the universal protocol with spec-mode parameters.

## Adversarial Review + Verdict Handling

After spec output, launch a Task with the `spec-adversary` agent (allowedTools: [Read, Grep, Glob]):

- Pass the full spec from conversation context
- The agent attacks the spec on 7 dimensions: ambiguity, edge cases, scope, dependencies, testability, decisions, rollback

### Verdict Handling (max 3 rounds)

- **FAIL**: Return to Grill-Me to address issues. Re-output spec, re-run adversary. Increment round.
- **CONDITIONAL**: Add suggested ACs to spec. Re-run adversary. Increment round.
- **PASS**: Persist spec to `docs/specs/YYYY-MM-DD-<slug>/spec.md`. Run `!bash .claude/hooks/phase-transition.sh solution plan <spec_file_path>`.

After 3 FAIL rounds, offer user override or abandon.

### Persist Spec to File

1. Generate slug from feature description (lowercase, hyphens, max 40 chars)
2. Create directory `docs/specs/YYYY-MM-DD-<slug>/`
3. Write full spec to `spec.md` (append `## Revision` block if file exists)
4. Pass file path to phase-transition command

## Spec Output Schema

All spec commands output these mandatory sections:

`# Spec: <title>` → `## Problem Statement` → `## Research Summary` → `## Decisions Made` (table) → `## User Stories` (US-NNN with ACs) → `## Affected Modules` → `## Constraints` → `## Non-Requirements` → `## E2E Boundaries Affected` (table) → `## Doc Impact Assessment` (table) → `## Open Questions`

## Campaign Init

Create `campaign.md` in `.claude/workflow/campaign.md` at Phase 0 when toolchain detection completes. Use the schema from `skills/campaign-manifest/SKILL.md`. Initial state: `Status: in-progress`, empty tables, toolchain populated. When the spec directory is created after adversary PASS, move `campaign.md` there and update `artifacts.campaign_path` in state.json.

## Grill-Me Disk Persistence

After each grill-me question is answered, append the Q&A pair to campaign.md's `## Grill-Me Decisions` table. Track Source as "Recommended" (user accepted) or "User" (user overrode). This ensures interview decisions survive session interruption.

## Draft Spec Persistence

Before dispatching the adversary, write the draft spec to `spec-draft.md` in the campaign.md directory. After adversary PASS, rename/overwrite `spec-draft.md` to `spec.md` in the final spec directory. This eliminates the chicken-and-egg fragility where the adversary reviews context-only content.

## Adversary History Tracking

After each adversary review round, append a row to campaign.md's `## Adversary History` table with Round number, Phase (spec/design), Verdict (PASS/CONDITIONAL/FAIL), and Key Findings summary.

## Agent Output Tracking

After each agent task (requirements-analyst, architect, evolution-analyst, etc.) returns, append a summary row to campaign.md's `## Agent Outputs` table with Agent name, Phase, and one-line Summary.
