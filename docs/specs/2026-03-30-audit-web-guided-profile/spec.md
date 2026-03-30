# Spec: Audit-Web Guided Profile & Self-Improvement (BL-107)

## Problem Statement

`/audit-web` scans 8 hardcoded dimensions with no project-specific customization, no persistence between runs, no self-improvement, and no deterministic validation. Users must manually specify `--focus` dimensions each time, custom dimensions are lost, and report quality is unverifiable. This makes the audit less useful over time and prevents building institutional knowledge about what matters for this project.

## Research Summary

- Prior art: existing `ecc sources` CLI commands follow the same Rust hexagonal pattern (domain types → app use cases → CLI wiring) — replicate for audit-web profile
- The `serde-saphyr` crate (migrated in BL-099) handles YAML parsing for the profile artifact
- ECC's existing profile/config patterns (`~/.ecc/config.toml`, `docs/sources.md`) provide structural precedent for persisted project artifacts

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | YAML format for profile | Matches ECC skill/agent frontmatter convention; human-editable; serde-saphyr available | Yes |
| 2 | Hexagonal architecture for profile types | Consistent with sources, backlog, workflow modules | No |
| 3 | Renumber audit-web phases 0-5 | Clean insertion of Phase 0 and Phase 5; renumber existing 1-4 to 1-4 within the 0-5 range | No |
| 4 | Deterministic report validation in Rust | Ensures report structure, scoring ranges, and source citations are machine-checkable | No |
| 5 | Sanitize URLs/queries in custom dimensions | Prevent injection via search query templates in profile | No |
| 6 | Profile at `docs/audits/` (not `.ecc/` or `.claude/`) | Profile is project-scoped, committed to git, visible in PRs. `.ecc/` is user-scoped, `.claude/` is tool-scoped | No |
| 7 | Schema version field (`version: 1`) in profile YAML | Enables future breaking changes with explicit migration errors | No |

## User Stories

### US-001: Audit-web profile domain types

**As a** developer, **I want** typed `AuditWebProfile`, `AuditDimension`, `DimensionThreshold`, and `ImprovementSuggestion` domain types, **so that** profile data is validated at construction time.

#### Acceptance Criteria

- AC-001.1: Given an `AuditWebProfile`, when constructed, then it contains dimensions (standard + custom), thresholds, and improvement history
- AC-001.2: Given an `AuditDimension`, when it has a custom search query template, then the template is URL-sanitized (no shell metacharacters, no script injection)
- AC-001.3: Given a YAML string representing a profile, when parsed via `serde_saphyr::from_str`, then it deserializes into `AuditWebProfile`
- AC-001.4: Given an `AuditWebProfile`, when serialized then re-parsed, then it produces an identical struct (round-trip)
- AC-001.5: Given a corrupted or unparseable YAML file, when loaded as `AuditWebProfile`, then it returns a typed error with parse location and human-readable message
- AC-001.6: Given an `AuditDimension` with a custom query template, then sanitization allows: alphanumeric, spaces, hyphens, underscores, periods, forward slashes, and `{placeholder}` tokens only — all other characters rejected with a validation error
- AC-001.7: Given the profile YAML, then it includes a `version: 1` field at the root; when the parser encounters an unknown version, it returns an error with upgrade instructions

#### Dependencies

- Depends on: none

### US-002: Audit-web profile CLI commands

**As a** CLI user, **I want** `ecc audit-web profile init|show|validate|reset` subcommands, **so that** I can manage the audit profile from the terminal.

#### Acceptance Criteria

- AC-002.1: Given `ecc audit-web profile init`, when run in a project with Cargo.toml, then it scans codebase characteristics and generates `docs/audits/audit-web-profile.yaml`
- AC-002.2: Given `ecc audit-web profile show`, when a profile exists, then it displays the profile contents to stdout
- AC-002.3: Given `ecc audit-web profile validate`, when run on a valid profile, then it passes with exit 0
- AC-002.4: Given `ecc audit-web profile reset`, when run, then it deletes the profile file
- AC-002.5: Given `ecc audit-web profile init`, when a profile already exists, then it exits with an error message (use `reset` first)
- AC-002.6: Given `ecc audit-web profile reset`, when run without `--force`, then it prompts for confirmation before deleting
- AC-002.7: Given `ecc audit-web profile init`, when run in a Cargo workspace with multiple Cargo.toml files, then it scans the workspace root manifest

#### Dependencies

- Depends on: US-001

### US-003: Deterministic report validation

**As a** developer, **I want** `ecc audit-web validate-report <path>` to check report structure deterministically, **so that** malformed reports are caught before they're committed.

#### Acceptance Criteria

- AC-003.1: Given a valid radar report with all required sections, when validated, then it passes with exit 0
- AC-003.2: Given a report missing required sections (Techniques, Tools, Platforms, Languages & Frameworks, Feature Opportunities), when validated, then it fails with specific error messages listing missing sections
- AC-003.3: Given a finding with scores outside 0-5 range, when validated, then it fails with a score-range error
- AC-003.4: Given a finding with fewer than 3 source citations, when validated, then it warns (non-blocking)

#### Dependencies

- Depends on: none

### US-004: Command markdown update (Phase 0 + Phase 5)

**As a** `/audit-web` user, **I want** Phase 0 (guided setup) and Phase 5 (self-improvement) in the command, **so that** the audit is customized to my project and improves over time.

#### Acceptance Criteria

- AC-004.1: Given no profile exists, when `/audit-web` runs, then Phase 0 triggers automatically and generates a profile interactively via AskUserQuestion
- AC-004.2: Given `--setup` flag, when `/audit-web --setup` runs, then Phase 0 triggers regardless of existing profile
- AC-004.3: Given an existing profile, when `/audit-web` runs normally, then Phase 0 loads the profile silently and passes custom dimensions to Phase 2
- AC-004.4: Given Phase 4 completes, when Phase 5 runs, then it analyzes findings for coverage gaps and suggests new dimensions
- AC-004.5: Given Phase 5 suggestions, when user accepts a suggestion via AskUserQuestion, then it's persisted to the profile
- AC-004.6: Given all phases, when numbered, then they are 0-5 (0: guided setup, 1: inventory, 2: landscape scan, 3: evaluate, 4: synthesize, 5: self-improvement)
- AC-004.7: Given stale profile entries (dimensions referencing removed tools/files), when Phase 0 loads, then it detects and flags them for user review
- AC-004.8: Given Phase 0 triggers in a non-interactive context (CI, piped stdin), when AskUserQuestion is unavailable, then it generates a default profile with standard dimensions only and logs a warning

#### Dependencies

- Depends on: US-001, US-002

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-domain/src/audit_web/profile.rs` | Domain | New: `AuditWebProfile`, `AuditDimension`, `DimensionThreshold`, `ImprovementSuggestion` types |
| `crates/ecc-domain/src/audit_web/dimension.rs` | Domain | New: standard dimension definitions, custom dimension validation |
| `crates/ecc-domain/src/audit_web/report_validation.rs` | Domain | New: report structure validation, score range checks, citation counting |
| `crates/ecc-app/src/audit_web.rs` | App | New: profile init/show/validate/reset + validate-report use cases |
| `crates/ecc-cli/src/commands/audit_web.rs` | CLI | New: `ecc audit-web profile` + `validate-report` subcommands |
| `commands/audit-web.md` | Command | Modified: Phase 0 (guided setup), Phase 5 (self-improvement), renumbered phases 0-5 |

## Constraints

- `ecc-domain` must have zero I/O imports — profile types are pure data with serde derives
- Standard 8 dimensions remain the default — custom dimensions add to, not replace them
- Profile is human-readable YAML, editable with any text editor
- URL/query sanitization in custom dimension templates (no shell metacharacters)
- All existing audit-web behavior preserved for runs without a profile
- Profile loading must not block or slow down audit startup (cold-path YAML parse)

## Non-Requirements

- Deterministic scoring or ring classification (stays LLM-evaluated in Phase 3)
- New dimension agent types (custom dimensions use existing web-scout pattern with query templates)
- Profile sharing across repos or teams
- UI for profile editing beyond CLI + text editor
- Web dashboard or visualization of profile history

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| FileSystem port | Reused | Profile read/write via existing FileSystem trait |
| CLI (ecc audit-web) | New subcommands | Integration tests for profile CRUD + validate-report |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New feature | ADR | `docs/adr/0035-audit-web-profile-system.md` | Create ADR for profile system design |
| New bounded context | Reference | `docs/domain/bounded-contexts.md` | Add Audit Web context with profile types |
| CLI commands | Project | `CLAUDE.md` | Add `ecc audit-web profile` + `validate-report` commands |
| CHANGELOG | Project | `CHANGELOG.md` | Add BL-107 entry |
| Backlog status | Metadata | `docs/backlog/BL-107-*.md` | Mark `status: implemented` |

## Open Questions

None — all resolved during grill-me interview.
