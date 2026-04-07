# Design: BL-124 Token Optimization Wave 1 — Zero-Cost CLI Redirects

## Spec Reference

`docs/specs/2026-04-06-token-cli-redirects/spec.md` — 5 US, 17 ACs.

## File Changes

| # | File(s) | Change | Spec Ref | Rationale |
|---|---------|--------|----------|-----------|
| 1 | `agents/doc-generator.md` | Step 4: replace inline `git log --format="%H\|%s\|%ai" --no-merges` with `ecc analyze changelog --since 6m`. Add error check: if exit non-zero, emit error and abort step. Remove "or run `git log`" fallback clause. | US-001 | Eliminate Haiku token waste on deterministic git parsing |
| 2 | `agents/evolution-analyst.md` | Step 2: replace `git log --since` with `ecc analyze hotspots --top N --since <window>d`. Step 5: replace manual commit→files co-change computation with `ecc analyze coupling --threshold 0.3 --since <window>d`. Step 6: replace `git shortlog -sn` with `ecc analyze bus-factor --top N --since <window>d`. Add error check per CLI call: if exit non-zero, report as `[EVOLUTION-ERR]` finding and continue. Steps 1/3/4/7 unchanged. | US-002 | Eliminate Opus token waste on deterministic git queries |
| 3 | `agents/backlog-curator.md` | Step 5 ("Check duplicates"): replace inline keyword/tag comparison logic with `ecc backlog check-duplicates <title> --tags <tags>`. If CLI returns empty output → "no duplicates found", proceed to Step 6. If CLI returns matches → present merge/replace/add-separately options. Add `"Bash"` to frontmatter `tools` list. | US-003 | Eliminate Sonnet token waste on deterministic scoring |
| 4 | 26 `commands/*.md` files | Replace each verbose `> **Narrative**: See \`skills/narrative-conventions/SKILL.md\` conventions. <trailing clause>` with `> **Narrative**: See narrative-conventions skill.` If the original line had a command-specific trailing clause beyond the standard "Before each..." pattern, preserve it on the next line as `> <trailing clause>`. | US-004 | Eliminate ~780 words of duplicated prose |
| 5 | 9 `commands/audit-*.md` files | Before the existing "Launch a Task with the `audit-challenger` agent" block, insert a conditional gate: `### Adversary Gate\n\nIf the aggregate finding count from the analysis phase is <3 AND all findings are MEDIUM severity or lower (threshold rationale: low-signal audits provide insufficient material for meaningful adversarial review — see BL-121 finding 4.5), skip the adversary challenge:\n\n> "Adversary challenge skipped: N findings, all ≤MEDIUM severity."\n\nOtherwise, proceed with the challenger launch as described below.` | US-005 | Eliminate subagent waste on clean/low-finding audits |
| 6 | `CLAUDE.md` | Add to Gotchas: `- CLI-redirected agents (doc-generator, evolution-analyst, backlog-curator) call \`ecc analyze\` and \`ecc backlog\` commands for raw data — agent still interprets results` | Doc Impact | Document behavioral change |
| 7 | `CHANGELOG.md` | Add BL-124 entry under latest version section | Doc Impact | Record change |

## Pass Conditions

| PC | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | content | doc-generator Step 4 references `ecc analyze changelog --since`, no inline git log, includes error handling | AC-001.1, AC-001.2, AC-001.3, AC-001.4 | `grep 'ecc analyze changelog' agents/doc-generator.md && grep '\-\-since' agents/doc-generator.md && ! grep 'git log --format' agents/doc-generator.md` | all three checks pass |
| PC-002 | content | evolution-analyst references 3 CLI commands, retains Steps 3/4/7, includes error handling | AC-002.1, AC-002.2, AC-002.3, AC-002.4 | `grep -c 'ecc analyze' agents/evolution-analyst.md` | count ≥3 |
| PC-003 | content | backlog-curator calls CLI + Bash in frontmatter + empty-output handling | AC-003.1, AC-003.2, AC-003.3, AC-003.4 | `grep 'check-duplicates' agents/backlog-curator.md && grep '"Bash"' agents/backlog-curator.md` | both match |
| PC-004 | content | All commands with narrative-conventions have normalized one-liner, trailing clauses preserved | AC-004.1, AC-004.2 | `grep -rl 'narrative-conventions' commands/*.md \| xargs grep -L 'See narrative-conventions skill' \| wc -l` | 0 (no un-normalized files remain) |
| PC-005 | content | 9 audit commands have conditional challenger gate | AC-005.1, AC-005.2, AC-005.3 | `grep -l 'Adversary Gate\|skip.*challenger\|≤MEDIUM' commands/audit-{archi,code,convention,doc,errors,evolution,observability,security,test}.md \| wc -l` | 9 |
| PC-006 | docs | CLAUDE.md gotchas + CHANGELOG entry | Doc Impact | `grep 'CLI-redirected' CLAUDE.md && grep 'BL-124' CHANGELOG.md` | both match |
| PC-007 | validation | All files pass structural validation | Constraints | `ecc validate agents && ecc validate commands` | exit 0 |

## Coverage Check

| AC | PC |
|----|----|
| AC-001.1 | PC-001 |
| AC-001.2 | PC-001 |
| AC-001.3 | PC-001 |
| AC-001.4 | PC-001 |
| AC-002.1 | PC-002 |
| AC-002.2 | PC-002 |
| AC-002.3 | PC-002 |
| AC-002.4 | PC-002 |
| AC-003.1 | PC-003 |
| AC-003.2 | PC-003 |
| AC-003.3 | PC-003 |
| AC-003.4 | PC-003 |
| AC-004.1 | PC-004 |
| AC-004.2 | PC-004 |
| AC-005.1 | PC-005 |
| AC-005.2 | PC-005 |
| AC-005.3 | PC-005 |

17/17 ACs covered.

## E2E Test Plan

No E2E tests needed — all changes are markdown instruction edits with no runtime code.

## E2E Activation Rules

None — no port/adapter boundaries touched.

## Test Strategy

1. Per-PC grep checks verify content was applied correctly
2. `ecc validate agents && ecc validate commands` verifies structural integrity
3. Manual review of diff confirms behavior preservation

## Doc Update Plan

| Doc File | Level | Action | Content Summary | Spec Ref |
|----------|-------|--------|-----------------|----------|
| CLAUDE.md | Gotchas | Add line | CLI-redirected agents note | US-001/002/003 |
| CHANGELOG.md | Entry | Add entry | BL-124 wave 1 changes summary | All US |

## SOLID Assessment

**PASS** — CLI delegation improves SRP (agents no longer embed deterministic git/scoring logic). Conditional gate is valid OCP extension. No LSP/ISP/DIP concerns (markdown files, not compiled code).

## Robert's Oath Check

**CLEAN** — with WARN: preserve command-specific trailing clauses in narrative normalization (addressed by AC-004.1). Add brief inline rationale for challenger threshold constant.

## Security Notes

**CLEAR** — no user input flows into CLI command strings. `ecc backlog check-duplicates` receives title/tags from the agent's own context, not raw user input. No injection surface identified.

## Rollback Plan

Each PC maps to one atomic commit. Any commit can be independently reverted via `git revert <sha>` with no cross-dependencies. PC-004 (26 files) and PC-005 (9 files) are higher merge-conflict surface but structurally safe to revert.

## Bounded Contexts Affected

None — content layer only (agent/command markdown). No domain, port, adapter, or infrastructure changes.
