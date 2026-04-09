# Spec: ASCII Diagram Doc-Comments Convention

## Problem Statement

Rust doc-comments across ECC (and any codebase Claude works on) are prose-only. Complex flows — state machines, dispatch chains, multi-step resolution logic — require reading the full implementation to understand behavior. Named design patterns (Repository, Strategy, State Machine, Aggregate Root) are implemented but unlabeled, making it hard for developers to recognize architectural intent. A reusable skill defining when and how to add ASCII diagrams and pattern references, enforced via code-reviewer (HIGH) and audit-code (MEDIUM), establishes this as a living convention rather than a one-time cleanup.

## Research Summary

- Rust doc-comments support fenced code blocks (````text`) that render as `<pre>` in `cargo doc` — ideal for ASCII diagrams
- One existing precedent in ECC: `ecc-domain/src/claw/turn.rs` uses ````text` for a markdown format example
- Zero existing ASCII diagrams or pattern annotations in the entire ECC codebase (confirmed by grep)
- 9 confirmed design patterns in ECC: Repository, Strategy, State Machine (x2), Command, Aggregate Root, Value Object, Factory, Anti-Corruption Layer, Ports Bundle
- `#![warn(missing_docs)]` enabled in `ecc-ports` — doc presence already enforced on public items

## Decisions Made

| # | Decision | Rationale | ADR? |
|---|----------|-----------|------|
| 1 | Reusable skill for any repo | User wants convention on every repo Claude works on | No |
| 2 | Enforced: code-reviewer HIGH, audit-code MEDIUM | Convention must be living, not one-time | No |
| 3 | Skill + convention first, ECC sweep separate | Clean separation — define then apply | No |
| 4 | `# Pattern` section header with `Name [Source]` | Matches rustdoc section rendering, discoverable | No |
| 5 | Heuristic thresholds + comprehension importance | 3+ states/branches/types OR key to project understanding | No |
| 6 | Changed files only in audit | Avoids noise; full sweep as separate backlog | No |
| 7 | Fenced ````text` blocks for diagrams | Matches existing precedent, renders in cargo doc | No |
| 8 | 80-column max, box-drawing with `+--+`, `\|`, `-->` | Universal monospace rendering, no unicode dependency | No |

## User Stories

### US-001: ASCII Doc-Comment Skill File

**As a** Claude Code user on any Rust project, **I want** a skill defining ASCII diagram and pattern reference conventions for doc-comments, **so that** complex code is visually documented with consistent style.

#### Acceptance Criteria

- AC-001.1: `skills/ascii-doc-diagrams/SKILL.md` exists with `name: ascii-doc-diagrams`, `description`, `origin: ECC`
- AC-001.2: Skill defines 3 diagram types: state transition (`[state] --> [state]`), flow/decision (`[condition?] --Y/N-->`), composition/box (`+--+` containment)
- AC-001.3: Skill defines pattern annotation format: `# Pattern` doc-comment section with `PatternName [Source]` where Source is GoF, DDD, Hexagonal Architecture, or Rust Idiom. Multiple sources allowed (e.g., `Repository [DDD] / Port [Hexagonal Architecture]`)
- AC-001.4: Skill defines eligibility criteria with concrete thresholds: public functions with 3+ decision branches → flow diagram, enums with 3+ variants used as state machines → state diagram, public structs composing 3+ domain types → composition diagram (domain type = type defined in current crate or project domain crates, excludes std/third-party/primitives), any item referenced in ARCHITECTURE.md or onboarding docs → pattern annotation, items with 5+ callers → pattern annotation
- AC-001.5: Skill defines ASCII style: fenced ````text` blocks, 80-column max, `+--+`/`|`/`-->` characters only (no `==>` — single arrow style for simplicity)
- AC-001.6: Skill body under 500 words (excluding frontmatter)
- AC-001.7: `ecc validate skills` passes

#### Dependencies

- Depends on: none

### US-002: Code-Reviewer Enforcement

**As a** developer writing new code, **I want** the code-reviewer to flag missing diagrams and pattern annotations on eligible code as HIGH findings, **so that** the convention is enforced during implementation.

#### Acceptance Criteria

- AC-002.1: `agents/code-reviewer.md` frontmatter `skills` list includes `ascii-doc-diagrams`
- AC-002.2: Code-reviewer instructions reference the skill for diagram/pattern checks on new or changed eligible code (eligibility as defined in AC-001.4)
- AC-002.3: Missing diagrams on eligible code (per AC-001.4) produce HIGH findings (blocking)

#### Dependencies

- Depends on: US-001

### US-003: Audit-Code Integration

**As a** project maintainer running audits, **I want** audit-code to flag missing diagrams on changed files as MEDIUM findings, **so that** existing code is gradually brought up to convention.

#### Acceptance Criteria

- AC-003.1: `commands/audit-code.md` references `ascii-doc-diagrams` skill in its instructions for the code-reviewer agent it dispatches
- AC-003.2: Missing diagrams on eligible changed files flagged as MEDIUM findings in audit output
- AC-003.3: Audit scope limited to files in the git diff (not full codebase)
- AC-003.4: Eligibility criteria for audit matches the skill's definition (AC-001.4) — single source of truth

#### Dependencies

- Depends on: US-001

### US-004: Backlog Item for Full ECC Sweep

**As a** project maintainer, **I want** a backlog entry for a full ASCII diagram sweep of all 9 ECC crates, **so that** existing code is documented when capacity allows.

#### Acceptance Criteria

- AC-004.1: Backlog entry created at `docs/backlog/BL-NNN-ascii-diagram-full-sweep.md` with status `open`
- AC-004.2: Entry references this spec and lists all 9 crates as scope

#### Dependencies

- Depends on: US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `skills/ascii-doc-diagrams/SKILL.md` | Skill | New — convention definition |
| `agents/code-reviewer.md` | Agent | Add `ascii-doc-diagrams` to skills list |
| `commands/audit-code.md` | Command | Add diagram convention reference for code-reviewer dispatch |
| `docs/backlog/BL-NNN-*.md` | Backlog | New entry for full sweep |

## Constraints

- Skill must be under 500 words
- Diagrams must render correctly in `cargo doc` (fenced ````text` blocks)
- 80-column max for all diagrams
- No Rust source code changes in this spec
- No new agents, tools, or crate dependencies

## Non-Requirements

- Full ECC codebase sweep (separate backlog item)
- Rust lint or clippy rule for diagram enforcement
- Mermaid or SVG diagrams (ASCII only — universal rendering)
- Automated diagram generation from code structure
- Diagrams on macro-generated output (convention applies to source-level items only)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | Markdown files only | No E2E impact |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CHANGELOG | root | CHANGELOG.md | Add convention entry |

## Open Questions

None — all resolved during grill-me interview.
