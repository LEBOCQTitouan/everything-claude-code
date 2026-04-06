# Spec: BL-124 Token Optimization Wave 1 — Zero-Cost CLI Redirects

## Problem Statement

Three agents (doc-generator, evolution-analyst, backlog-curator) reimplement logic that already exists as compiled Rust CLI commands (`ecc analyze changelog`, `ecc analyze hotspots/coupling/bus-factor`, `ecc backlog check-duplicates`). Each redundant agent invocation wastes Opus/Sonnet/Haiku tokens on purely deterministic work. Additionally, 26 command files carry verbose narrative-conventions references (~30 words each, ~780 words total) that inflate context unnecessarily. Nine audit commands unconditionally launch audit-challenger subagents even for clean/low-finding runs, wasting context on adversarial review with nothing substantive to challenge.

## Research Summary

- Agent-to-CLI delegation eliminates hosted API tokens for deterministic work with zero reasoning requirement
- Conditional subagent launch is a standard cost optimization — skip adversarial review when there's nothing substantive to challenge
- Boilerplate normalization across 26 files is mechanical find-replace with no behavioral risk
- The caveman research (BL-123) demonstrated ~65% output token reduction through linguistic constraint; this spec targets input token reduction via instruction compression
- Web research skipped: LOW-scope internal markdown refactoring with no external technology decisions

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | Drop finding 4.6 (spec-dev parallel agents) | Architect depends on requirements-analyst output — not parallelizable | No |
| 2 | Evolution-analyst: redirect 3 raw metrics, keep composite scoring | CLI provides raw data; agent retains composite hotspot × complexity score | No |
| 3 | 5 independent atomic commits | Each change is self-contained, no cross-dependencies | No |
| 4 | Accept conditional challenger trade-off | <3 findings all ≤MEDIUM provide insufficient adversarial value | No |
| 5 | Add Bash to backlog-curator tools | CLI redirect requires Bash tool access; frontmatter constraint amended | No |

## User Stories

### US-001: Redirect doc-generator changelog to CLI

**As a** pipeline operator, **I want** doc-generator to call `ecc analyze changelog` instead of reimplementing git log parsing, **so that** Haiku tokens are saved on deterministic work.

#### Acceptance Criteria

- AC-001.1: Given doc-generator Step 4, when changelog is requested, then the agent calls `ecc analyze changelog` via Bash tool and incorporates CLI output (replacing inline git log, not as fallback — CLI is the primary path)
- AC-001.2: Given the changelog-gen skill reference, when Step 4 runs, then inline `git log --format` commands are removed from the agent file
- AC-001.3: Given `ecc analyze changelog` exits non-zero, then doc-generator emits an error message and aborts the changelog step (no silent empty output)
- AC-001.4: Given `ecc analyze changelog` is called, then the invocation includes `--since 6m` to match the current 6-month limit

#### Dependencies

- Depends on: none

### US-002: Redirect evolution-analyst to CLI for raw metrics

**As a** pipeline operator, **I want** evolution-analyst to call `ecc analyze hotspots`, `ecc analyze coupling`, and `ecc analyze bus-factor` for raw data, **so that** Opus tokens are saved on deterministic git queries.

#### Acceptance Criteria

- AC-002.1: Given evolution-analyst Step 2 (Change Frequency), then the agent calls `ecc analyze hotspots --top N --since <window>d` and parses the output for raw change counts per file. Step 5 (Co-Change Coupling) calls `ecc analyze coupling --threshold 0.3 --since <window>d`. Step 6 (Bus Factor) calls `ecc analyze bus-factor --top N --since <window>d`.
- AC-002.2: Given Steps 3/4/7 (complexity approximation, hotspot scoring, complexity trends), when composite scoring is needed, then the agent retains its own reasoning over CLI output
- AC-002.3: Given the agent's existing Step 1 (codebase size detection), when analysis begins, then Step 1 remains unchanged (no CLI equivalent)
- AC-002.4: Given any `ecc analyze` CLI call exits non-zero, then evolution-analyst reports the failure as a finding with location "CLI/environment" and continues with remaining steps

#### Dependencies

- Depends on: none

### US-003: Redirect backlog-curator duplicate check to CLI

**As a** backlog operator, **I want** backlog-curator to call `ecc backlog check-duplicates` instead of reimplementing keyword/tag scoring, **so that** Sonnet tokens are saved on deterministic comparison.

#### Acceptance Criteria

- AC-003.1: Given backlog-curator Step 5, when duplicate check runs, then the agent calls `ecc backlog check-duplicates <title> --tags <tags>` via Bash tool
- AC-003.2: Given CLI returns matches, when the agent processes results, then it still presents merge/replace/add-separately options to the user via AskUserQuestion
- AC-003.3: Given `ecc backlog check-duplicates` returns empty output (no matches), then backlog-curator treats this as "no duplicates found" and proceeds to Step 6
- AC-003.4: Given the backlog-curator agent requires Bash tool access for CLI calls, then the agent frontmatter `tools` list is updated to include "Bash"

#### Dependencies

- Depends on: none

### US-004: Standardize narrative-conventions one-liner

**As a** context window optimizer, **I want** all 26 command files to use a consistent short-form narrative-conventions reference, **so that** ~780 words of repeated prose are eliminated.

#### Acceptance Criteria

- AC-004.1: Given 26 command files with verbose narrative-conventions references, when standardized, then each file's first narrative line reads: `> **Narrative**: See narrative-conventions skill.` — if the original line contained a command-specific trailing behavioral clause (e.g., audit-archi's "After findings are produced, tell the user how to reference this report"), that clause is preserved as an addendum on the next line
- AC-004.2: Given the standardization, when any command is invoked, then behavior is unchanged — both the skill reference and any command-specific behavioral instructions are preserved

#### Dependencies

- Depends on: none

### US-005: Make audit-challenger conditional

**As a** audit pipeline operator, **I want** audit-challenger to launch only when the primary agent returns ≥3 findings or any finding with severity HIGH+, **so that** subagent context is not wasted on clean/low-finding audits.

#### Acceptance Criteria

- AC-005.1: Given 9 audit commands (archi, code, evolution, test, errors, security, doc, observability, convention), when the aggregate finding count across all analysis agents in that command is <3 AND all findings are MEDIUM or lower, then audit-challenger is skipped with message: "Adversary challenge skipped: N findings, all ≤MEDIUM severity"
- AC-005.2: Given the same 9 commands, when the primary agent returns ≥3 findings OR any HIGH/CRITICAL finding, then audit-challenger launches as before (no behavioral change)
- AC-005.3: Given audit-full.md, when it orchestrates individual audits, then the conditional logic applies per-domain (not at the orchestrator level)

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| agents/doc-generator.md | agent | Replace Step 4 inline git with CLI call |
| agents/evolution-analyst.md | agent | Replace Steps 2/5/6 inline git with CLI calls |
| agents/backlog-curator.md | agent | Replace Step 5 inline scoring with CLI call |
| commands/audit-archi.md | command | Add conditional gate before audit-challenger |
| commands/audit-code.md | command | Add conditional gate before audit-challenger |
| commands/audit-convention.md | command | Add conditional gate before audit-challenger |
| commands/audit-doc.md | command | Add conditional gate before audit-challenger |
| commands/audit-errors.md | command | Add conditional gate before audit-challenger |
| commands/audit-evolution.md | command | Add conditional gate before audit-challenger |
| commands/audit-observability.md | command | Add conditional gate before audit-challenger |
| commands/audit-security.md | command | Add conditional gate before audit-challenger |
| commands/audit-test.md | command | Add conditional gate before audit-challenger |
| commands/*.md (26 files) | command | Normalize narrative-conventions reference |

## Constraints

- All changes are behavior-preserving — no functional change to agent/command output
- Each user story ships as an independent atomic commit
- Validation: `ecc validate agents commands` must pass after each change
- No changes to agent frontmatter except backlog-curator (which needs Bash added to tools for CLI access)

## Non-Requirements

- TodoWrite boilerplate extraction (deferred to BL-125)
- CLAUDE.md trimming (deferred to BL-125)
- New CLI commands for drift-checker, module-summary-updater, etc. (deferred to BL-126)
- Spec-dev parallel agents (dropped — false positive from BL-121 audit)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Agent markdown | Instruction edit | None — markdown files, not runtime code |
| Command markdown | Instruction edit | None — markdown files, not runtime code |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CLI redirect agents | CLAUDE.md | Gotchas | Add note about CLI-redirected agents |
| BL-124 completion | CHANGELOG.md | Entry | Add BL-124 changelog entry |

## Open Questions

None — all resolved during grill-me interview.
