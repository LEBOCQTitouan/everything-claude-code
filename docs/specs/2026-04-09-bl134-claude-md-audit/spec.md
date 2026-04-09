# Spec: Audit CLAUDE.md for Inferable Content (BL-134)

## Problem Statement

CLAUDE.md contains ~40% inferable content (test commands, architecture, CLI lists, doc hierarchy, dev notes) alongside ~60% non-inferable rules (gotchas, policies, glossary). ETH Zurich research (2026) shows LLM-generated context files hinder agents. Anthropic recommends human-written, non-inferable instructions only. Removing inferable content reduces token cost per session and prevents staleness.

## Research Summary

- ETH Zurich (2026): LLM-generated context files hinder agent performance
- Anthropic engineering guide: CLAUDE.md should contain human-written, non-inferable instructions only
- InfoQ (March 2026): AGENTS.md-style files most effective when containing constraints, not descriptions
- Best practice: pointer pattern — replace removed sections with "See X for details"
- Risk: removing too much can break workflows that parse CLAUDE.md (e.g., ecc validate claude-md)

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Remove ALL inferable sections | Research-backed: agents perform better without redundant context | No |
| 2 | Keep gotchas, policy rules, glossary | Non-derivable constraints that prevent agent errors | No |
| 3 | Add pointers to source-of-truth locations | Info remains discoverable without bloating context | No |

## User Stories

### US-001: Remove inferable content from CLAUDE.md

**As a** developer, **I want** CLAUDE.md to contain only non-inferable instructions, **so that** agents get cleaner context and the file doesn't go stale.

#### Acceptance Criteria

- AC-001.1: "Running Tests" section removed
- AC-001.2: "Architecture" section replaced with pointer to docs/ARCHITECTURE.md
- AC-001.3: "CLI Commands" section removed
- AC-001.4: "Slash Commands" section removed
- AC-001.5: "Doc Hierarchy" section removed or replaced with pointer
- AC-001.6: "Development Notes" section removed
- AC-001.7: All gotchas preserved verbatim
- AC-001.8: Spec-driven pipeline rules preserved
- AC-001.9: Command workflow enforcement rules preserved
- AC-001.10: Glossary preserved
- AC-001.11: CLAUDE.md line count reduced by >= 30 lines
- AC-001.12: ecc validate claude-md still passes

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| CLAUDE.md | Root | Remove inferable sections, add pointers |

## Constraints

- Must not remove any gotcha or policy rule
- Must preserve glossary definitions
- ecc validate claude-md must still pass after changes
- Pointers must reference correct file paths

## Non-Requirements

- Not rewriting gotchas (preserve verbatim)
- Not changing crates/CLAUDE.md or other nested CLAUDE.md files
- Not restructuring the gotchas order

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
