# Spec: BL-125 Token Optimization Wave 2

## Problem Statement

ECC loads ~1,300 lines of baseline context per session. BL-121 audit Axis 3 identified 5 mechanical cleanup opportunities: TodoWrite boilerplate duplicated across 25 agents (~1,000 words), CLAUDE.md CLI reference bloat (77 lines when full listing exists elsewhere), possibly-loaded language rules for non-project languages (~2,500 lines), and two oversized common rules files. Combined: ~5,000+ tokens of waste per session.

## Research Summary

- BL-121 audit pre-validated all 5 findings with component-auditor agent
- Install pipeline has `applies-to` rule filtering but no agent frontmatter expansion — new expansion step needed
- 25 agents use identical TodoWrite conditional pattern (~40 words each)
- 14 language rule dirs exist; only `applies-to` drives install filtering, `paths:` is unused
- CLAUDE.md CLI section is 77 lines; `docs/commands-reference.md` already has full listing
- No runtime hook changes needed — install-time expansion is sufficient

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | All 5 items in scope | Independent, LOW complexity, pre-validated | No |
| 2 | TodoWrite: install-time expansion via `tracking: todowrite` frontmatter | User chose install pipeline over runtime hook | No |
| 3 | Independent commits per item | Easier review and partial revert | No |
| 4 | Pure cleanup except TodoWrite expansion | No behavioral changes beyond install expansion | No |
| 5 | Language rules: verify-only | If working, no fix. If broken, file bug. | No |

## User Stories

### US-001: Extract TodoWrite boilerplate to frontmatter

**As a** maintainer, **I want** TodoWrite boilerplate extracted from 25 agents to a `tracking: todowrite` frontmatter field with install-time expansion, **so that** agent files are smaller and the pattern is centralized.

#### Acceptance Criteria

- AC-001.0: The canonical TodoWrite block is defined as the following text, stored in `agents/.templates/todowrite-block.md`:
  ```
  > **Tracking**: Create a TodoWrite checklist for this workflow. If TodoWrite is unavailable, proceed without tracking — the workflow executes identically.
  
  Mark each item complete as the step finishes.
  ```
  The per-agent TodoWrite item list (specific step names) remains inline in each agent file — only the guard text and tracking instruction are centralized.
- AC-001.1: 25 agent files have `tracking: todowrite` in frontmatter and the guard text + tracking instruction removed (per-agent step lists remain)
- AC-001.2: Install pipeline adds a post-copy expansion step: after copying agent files, scan for `tracking: todowrite` in frontmatter. If found, read `agents/.templates/todowrite-block.md` and insert its content before the first `TodoWrite items:` line (or at end of file if no items list). Implementation: new function `expand_tracking_field()` called from `merge_directory()` in the install orchestrator.
- AC-001.3: `ecc validate agents` passes after changes
- AC-001.4: Integration test: use InMemoryFileSystem with a test agent containing `tracking: todowrite`, run expansion, verify output contains the canonical block text

#### Dependencies

- Depends on: none

### US-002: Slim CLAUDE.md CLI reference

**As a** developer, **I want** CLAUDE.md CLI Commands section reduced to top-10 most-used commands with a pointer to the full reference, **so that** per-session context is ~60 lines smaller.

#### Acceptance Criteria

- AC-002.0: The top-10 commands retained in CLAUDE.md are (by frequency of use in the spec-driven pipeline): `cargo test`, `cargo clippy`, `cargo build`, `ecc workflow init|transition|status`, `ecc validate`, `ecc hook`, `ecc backlog`, `ecc worktree`, `ecc status`, `ecc dev`
- AC-002.1: CLAUDE.md CLI Commands section contains at most 15 lines (the 10 commands above + header + pointer: "Full CLI reference: `docs/commands-reference.md`")
- AC-002.2: `docs/commands-reference.md` already contains the full CLI listing (verify existence, do not create)

#### Dependencies

- Depends on: none

### US-003: Verify language rule conditional loading

**As a** maintainer, **I want** confirmation that non-Rust language rules are excluded during `ecc install` in a Rust-only project, **so that** ~2,500 lines aren't wasted.

#### Acceptance Criteria

- AC-003.1: Run `ecc install` in a Rust-only project, verify only Rust + common + ECC rules are installed to `~/.claude/rules/`
- AC-003.2: If non-matching rules ARE installed, document the bug as a new backlog item for follow-up fix
- AC-003.3: Document the verification result in the spec revision or implement-done.md

#### Dependencies

- Depends on: none

### US-004: Trim performance.md

**As a** developer, **I want** rules/common/performance.md trimmed to the model routing table and thinking effort tiers only, **so that** context management prose doesn't waste tokens.

#### Acceptance Criteria

- AC-004.1: performance.md reduced to at most 30 lines (model routing table + thinking effort tiers)
- AC-004.2: Removed content either moved to docs/ or deleted if redundant with existing docs
- AC-004.3: `ecc validate rules` passes

#### Dependencies

- Depends on: none

### US-005: Trim agents.md

**As a** developer, **I want** rules/common/agents.md trimmed to a 10-15 line summary with pointer to system prompt agent listing, **so that** the listing isn't duplicated.

#### Acceptance Criteria

- AC-005.1: agents.md reduced to at most 15 lines
- AC-005.2: `ecc validate rules` passes

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| agents/*.md (25 files) | instruction | Remove boilerplate, add frontmatter field |
| crates/ecc-app/src/install/ | app (Rust) | Add TodoWrite expansion during agent file copy |
| CLAUDE.md | project | Trim CLI section to top-10 |
| rules/common/performance.md | rule | Trim to routing table + tiers |
| rules/common/agents.md | rule | Trim to summary |

## Constraints

- `ecc validate agents` must pass after all changes
- `ecc validate rules` must pass after all changes
- Installed agents must still contain TodoWrite instructions (behavioral equivalence)
- No runtime code changes beyond install pipeline

## Non-Requirements

- Runtime hook changes for TodoWrite injection
- Language rule loading fix (verification only; fix is separate backlog item if needed)
- New documentation files
- Formatting/linting of non-modified files

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Install pipeline | New expansion step | Agent files post-install must be verified |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| CLI trim | CLAUDE.md | CLI Commands section | Reduce to top-10 + pointer |
| Convention | rules/ecc/development.md | Agent Conventions section | Document `tracking: todowrite` convention |
| Changelog | CHANGELOG.md | Unreleased | Add BL-125 entry |

## Open Questions

None — all resolved during grill-me interview.
