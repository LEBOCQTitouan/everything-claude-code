# Spec: BL-084 /audit-backlog Conformance Command

## Problem Statement

ECC has 57+ implemented backlog entries with no mechanism to verify implementations match original intent. Acceptance criteria may have drifted, features may be partially implemented, tests may not cover requirements, and some entries marked `open` may actually be implemented (shadow implementations). A systematic conformance audit is needed to catch drift, detect shadow implementations, and generate actionable remediation reports.

## Research Summary

- Web research skipped: no search tool available
- 11 existing `/audit-*` commands follow consistent pattern: frontmatter, analysis phase, report phase, present phase
- Backlog entries have YAML frontmatter (id, title, status, scope, tags) + markdown body
- Early entries (BL-001 to BL-009) lack formal ACs -- need graceful degradation
- `promoted` status unused (0 entries) but kept as no-cost filter
- `/audit-full` orchestrates domains via `audit-orchestrator` agent

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Command-only (no dedicated agent) | Sequential logic, simpler audit commands use this pattern | No |
| 2 | --run-tests on by default | User preference for thoroughness over speed | No |
| 3 | Three-tier shadow confidence (HIGH/MEDIUM/LOW) | Prevents false positives while still flagging candidates | No |
| 4 | Early entries use "inferred" verdicts | BL-001-009 have no formal ACs but artifacts can be verified | No |

## User Stories

### US-001: Standalone /audit-backlog Command

**As a** developer, **I want** `/audit-backlog` to scan all implemented entries and verify conformance, **so that** I can detect drift without manual review.

#### Acceptance Criteria

- AC-001.1: Given the command is invoked, when entries have status `implemented`, then each is audited with verdict PASS/PARTIAL/FAIL/MISSING
- AC-001.2: Given an entry references files/commands/functions, when artifacts exist, then verdict is PASS; partially present = PARTIAL; absent = FAIL
- AC-001.3: Given an entry has status `open`, when git log contains BL-NNN commits or artifacts exist, then flagged as shadow implementation with confidence HIGH/MEDIUM/LOW
- AC-001.4: Given early entries (BL-001-009) with no formal ACs, when described artifact exists, then verdict is PASS with note "inferred from description"
- AC-001.5: Given `--run-tests` (default on), when an entry references a crate, then `cargo test -p <crate>` is executed and results included in verdict
- AC-001.6: Given `--no-run-tests` flag, when provided, then test execution is skipped (artifact-only verification)
- AC-001.7: Given the command follows /audit-* conventions, when inspected, then it has standard frontmatter, mandatory workflow banner, narrative references, and "STOP. DO NOT modify source code" footer

#### Dependencies

- Depends on: none

### US-002: Conformance Report Format

**As a** developer, **I want** a structured report at `docs/audits/backlog-conformance-YYYY-MM-DD.md`, **so that** I can act on gaps systematically.

#### Acceptance Criteria

- AC-002.1: Given the report, when generated, then it includes: per-entry table (ID, Title, Status, Verdict, Evidence, Gaps), Summary Statistics, Shadow Implementations, Remediation Suggestions
- AC-002.2: Given an entry with PARTIAL/FAIL verdict, when report generated, then specific remediation is included
- AC-002.3: Given report completes, when displayed, then console summary shows conformance rate, verdict distribution, shadow count, report path
- AC-002.4: Given terminology, when report header inspected, then it defines: conformance, shadow implementation, verdict tiers, confidence tiers

#### Dependencies

- Depends on: US-001

### US-003: /audit-full Integration

**As a** developer, **I want** backlog conformance as a domain in `/audit-full`, **so that** conformance drift is caught during routine audits.

#### Acceptance Criteria

- AC-003.1: Given `/audit-full` is invoked, when backlog domain runs, then conformance audit executes as a parallel domain
- AC-003.2: Given `--domain=backlog`, when `/audit-full` runs, then only backlog conformance executes
- AC-003.3: Given backlog findings exist, when cross-domain correlation runs, then FAIL entries overlapping with other domain findings are escalated
- AC-003.4: Given backlog audit fails mid-execution, when in /audit-full, then failure is logged and other domains continue

#### Dependencies

- Depends on: US-001, US-002

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| commands/audit-backlog.md | Command (Markdown) | New: standalone audit command |
| commands/audit-full.md | Command (Markdown) | Modify: add backlog domain |
| agents/audit-orchestrator.md | Agent (Markdown) | Modify: add backlog domain dispatch |

## Constraints

- Must follow existing /audit-* command conventions exactly
- Must handle 57+ entries without timeout
- Must not modify backlog entries (read-only audit)
- Early entries without ACs must degrade gracefully
- `--run-tests` default on means 10+ min audit runs are expected

## Non-Requirements

- Dedicated agent (command-only for now)
- Automatic status promotion (audit reports, doesn't fix)
- Backlog entry schema migration
- Modifying backlog entries to add missing ACs

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | N/A | Markdown commands, no hex boundaries |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Command ref | Project | CLAUDE.md | Add /audit-backlog to audit commands list |
| Backlog | Project | docs/backlog/BL-084 | Status -> implemented |
| CHANGELOG | Project | CHANGELOG.md | Add entry |

## Open Questions

None -- all resolved during grill-me interview.
