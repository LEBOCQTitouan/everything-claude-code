# Spec: BL-026 Quarterly MCP Version Audit

## Problem Statement

ECC pins MCP server versions in `mcp-configs/mcp-servers.json` (BL-002). Without a check mechanism, these pins silently drift from upstream releases. A quarterly manual audit prevents security and compatibility issues, but needs tooling to be practical — manually checking 10 packages against npm is tedious and error-prone.

## Research Summary

- `npm view <package> version` is the standard CLI approach; `curl https://registry.npmjs.org/<package>/latest | jq .version` works without npm installed
- MCP best practices recommend treating MCP servers like code dependencies: pin versions, have rollback strategies
- No existing MCP version audit tooling exists — this would be first-of-kind
- The project already has shell scripts in `scripts/` following `#!/usr/bin/env bash` + `set -euo pipefail` conventions

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Shell script, not Rust CLI | Quarterly task, zero domain logic, disproportionate to build into hexagonal architecture | No |
| 2 | Report only, no auto-update | Prevents silent breaking changes from major version bumps | No |
| 3 | Use npm registry HTTP API | Works without npm installed; curl + jq are already project dependencies | No |

## User Stories

### US-001: Check MCP versions against npm registry

**As a** ECC maintainer, **I want** to run a script that compares pinned MCP versions against latest npm releases, **so that** I can see which packages are outdated.

#### Acceptance Criteria

- AC-001.1: Given `mcp-servers.json` with npx-based servers, when the script runs, then it outputs a table: package | pinned | latest | status (current/outdated/unpinned)
- AC-001.2: Given a package using `@latest`, when checked, then it is flagged as "unpinned"
- AC-001.3: Given all versions are current, when the script runs, then it confirms no updates needed
- AC-001.4: Given an HTTP-type server, when the script runs, then it is skipped (not in the table)
- AC-001.5: Given npm registry is unreachable for a package, when the script runs, then it skips that package with a warning and continues
- AC-001.6: Given a package arg with no version suffix (no `@version`), when the script runs, then it is flagged as "unpinned"
- AC-001.7: Given the script completes, when all packages are current, then exit code is 0; when any package is outdated, then exit code is 1

#### Dependencies

- Depends on: none (BL-002 already implemented)

### US-002: Quarterly audit runbook

**As a** ECC maintainer, **I want** a runbook documenting the quarterly audit process, **so that** the process is repeatable and not dependent on memory.

#### Acceptance Criteria

- AC-002.1: Runbook exists at `docs/runbooks/audit-mcp-versions.md`
- AC-002.2: Runbook includes: prerequisites, how to run the script, how to interpret output, how to update pins, how to update the audit_reminder date

#### Dependencies

- Depends on: US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `scripts/` | Infrastructure (outside Rust) | New `audit-mcp-versions.sh` |
| `docs/runbooks/` | Documentation | New `audit-mcp-versions.md` |

Zero Rust crate changes. Zero hexagonal boundary impact.

## Constraints

- Must use `curl` + `jq` (not `npm`) to avoid requiring Node.js installation
- Must follow existing script conventions (`set -euo pipefail`, snake_case functions)
- Must handle `@latest` as a special "unpinned" case
- Must skip HTTP-type MCP servers gracefully
- Must validate that `curl` and `jq` are available at startup and exit with a clear error message if either is missing
- Must extract the package identifier as `args[1]` in npx-based servers (skipping the `-y` flag at index 0); any remaining args beyond index 1 are ignored

## Non-Requirements

- Auto-updating versions in mcp-servers.json
- CI/CD integration or GitHub Actions workflow
- Rust CLI subcommand (`ecc validate mcp-versions`)
- Reminder mechanism beyond the existing `_comments.audit_reminder` field
- Handling private npm packages (all are public)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | None | None — no Rust code changes |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New script | Low | CLAUDE.md | No change needed |
| New runbook | Low | docs/runbooks/ | Add new file |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | What is explicitly OUT of scope? | Report only — no auto-updating, no CI, no Rust CLI | Recommended |
| 2 | What happens when npm registry is unreachable? | Skip + warn, continue checking others, exit code 1 | Recommended |
| 3 | Which paths need 100% coverage vs 80%? | Version parsing 100%, npm query logic 80% | Recommended |
| 4 | Are there latency/throughput requirements? | No constraints — quarterly manual script | Recommended |
| 5 | Does this touch auth, user data, or external APIs? | npm registry only — no auth, read-only | Recommended |
| 6 | Will this change any existing public API or data contract? | No breaking changes — new files only | Recommended |
| 7 | Are there domain terms that need defining? | No new terms needed | Recommended |
| 8 | Which decisions warrant an ADR? | None — LOW scope doesn't warrant ADR overhead | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Check MCP versions against npm registry | 7 | none |
| US-002 | Quarterly audit runbook | 2 | US-001 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Output table: package, pinned, latest, status | US-001 |
| AC-001.2 | Flag `@latest` as "unpinned" | US-001 |
| AC-001.3 | Confirm no updates needed when all current | US-001 |
| AC-001.4 | Skip HTTP-type servers | US-001 |
| AC-001.5 | Skip unreachable packages with warning | US-001 |
| AC-001.6 | Flag no-version-suffix args as "unpinned" | US-001 |
| AC-001.7 | Exit 0 when current, exit 1 when outdated | US-001 |
| AC-002.1 | Runbook at docs/runbooks/audit-mcp-versions.md | US-002 |
| AC-002.2 | Runbook covers prerequisites, usage, interpretation, updating | US-002 |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| Ambiguity | PASS | ACs use Given/When/Then, table format concrete |
| Edge cases | PASS (round 3) | Added AC-001.6, AC-001.7, args extraction constraint |
| Scope | PASS | Tightly bounded, explicit non-requirements |
| Dependencies | PASS | curl/jq validated at startup, BL-002 landed |
| Testability | PASS (round 3) | Exit codes explicit, all ACs observable at shell boundary |
| Decisions | PASS | Three decisions documented with rationale |
| Rollback | PASS | Two new files, trivial git revert |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-26-quarterly-mcp-version-audit/spec.md | Full spec |
