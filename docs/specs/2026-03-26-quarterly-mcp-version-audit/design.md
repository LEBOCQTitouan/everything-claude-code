# Solution: BL-026 Quarterly MCP Version Audit

## Spec Reference
Concern: dev, Feature: BL-026 — Quarterly MCP version audit process

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `scripts/tests/test-audit-mcp-versions.bats` | Create | Bats test suite — RED phase first, drives script implementation | US-001, US-002 |
| 2 | `scripts/audit-mcp-versions.sh` | Create | Main script: parse JSON, query npm registry, output table, set exit code | US-001 (all ACs) |
| 3 | `docs/runbooks/audit-mcp-versions.md` | Create | Quarterly audit runbook: prerequisites, usage, interpretation, updating | US-002 (AC-002.1, AC-002.2) |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | unit | Extracts package names and pinned versions from npx args | AC-001.1, AC-001.6 | `bats scripts/tests/test-audit-mcp-versions.bats --filter "extracts pinned version from args"` | PASS |
| PC-002 | unit | Flags @latest as unpinned | AC-001.2 | `bats scripts/tests/test-audit-mcp-versions.bats --filter "flags @latest as unpinned"` | PASS |
| PC-003 | unit | Flags no-version-suffix as unpinned | AC-001.6 | `bats scripts/tests/test-audit-mcp-versions.bats --filter "flags no-version-suffix as unpinned"` | PASS |
| PC-004 | unit | Skips HTTP-type servers | AC-001.4 | `bats scripts/tests/test-audit-mcp-versions.bats --filter "skips HTTP-type servers"` | PASS |
| PC-005 | unit | Exits 0 when all versions are current | AC-001.3, AC-001.7 | `bats scripts/tests/test-audit-mcp-versions.bats --filter "exits 0 when all versions are current"` | PASS |
| PC-006 | unit | Exits 1 when any package is outdated | AC-001.7 | `bats scripts/tests/test-audit-mcp-versions.bats --filter "exits 1 when any package is outdated"` | PASS |
| PC-007 | unit | Warns and skips unreachable registry | AC-001.5 | `bats scripts/tests/test-audit-mcp-versions.bats --filter "warns and skips unreachable registry"` | PASS |
| PC-008 | unit | Exits with error when required tool is missing | Constraint | `bats scripts/tests/test-audit-mcp-versions.bats --filter "exits with error when required tool is missing"` | PASS |
| PC-009 | unit | Outputs table with correct columns | AC-001.1 | `bats scripts/tests/test-audit-mcp-versions.bats --filter "outputs table with correct columns"` | PASS |
| PC-010 | unit | Exits 1 when unpinned packages exist | AC-001.7 | `bats scripts/tests/test-audit-mcp-versions.bats --filter "exits 1 when unpinned packages exist"` | PASS |
| PC-011 | integration | Runbook exists with required sections | AC-002.1, AC-002.2 | `bats scripts/tests/test-audit-mcp-versions.bats --filter "runbook exists with required sections"` | PASS |
| PC-012 | lint | Runbook markdownlint | AC-002.1 | `npx markdownlint-cli docs/runbooks/audit-mcp-versions.md` | exit 0 |
| PC-013 | lint | Script shellcheck | Constraint | `shellcheck scripts/audit-mcp-versions.sh` | exit 0 |
| PC-014 | integration | Runs against real mcp-servers.json (skipped if SKIP_NETWORK=1) | AC-001.1, AC-001.4 | `bats scripts/tests/test-audit-mcp-versions.bats --filter "runs against real mcp-servers.json"` | PASS |
| PC-015 | unit | Ignores trailing args after package version (e.g. --project-ref) | AC-001.1 | `bats scripts/tests/test-audit-mcp-versions.bats --filter "ignores trailing args after package"` | PASS |
| PC-016 | unit | Mixed-state output: current + outdated + unpinned + HTTP in one run | AC-001.1, AC-001.7 | `bats scripts/tests/test-audit-mcp-versions.bats --filter "mixed state output"` | PASS |

### Coverage Check

| AC | Covered by PC(s) |
|----|-------------------|
| AC-001.1 | PC-001, PC-009, PC-014, PC-015, PC-016 |
| AC-001.2 | PC-002 |
| AC-001.3 | PC-005 |
| AC-001.4 | PC-004, PC-014 |
| AC-001.5 | PC-007 |
| AC-001.6 | PC-001, PC-003 |
| AC-001.7 | PC-005, PC-006, PC-010 |
| AC-002.1 | PC-011, PC-012 |
| AC-002.2 | PC-011 |

All ACs covered. Zero uncovered.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| — | None | None | None | No E2E boundaries affected | — | — |

### E2E Activation Rules

No E2E tests to activate — zero Rust code changes, zero port/adapter changes.

## Test Strategy

TDD order (dependency-first):

1. **PC-008** — Missing tool detection (simplest, gates everything)
2. **PC-004** — HTTP server skipping (pure JSON parsing, no network)
3. **PC-001** — Version extraction (core parsing logic)
4. **PC-009** — Table format (validates output structure)
5. **PC-003** — No-version-suffix as unpinned
6. **PC-002** — @latest detection
7. **PC-010** — Exit 1 on unpinned
8. **PC-005** — Exit 0 when all current (mocked curl)
9. **PC-006** — Exit 1 when outdated (mocked curl)
10. **PC-007** — Unreachable registry (mocked curl failure)
11. **PC-015** — Trailing args ignored (e.g. --project-ref)
12. **PC-016** — Mixed-state output (current + outdated + unpinned + HTTP)
13. **PC-014** — Integration with real mcp-servers.json (skip if SKIP_NETWORK=1)
14. **PC-011** — Runbook existence and content
15. **PC-012** — Markdownlint
14. **PC-013** — Shellcheck

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/runbooks/audit-mcp-versions.md` | New | Create | Prerequisites, usage, output interpretation, updating pins, audit_reminder | US-002 |
| 2 | `CHANGELOG.md` | Update | Append | BL-026: Add quarterly MCP version audit script and runbook | US-001, US-002 |

No ADRs needed.

## SOLID Assessment

**CLEAN** (Uncle Bob). Shell script outside Rust workspace — no hexagonal boundary impact. Implementation notes:
- Use named constant for registry URL (`readonly NPM_REGISTRY_URL`)
- Separate parse and query functions for testability
- Handle curl failures explicitly with per-package warnings

## Robert's Oath Check

**CLEAN** (after resolving two notes):
- W1 (shell vs Rust): justified by spec decision #1 — quarterly task, zero domain logic
- W2 (script hygiene): `set -euo pipefail` already a spec constraint

## Security Notes

**One HIGH finding — must address in implementation:**
- Package names from JSON must be validated against `^[@a-zA-Z0-9._/-]+$` before curl URL interpolation
- Use `while IFS= read -r` for jq output consumption (no field splitting)
- Always double-quote variables in curl calls
- Use `curl -sf` (silent + fail) for proper error detection
- HTTPS confirmed for registry URL

## Rollback Plan

Reverse dependency order:
1. Remove `docs/runbooks/audit-mcp-versions.md`
2. Remove `scripts/audit-mcp-versions.sh`
3. Remove `scripts/tests/test-audit-mcp-versions.bats`

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID | CLEAN | 0 |
| Robert | CLEAN | 0 (2 notes resolved) |
| Security | HIGH (addressed) | 1 (package name validation) |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| Coverage | PASS | All 9 ACs covered by 16 PCs |
| Order | PASS | Dependency-first TDD ordering |
| Fragility | PASS (round 2) | PC-014 skip-friendly, assertions clarified |
| Rollback | PASS | Three new files, trivial git revert |
| Architecture | PASS | Outside Rust hexagonal boundary |
| Blast radius | PASS | Zero existing file modifications |
| Missing PCs | PASS (round 2) | Added PC-015 (trailing args), PC-016 (mixed state) |
| Doc plan | PASS | Runbook + CHANGELOG, no ADR needed |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `scripts/tests/test-audit-mcp-versions.bats` | Create | US-001, US-002 |
| 2 | `scripts/audit-mcp-versions.sh` | Create | US-001 |
| 3 | `docs/runbooks/audit-mcp-versions.md` | Create | US-002 |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-26-quarterly-mcp-version-audit/spec.md | Full spec |
| docs/specs/2026-03-26-quarterly-mcp-version-audit/design.md | Full design |

Trivial `git revert` — all additive changes, zero existing file modifications.
