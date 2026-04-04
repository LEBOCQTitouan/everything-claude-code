# Spec: BL-079 Conditional Rule/Skill Loading

## Problem Statement

ECC loads all rules and skills unconditionally regardless of the target project's stack. This causes prompt bloat -- Django skills loaded for Rust projects, Spring Boot patterns for Python codebases. Stripe notes that "almost all agent rules are conditionally applied based on subdirectories." ECC already has 15+ language and 30+ framework detectors in `ecc-domain/src/detection/` but they're not used for rule filtering.

## Research Summary

- **Stripe Minions use directory-scoped rule loading** -- rules attach based on subdirectories/file patterns, synced across Cursor + Claude Code
- **Claude Code already has `paths:` in rules frontmatter** for file-pattern conditional loading -- `applies-to` extends this to stack-level
- **Tech stack detection uses file-marker heuristics** -- sentinel files (Cargo.toml, package.json, manage.py) in layered approach
- **Monorepos are the primary false-positive source** -- per-subtree resolution needed, not just root-level detection
- **User override mechanisms are essential** -- ability to force-install rules regardless of detection
- **Context window budget is the forcing function** -- keeping irrelevant rules out improves signal density

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Install-time filtering + applies-to frontmatter | Filter at install, frontmatter for transparency. No runtime hook needed. | Yes |
| 2 | Per-subdirectory detection for monorepos | Monorepos need subtree-level stack detection. Detect all stacks, install all matching. | No |
| 3 | No applies-to = install unconditionally | Backwards compatible. Existing rules and custom rules continue to work. | No |
| 4 | Reuse existing ecc-domain detection infra | 15+ language, 30+ framework detectors already exist. Wire into install. | No |

## User Stories

### US-001: applies-to Frontmatter Schema

**As a** rule/skill author, **I want** an `applies-to` field in frontmatter declaring applicability conditions, **so that** the rule is only installed when relevant.

#### Acceptance Criteria

- AC-001.1: Given a rule with `applies-to: { languages: [rust] }`, when parsed, then the applicability condition is extracted correctly
- AC-001.2: Given a rule with `applies-to: { frameworks: [django] }`, when parsed, then the framework condition is extracted
- AC-001.3: Given a rule with `applies-to: { files: ["manage.py"] }`, when parsed, then the sentinel file condition is extracted (file check is project root only, not recursive)
- AC-001.6: Given a rule with multiple conditions `applies-to: { languages: [python], frameworks: [django] }`, when evaluated, then conditions are combined with OR semantics (any match means applicable)
- AC-001.4: Given a rule WITHOUT `applies-to`, when parsed, then it is treated as universally applicable (backwards compatible)
- AC-001.5: Given `ecc validate rules`, when a rule has `applies-to` with invalid values, then a warning is emitted

#### Dependencies

- Depends on: none

### US-002: Install-Time Stack Detection and Filtering

**As a** developer running `ecc install`, **I want** rules filtered based on my project's detected stack, **so that** only relevant rules are installed.

#### Acceptance Criteria

- AC-002.1: Given `ecc install` runs in a Rust project (Cargo.toml present), when rules are merged, then only rules with `applies-to: { languages: [rust] }` or no `applies-to` are installed
- AC-002.2: Given `ecc install` runs in a monorepo with Rust + TypeScript, when detection runs, then both language rules are installed
- AC-002.3: Given `ecc install --all-rules`, when the flag is passed, then ALL rules are installed regardless of detection (override)
- AC-002.4: Given the install process, when detection results are shown, then the output includes "Detected: [language1, language2]" and "Skipped N rules (not matching detected stack)" lines on stderr
- AC-002.5: Given detection finds zero stacks (empty/new project), when `ecc install` runs, then it falls back to installing ALL rules (fail-open) with a warning "No stack detected, installing all rules"
- AC-002.6: Given detection throws an error (corrupt filesystem, permission denied), when `ecc install` runs, then it falls back to installing ALL rules (fail-open) with a warning

#### Dependencies

- Depends on: US-001

### US-003: Add applies-to to Existing Rules

**As a** maintainer, **I want** all existing language-specific rules annotated with `applies-to`, **so that** they are filtered correctly.

#### Acceptance Criteria

- AC-003.1: Given all rules in `rules/python/`, when inspected, then they have `applies-to: { languages: [python] }`
- AC-003.2: Given all rules in `rules/rust/`, when inspected, then they have `applies-to: { languages: [rust] }`
- AC-003.3: Given rules in `rules/common/`, when inspected, then they have NO `applies-to` (universal)
- AC-003.4: Given all rules in `rules/shell/`, `rules/yaml/`, `rules/ecc/`, when inspected, then they have appropriate `applies-to` annotations

#### Dependencies

- Depends on: US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `crates/ecc-domain/src/config/validate.rs` | Domain | Parse `applies-to` from frontmatter |
| `crates/ecc-domain/src/detection/` | Domain | Expose detection results for install filtering |
| `crates/ecc-app/src/install/global/steps.rs` | App | Add detection + filtering before merge |
| `crates/ecc-app/src/merge/mod.rs` | App | Filter rules by applicability before merge |
| `rules/python/*.md`, `rules/rust/*.md`, etc. | Content | Add `applies-to` frontmatter |

## Constraints

- Backwards compatible -- rules without `applies-to` install unconditionally
- Must not break `ecc validate rules` for existing rules
- Detection reuses existing `ecc-domain/src/detection/` modules
- `--all-rules` flag overrides filtering for power users
- ecc-domain must have zero I/O imports

## Non-Requirements

- No runtime rule filtering (install-time only)
- No skill filtering in v1 (rules only, skills later)
- No per-file conditional loading (Claude Code's `paths:` handles this)
- No UI for interactive rule selection (detection is automatic)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Install pipeline | Modified | `ecc install` now filters rules by detected stack |
| Validate pipeline | Modified | `ecc validate rules` checks `applies-to` syntax |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New pattern | MEDIUM | docs/adr/ | ADR for conditional loading |
| Schema change | MEDIUM | CLAUDE.md | Note applies-to field |
| Changelog | LOW | CHANGELOG.md | Feature entry |

## Open Questions

None -- all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope: install-time vs runtime vs both? | Both: install-time filtering + applies-to for documentation | User |
| 2 | Monorepo handling? | Per-subdirectory detection, install all matching | User |
| 3 | Test strategy? | Applicability evaluator + install integration + E2E with mock projects | User |
| 4 | Backwards compatible? | Yes, no applies-to = install unconditionally | Recommended |
| 5 | ADR? | Yes, for conditional loading pattern | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | applies-to Frontmatter Schema | 6 | none |
| US-002 | Install-Time Stack Detection and Filtering | 6 | US-001 |
| US-003 | Add applies-to to Existing Rules | 4 | US-001 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Parse languages condition | US-001 |
| AC-001.2 | Parse frameworks condition | US-001 |
| AC-001.3 | Parse files condition (root only) | US-001 |
| AC-001.4 | No applies-to = universal | US-001 |
| AC-001.5 | Validate invalid values | US-001 |
| AC-001.6 | OR semantics for multi-condition | US-001 |
| AC-002.1 | Rust project filtering | US-002 |
| AC-002.2 | Monorepo multi-stack | US-002 |
| AC-002.3 | --all-rules override | US-002 |
| AC-002.4 | Output format (Detected/Skipped) | US-002 |
| AC-002.5 | Zero-stack fallback (fail-open) | US-002 |
| AC-002.6 | Detection error fallback | US-002 |
| AC-003.1 | Python rules annotated | US-003 |
| AC-003.2 | Rust rules annotated | US-003 |
| AC-003.3 | Common rules: no applies-to | US-003 |
| AC-003.4 | Shell/yaml/ecc rules annotated | US-003 |

### Adversary Findings

| Dimension | R1 Score | R2 Score | Verdict |
|-----------|----------|----------|---------|
| Ambiguity | 72 | 90 | PASS |
| Edge Cases | 61 | 92 | PASS |
| Scope | 82 | 95 | PASS |
| Dependencies | 78 | 88 | PASS |
| Testability | 74 | 90 | PASS |
| Decisions | 85 | 88 | PASS |
| Rollback | 55 | 85 | PASS |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-04-conditional-rule-loading/spec.md` | Full spec + Phase Summary |
