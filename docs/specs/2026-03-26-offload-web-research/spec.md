# Spec: Offload Web Research to Task Subagents in /spec-* Commands (BL-049)

## Problem Statement

Currently each `/spec-*` command (spec-dev, spec-fix, spec-refactor) runs WebSearch directly in the main session during Phase 3. These calls accumulate throwaway tokens — raw search results that are never re-used after the Research Summary is produced. Over a full spec session this inflates context meaningfully and can trigger `/compact` interruptions. The fix is to delegate Phase 3 to a Task subagent that runs WebSearch in its own isolated context and returns only a condensed Research Summary to the main session.

## Research Summary

Web research skipped — this is a mechanical refactoring of markdown command files with clear scope from the backlog item.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Use Task subagent (not a new agent file) for web research | Keeps it lightweight — no new agent markdown file. The Task instruction is inline in each command. | No |
| 2 | Pass intent_type + description + tech stack to subagent | Minimal context needed for query derivation. No full conversation context. | No |
| 3 | Subagent returns only Research Summary (3-7 bullets) | Matches existing output format. Raw search tokens stay in subagent context. | No |
| 4 | Phase 4 (grill-me) stays inline | Needs full prior-phase context and AskUserQuestion must feel native. | No |
| 5 | Domain-specific framing preserved per command | Each command's Phase 3 has different search focus (libraries vs fix patterns vs refactoring guides). The subagent prompt includes this framing. | No |

## User Stories

### US-001: Delegate Phase 3 WebSearch to Task subagent

**As a** developer running `/spec-*`, **I want** Phase 3 web research delegated to a Task subagent, **so that** raw WebSearch tokens don't inflate my main session context.

#### Acceptance Criteria

- AC-001.1: Given `commands/spec-dev.md`, when Phase 3 runs, then it launches a Task subagent for WebSearch instead of calling WebSearch inline
- AC-001.2: Given `commands/spec-fix.md`, when Phase 3 runs, then it launches a Task subagent for WebSearch
- AC-001.3: Given `commands/spec-refactor.md`, when Phase 3 runs, then it launches a Task subagent for WebSearch
- AC-001.4: Given the subagent, when it receives context, then it gets only: intent type (dev/fix/refactor), user's input description, and detected tech stack
- AC-001.5: Given the subagent, when it completes, then it returns only a Research Summary bullet list (3-7 items) to the main context
- AC-001.6: Given the subagent, when WebSearch is unavailable, then it returns a warning message and the main command proceeds without hard-failing
- AC-001.7: Given all three command files, when Phase 3 markdown is inspected, then no inline WebSearch or exa-web-search invocation instructions remain — all search is delegated to the Task subagent
- AC-001.8: Given Phase 4 (grill-me), when it runs after Phase 3, then it is unchanged and still runs inline in the main session
- AC-001.9: Given the Task subagent invocation in each command file, when inspected, then `allowedTools` includes `WebSearch` (and optionally exa-web-search MCP) and the prompt template is documented inline
- AC-001.10: Given the subagent, when WebSearch is unavailable, then it attempts exa-web-search MCP as fallback before returning a warning
- AC-001.11: Given the subagent Task, when it fails or times out, then the main command proceeds with "Web research skipped: subagent failed" and does not hard-fail

#### Dependencies

- Depends on: none

### US-002: Preserve domain-specific search framing

**As a** developer, **I want** each command's web research to retain its domain-specific search focus, **so that** the Research Summary quality is preserved or improved.

#### Acceptance Criteria

- AC-002.1: Given spec-dev Phase 3, when the subagent derives queries, then the focus is on "external best practices, libraries, patterns, prior art"
- AC-002.2: Given spec-fix Phase 3, when the subagent derives queries, then the focus is on "known issues, fix patterns, pitfalls"
- AC-002.3: Given spec-refactor Phase 3, when the subagent derives queries, then the focus is on "refactoring patterns, migration guides, best practices"
- AC-002.4: Given all three commands, when the subagent prompt is checked, then it includes the domain-specific framing from the command type

#### Dependencies

- Depends on: US-001

### US-003: Documentation and quality gate

**As a** maintainer, **I want** the CHANGELOG updated and all tests passing, **so that** the refactoring is complete.

#### Acceptance Criteria

- AC-003.1: Given `CHANGELOG.md`, when checked, then BL-049 entry exists
- AC-003.2: Given `cargo test`, when run, then all tests pass
- AC-003.3: Given `cargo clippy -- -D warnings`, when run, then zero warnings

#### Dependencies

- Depends on: US-001, US-002

## Affected Modules

| Module | Layer | Nature of Change |
|--------|-------|-----------------|
| `commands/spec-dev.md` | Command | Modify: Phase 3 replaced with Task subagent invocation |
| `commands/spec-fix.md` | Command | Modify: Phase 3 replaced with Task subagent invocation |
| `commands/spec-refactor.md` | Command | Modify: Phase 3 replaced with Task subagent invocation |
| `CHANGELOG.md` | Docs | Modify: BL-049 entry |

## Constraints

- All refactoring steps are behavior-preserving (same Research Summary output)
- `cargo test` must pass (no Rust changes)
- Phase 4 (grill-me) is NOT touched
- No new agent files created
- Research Summary format (3-7 bullets) preserved
- WebSearch fallback behavior preserved (graceful degradation, no hard-fail)

## Non-Requirements

- Touching Phase 4 (grill-me) or any other phase
- Modifying `/design` or `/implement`
- Changing the Research Summary format consumed by later phases
- Creating a new agent markdown file
- Any Rust code changes

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | No port/adapter changes | Pure command markdown modification |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Feature entry | Project | `CHANGELOG.md` | Add BL-049 entry |

## Open Questions

None — all resolved during grill-me interview.
