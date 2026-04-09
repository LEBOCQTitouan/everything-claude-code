# Spec: Complete ECC_WORKFLOW_BYPASS removal (ADR-0056 finale)

## Problem Statement

The `ECC_WORKFLOW_BYPASS` env var was deprecated in ADR-0056 and replaced by the auditable bypass system (ADR-0055, `ecc bypass grant`). All Rust code already ignores the env var, but cleanup artifacts remain: `.envrc` still exports it, and CLAUDE.md gotchas still reference the old approach. This creates confusion for developers who see the env var and assume it's functional.

## Research Summary

Web research skipped — this is a cleanup of a completed internal deprecation.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Delete .envrc entirely | Its only content is the defunct env var | No |
| 2 | Update CLAUDE.md gotcha | Remove reference to bypass via .envrc | No |
| 3 | Keep regression tests | Tests prove the var is ignored — useful documentation | No |

## User Stories

### US-001: Remove defunct bypass artifacts

**As a** developer, **I want** the defunct `ECC_WORKFLOW_BYPASS` artifacts removed, **so that** the codebase doesn't mislead about hook bypass mechanisms.

#### Acceptance Criteria

- AC-001.1: `.envrc` file deleted from project root
- AC-001.2: CLAUDE.md no longer references `ECC_WORKFLOW_BYPASS=1` or `.envrc` bypass
- AC-001.3: CLAUDE.md references the current bypass mechanism (`ecc bypass grant`)
- AC-001.4: `cargo test` passes (no regression from .envrc deletion)
- AC-001.5: `cargo clippy -- -D warnings` clean

#### Dependencies
- None

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `.envrc` | Config | Delete |
| `CLAUDE.md` | Docs | Update gotcha section |

## Constraints

- Must not break existing bypass system (ADR-0055 `ecc bypass grant`)
- Historical ADR docs (0055, 0056) must NOT be modified

## Non-Requirements

- Removing regression tests that set the env var (they prove it's ignored)
- Modifying the user's private `~/.claude/CLAUDE.md` (outside repo)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | N/A | Pure file cleanup |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Gotcha update | Onboarding | CLAUDE.md | Update bypass reference |
| Changelog | Project | CHANGELOG.md | Note ADR-0056 completion |

## Open Questions

None.
