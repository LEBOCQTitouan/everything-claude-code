# Spec: BL-125 — Token Boilerplate Extraction & Context Trimming

## Problem Statement

14 agent/command files contain identical ~40-word TodoWrite graceful degradation boilerplate. CLAUDE.md may have excessive CLI listings. Language-specific rules may load unnecessarily in Rust-only projects. Generic rules files may contain prose that belongs in docs/ rather than context-loaded rules/.

## Research Summary

- Web research skipped — internal content refactoring, no external patterns needed.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | All 5 items addressed | User chose full scope | No |
| 2 | Independent commits per item | Each can be reverted independently | No |

## User Stories

### US-001: Extract TodoWrite Boilerplate
**As a** context optimizer, **I want** the TodoWrite graceful degradation boilerplate extracted from 14 files, **so that** ~560 tokens of repeated text are eliminated from agent/command context.

#### Acceptance Criteria
- AC-001.1: Given 14 files with inline TodoWrite boilerplate, when the boilerplate is removed, then the phrase "If TodoWrite is unavailable, proceed without tracking" no longer appears inline
- AC-001.2: Given rules/ecc/development.md, when updated, then it documents the convention that agents/commands with TodoWrite should handle unavailability gracefully
- AC-001.3: Given ecc validate agents and ecc validate commands, when run, then both pass

### US-002: Slim CLAUDE.md CLI Reference
**As a** context optimizer, **I want** CLAUDE.md CLI listing trimmed to essential commands, **so that** context tokens are saved.

#### Acceptance Criteria
- AC-002.1: Given CLAUDE.md, when the CLI section is inspected, then it contains at most 20 `ecc` command lines (currently 16+6=22 from BL-126)
- AC-002.2: Given trimmed commands, when a pointer to docs/commands-reference.md exists, then users can find the full listing

### US-003: Verify Language Rule Loading
**As a** context optimizer, **I want** language rules verified for conditional loading, **so that** Rust-only projects don't load 13 irrelevant language rule sets.

#### Acceptance Criteria
- AC-003.1: Given a Rust-only project, when rules are loaded, then only rules/common/ and rules/rust/ are included (not perl/, swift/, java/, etc.)
- AC-003.2: Given each non-Rust rule dir, when inspected, then it has correct paths: or applies-to: frontmatter for conditional loading

### US-004: Trim performance.md
**As a** context optimizer, **I want** rules/common/performance.md trimmed to essential content, **so that** context-loaded rules are concise.

#### Acceptance Criteria
- AC-004.1: Given performance.md, when trimmed, then it retains model routing table and thinking effort tiers
- AC-004.2: Given verbose prose sections, when moved, then they are relocated to docs/ (not deleted)
- AC-004.3: Given trimmed file, when measured, then it is <=30 lines

### US-005: Trim agents.md
**As a** context optimizer, **I want** rules/common/agents.md trimmed, **so that** it doesn't duplicate the system-reminder agent listing.

#### Acceptance Criteria
- AC-005.1: Given agents.md, when trimmed, then it is <=10 lines
- AC-005.2: Given the trimmed content, when inspected, then it has a pointer to the system-reminder for the full agent listing

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| agents/*.md (2 files) | Content | Remove TodoWrite boilerplate |
| commands/*.md (12 files) | Content | Remove TodoWrite boilerplate |
| rules/ecc/development.md | Content | Document convention |
| CLAUDE.md | Content | Slim CLI section |
| rules/common/performance.md | Content | Trim |
| rules/common/agents.md | Content | Trim |
| rules/{lang}/*.md (13 dirs) | Content | Verify frontmatter |

## Constraints
- ecc validate rules, agents, commands must pass after changes
- No Rust code changes
- No behavior changes — content only

## Non-Requirements
- New Rust code, new CLI commands, architecture changes

## E2E Boundaries Affected
None — content-only changes.

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Changelog | CHANGELOG | CHANGELOG.md | Add entry |

## Open Questions
None.
