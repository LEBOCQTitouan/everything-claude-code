# Spec: Audit CLAUDE.md for Inferable Content (BL-134)

## Revision
Revised 2026-04-09: Narrowed scope — keep pipeline-consumed sections (Running Tests, CLI Commands, Architecture). Only remove truly unused inferable content.

## Problem Statement

CLAUDE.md contains some inferable content that adds no value (doc hierarchy, development notes, slash command listing) alongside essential pipeline references and non-inferable rules. ETH Zurich research (2026) recommends removing content that hinders agents — but content actively consumed by pipeline commands is optimization, not hindrance.

## Research Summary

- ETH Zurich (2026): Remove content that hinders agents, not all inferable content
- Anthropic: CLAUDE.md should contain human-written, non-inferable instructions only
- Distinction: inferable + unused = remove; inferable + pipeline-consumed = keep
- Risk: removing pipeline-consumed sections forces agents to spend tokens rediscovering commands

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Keep Running Tests, CLI Commands, Architecture | Actively consumed by /implement, /verify, /audit pipelines | No |
| 2 | Remove Doc Hierarchy, Dev Notes, Slash Commands listing | Not referenced by any pipeline; agents find these by search | No |
| 3 | Remove 2 inferable gotchas (hooks.json, skill naming) | Discoverable from filesystem inspection | No |

## User Stories

### US-001: Remove unused inferable content from CLAUDE.md

**As a** developer, **I want** CLAUDE.md to contain only pipeline-relevant and non-inferable content, **so that** agents get cleaner context without losing active references.

#### Acceptance Criteria

- AC-001.1: "Doc Hierarchy" section removed
- AC-001.2: "Development Notes" section removed
- AC-001.3: "Slash Commands" listing removed (keep Spec-Driven Pipeline subsection)
- AC-001.4: Gotcha "hooks.json lives in hooks/" removed
- AC-001.5: Gotcha "Skill directory name must match" removed
- AC-001.6: "Running Tests" section preserved
- AC-001.7: "CLI Commands" section preserved
- AC-001.8: "Architecture" section preserved
- AC-001.9: All remaining gotchas preserved verbatim
- AC-001.10: Glossary preserved
- AC-001.11: CLAUDE.md line count reduced by >= 15 lines
- AC-001.12: ecc validate claude-md still passes

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| CLAUDE.md | Root | Remove unused inferable sections |

## Constraints

- Must not remove Running Tests, CLI Commands, or Architecture sections
- Must preserve all gotchas except the 2 identified as inferable
- ecc validate claude-md must still pass

## Non-Requirements

- Not rewriting any kept section
- Not changing crates/CLAUDE.md
- Not restructuring gotchas order

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | Doc-only change | No E2E impact |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | root | CHANGELOG.md | Add BL-134 entry |

## Open Questions

None.
