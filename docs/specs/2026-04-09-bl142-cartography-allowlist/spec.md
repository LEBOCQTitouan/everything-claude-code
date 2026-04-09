# Spec: Add docs/cartography/ to Phase-Gate Allowlist (BL-142)

## Problem Statement

The phase-gate hook blocks Write/Edit to paths not in its allowlist during spec/design phases. \`docs/cartography/\` is missing, so cartography delta processing fails when any workflow is active. Other \`docs/\` paths (specs, audits, backlog, adr, plans, designs, prds) are already allowed.

## Research Summary

- Root cause: \`allowed_prefixes()\` in \`phase_gate.rs\` has a hardcoded list of doc paths
- \`docs/cartography/\` was added after the phase-gate was implemented (BL-064)
- The fix is adding one string to the prefixes vec
- No web research needed — purely internal consistency fix

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Add to existing allowlist | Consistent with other docs/ paths | No |

## User Stories

### US-001: Allow cartography writes during workflow phases

**As a** doc-orchestrator pipeline, **I want** \`docs/cartography/\` writes permitted during spec/design phases, **so that** cartography delta processing works regardless of workflow state.

#### Acceptance Criteria

- AC-001.1: \`docs/cartography/\` added to \`allowed_prefixes()\` in phase_gate.rs
- AC-001.2: Phase-gate passes for Write to \`docs/cartography/journeys/test.md\` during plan phase
- AC-001.3: All existing phase_gate tests pass
- AC-001.4: cargo clippy passes
- AC-001.5: cargo build passes

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| \`crates/ecc-workflow/src/commands/phase_gate.rs\` | Adapter | Add 1 string to allowlist |

## Constraints

- Must not change any other phase-gate behavior
- Must not affect non-doc paths

## Non-Requirements

- Not restructuring the allowlist mechanism
- Not making the allowlist configurable

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | Allowlist addition | No E2E impact |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | root | CHANGELOG.md | Add BL-142 entry |

## Open Questions

None.
