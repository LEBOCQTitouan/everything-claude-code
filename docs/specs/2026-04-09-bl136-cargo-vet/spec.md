# Spec: Add cargo-vet for SLSA Level 2 Supply Chain Compliance (BL-136)

## Problem Statement

ECC's supply chain security has cargo-deny (license + advisory) and cosign (artifact signing) but lacks human-review verification for dependencies. cargo-vet fills this gap by requiring that every dependency has been audited by a trusted entity. This is the remaining gap for SLSA Level 2 compliance. With 30 direct deps, most are covered by Mozilla/Google community audit sets; ~8 need local certification.

## Research Summary

- cargo-vet init creates supply-chain/ with config.toml, audits.toml, imports.lock
- Existing deps get exemptions automatically — zero-disruption bootstrap
- Mozilla + Google audit sets cover most popular crates (serde, tokio, regex, tracing)
- CI integration: cache cargo-vet binary, run `cargo vet --locked`
- Complements cargo-deny: deny = "is this crate allowed?", vet = "has anyone read this code?"
- Effort: 1-2 hours total for a 30-dep workspace
- Dev deps can use safe-to-run (lighter criteria)

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Exemptions first, audit incrementally | Zero-disruption rollout; backlog worked down over time | No |
| 2 | Import Mozilla + Google audit sets | Broadest coverage, fewest local audits | No |
| 3 | safe-to-run for dev deps, safe-to-deploy for production | Standard practice — dev deps don't ship in binary | No |
| 4 | CI check in validate job (blocking) | security job is non-blocking; validate is the hard gate | No |

## User Stories

### US-001: Bootstrap cargo-vet configuration

**As a** maintainer, **I want** cargo-vet initialized in the workspace with a supply-chain/ directory, **so that** there is a canonical audit record for all dependencies.

#### Acceptance Criteria

- AC-001.1: supply-chain/config.toml created with safe-to-deploy default policy
- AC-001.2: supply-chain/audits.toml created (may be empty initially)
- AC-001.3: supply-chain/imports.lock committed
- AC-001.4: cargo vet init exits 0

#### Dependencies

- Depends on: none

### US-002: Import community audit sets

**As a** maintainer, **I want** Mozilla and Google audit sets imported, **so that** widely-audited crates are pre-certified without manual effort.

#### Acceptance Criteria

- AC-002.1: Mozilla audit set imported in config.toml
- AC-002.2: Google audit set imported in config.toml
- AC-002.3: cargo vet fetch-imports succeeds and imports.lock updated

#### Dependencies

- Depends on: US-001

### US-003: Certify remaining unresolved crates

**As a** maintainer, **I want** all deps covered (via audit, import, or exemption), **so that** cargo vet check exits 0.

#### Acceptance Criteria

- AC-003.1: cargo vet check exits 0 (all deps covered or exempted)
- AC-003.2: Dev deps configured with safe-to-run criteria via policy
- AC-003.3: rusqlite cert notes bundled C limitation

#### Dependencies

- Depends on: US-002

### US-004: Add blocking CI check

**As a** CI system, **I want** cargo vet to run as a required check on every PR, **so that** no unaudited dep can land on main.

#### Acceptance Criteria

- AC-004.1: ci.yml validate job includes cargo vet --locked step
- AC-004.2: cargo-vet binary cached in CI (runner.tool_cache pattern)
- AC-004.3: CI step is blocking (not continue-on-error)

#### Dependencies

- Depends on: US-003

### US-005: Document audit policy

**As a** contributor, **I want** a written policy for the audit workflow, **so that** I know how to add a new dependency.

#### Acceptance Criteria

- AC-005.1: supply-chain/README.md with 4-step new-dep workflow (check, certify, commit, push)
- AC-005.2: CLAUDE.md Running Tests section mentions cargo vet
- AC-005.3: CHANGELOG entry for BL-136

#### Dependencies

- Depends on: US-004

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| supply-chain/ | Root (new) | cargo-vet config, audits, imports |
| .github/workflows/ci.yml | CI | Add cargo vet step to validate job |
| supply-chain/README.md | Docs (new) | Audit policy and new-dep workflow |
| CLAUDE.md | Docs | 1-line addition to Running Tests |
| CHANGELOG.md | Docs | BL-136 entry |

## Constraints

- Must not break existing cargo-deny checks
- supply-chain/imports.lock must be committed (deterministic CI)
- cargo vet --locked in CI (no network fetch)
- validate job (blocking), not security job (non-blocking)
- Must work with the 5-target cross-compilation matrix in release.yml

## Non-Requirements

- Not replacing cargo-deny (complementary tools)
- Not fully auditing all 30 deps upfront (exemptions OK)
- Not adding cargo-auditable (separate BL-135)
- Not changing release.yml or cosign signing

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | Config + CI only | No E2E impact |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | root | CHANGELOG.md | Add BL-136 entry |
| CLAUDE.md | root | Running Tests | Add cargo vet line |

## Open Questions

None.
