# Spec: /commit Slash Command (BL-063)

## Problem Statement

Developers using ECC must manually ask Claude to commit after changes, breaking workflow rhythm. There is no standardized commit command that enforces ECC's conventional commits format, atomic commit discipline, and pre-flight build/test gates. The native `commit-commands:commit` skill exists but lacks ECC-specific enforcement (atomic commit warnings, workflow-state awareness, toolchain detection from state.json).

## Research Summary

- Conventional commits format already codified in `rules/common/git-workflow.md`
- Native `commit-commands:commit` skill exists in Claude Code but lacks ECC enforcement
- `/verify` and `/build-fix` have toolchain detection patterns (Cargo.toml, package.json, go.mod)
- Prior audit [SELF-004] requires all commands to include `allowed-tools` in frontmatter
- BL-059 (auto-commit backlog edits) is an adjacent pattern that could eventually delegate to `/commit`

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | All logic in commands/commit.md, no skill extraction | YAGNI — extract if /implement reuse needed later | No |
| 2 | git status as primary staging signal | Reliable across all contexts; session context as enrichment only | No |
| 3 | No force-proceed on pre-flight failure | Aligns with git-workflow.md mandatory rules | No |
| 4 | Warn during active /implement workflow | Prevents breaking TDD commit trail | No |
| 5 | Self-contained toolchain detection | Read state.json first, fallback to file-based detection | No |
| 6 | Infer scope from directory when unambiguous | Better commit messages with minimal complexity | No |

## User Stories

### US-001: Nothing-to-Commit Detection

**As a** developer, **I want** `/commit` to detect a clean working tree and inform me gracefully, **so that** I don't see confusing errors when there is nothing to commit.

#### Acceptance Criteria

- AC-001.1: Given a clean working tree (no staged, modified, or untracked files), when I run `/commit`, then the command displays "Nothing to commit" and exits without error.
- AC-001.2: Given only .gitignore-covered untracked files, when I run `/commit`, then the command treats the tree as clean.
- AC-001.3: Given unresolved merge conflicts in the working tree, when I run `/commit`, then the command detects the conflict state, displays the conflicting files, and blocks the commit.

#### Dependencies

- Depends on: none

### US-002: Intelligent File Staging

**As a** developer, **I want** `/commit` to automatically stage the right files, **so that** I don't have to manually `git add` before committing.

#### Acceptance Criteria

- AC-002.1: Given files already staged via `git add`, when I run `/commit`, then the command respects the existing staging and does not re-stage or unstage.
- AC-002.2: Given unstaged modified files and session context available, when I run `/commit`, then the command uses session context to identify session-relevant files and proposes staging them.
- AC-002.3: Given unstaged modified files with no session context, when I run `/commit`, then the command uses `git status` to propose a staging hypothesis for user confirmation.
- AC-002.4: Given a staging proposal, when the user confirms or adjusts, then the confirmed files are staged before proceeding.

#### Dependencies

- Depends on: US-001

### US-003: Atomic Commit Enforcement

**As a** developer, **I want** `/commit` to warn me when the diff spans multiple unrelated concerns, **so that** I maintain atomic commit discipline.

#### Acceptance Criteria

- AC-003.1: Given a staged diff touching a single logical concern, when I run `/commit`, then no warning is shown.
- AC-003.2: Given a staged diff touching multiple unrelated concerns, when the command detects this, then it warns and suggests splitting.
- AC-003.3: Given a multi-concern warning, when the user confirms to proceed anyway, then the commit proceeds.
- AC-003.4: Given a multi-concern warning, when the user chooses to split, then the command guides unstaging files for separate commits.

#### Dependencies

- Depends on: US-002

### US-004: Conventional Commit Message Generation

**As a** developer, **I want** `/commit` to auto-generate a conventional commit message from the diff, **so that** I get correctly formatted messages without manual effort.

#### Acceptance Criteria

- AC-004.1: Given a staged diff, when the command generates a message, then it follows `<type>[(<scope>)]: <description>` with optional body.
- AC-004.2: Given the type selection, then type is one of: feat, fix, refactor, docs, test, chore, perf, ci.
- AC-004.3: Given all changes in a single directory subtree, when scope is inferred, then it reflects the directory (e.g., `domain` for `crates/ecc-domain/`).
- AC-004.4: Given changes spanning multiple directories, when scope would be ambiguous, then scope is omitted.
- AC-004.5: Given a generated message, when shown to the user, then the user can accept, edit, or reject before commit executes.

#### Dependencies

- Depends on: US-003

### US-005: Build and Test Pre-Flight

**As a** developer, **I want** `/commit` to run build and tests before committing, **so that** I never commit broken code.

#### Acceptance Criteria

- AC-005.1: Given `.claude/workflow/state.json` has toolchain.test and toolchain.build set, when pre-flight runs, then those commands are used.
- AC-005.2: Given no state.json toolchain, when the project has Cargo.toml, then `cargo build` and `cargo test` are used.
- AC-005.3: Given no state.json toolchain, when the project has package.json, then the appropriate npm/yarn commands are used.
- AC-005.4: Given pre-flight passes, then the commit proceeds to message confirmation.
- AC-005.5: Given pre-flight fails, then the commit is blocked, failures are displayed, and the user is told to fix the issues before committing.

#### Dependencies

- Depends on: US-004

### US-006: Workflow State Awareness

**As a** developer, **I want** `/commit` to warn me if an /implement workflow is active, **so that** I don't break the TDD commit trail.

#### Acceptance Criteria

- AC-006.1: Given `.claude/workflow/state.json` phase is "implement", when I run `/commit`, then a warning is displayed explaining that commits are managed by /implement.
- AC-006.2: Given the workflow warning, when the user confirms to proceed, then the commit continues normally.
- AC-006.3: Given state.json does not exist or phase is not "implement", when I run `/commit`, then no warning is shown.

#### Dependencies

- Depends on: none

### US-007: Command File with Proper Frontmatter

**As an** ECC maintainer, **I want** the command file to follow ECC conventions, **so that** it integrates with the command system.

#### Acceptance Criteria

- AC-007.1: Given `commands/commit.md`, when validated with `ecc validate commands`, then it passes.
- AC-007.2: Given the command file, when inspected, then it has YAML frontmatter with `description` and `allowed-tools`.
- AC-007.3: Given `/commit` is invoked with `$ARGUMENTS`, when the argument is a commit message string, then it is used as the initial message (user can still edit). When no arguments are provided, the message is auto-generated from the diff.

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `commands/commit.md` | Content (outside Rust hexagon) | New command file |

## Constraints

- Must include `allowed-tools` in frontmatter (audit SELF-004)
- Must follow conventional commits format from git-workflow.md
- Pre-flight failure always blocks — no force-proceed
- No Rust code changes
- No modification to existing command files

## Non-Requirements

- Push/PR creation (separate command)
- Branch management
- Rust code changes
- Modifying existing commands
- Skill extraction (deferred)
- Force-proceed on pre-flight failure

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | N/A | Content layer only |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New command | Minor | CLAUDE.md | Add /commit to Slash Commands section |
| Content | Minor | CHANGELOG.md | Add entry |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | What is out of scope? | Push/PR, branch mgmt, Rust changes, existing cmd changes, skill extraction | Recommended |
| 2 | How should staging work? | git status primary, session context as enrichment | Recommended |
| 3 | Force-proceed on pre-flight failure? | Removed entirely — always blocks | User |
| 4 | Workflow state awareness? | Warn during active /implement | Recommended |
| 5 | Pre-flight detection method? | Self-contained: state.json first, file-based fallback | Recommended |
| 6 | Scope prefix inference? | Infer from directory when unambiguous, omit otherwise | Recommended |
| 7 | Security implications? | None — local git commit only | Recommended |
| 8 | Breaking changes / glossary / ADR? | None needed for all three | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Nothing-to-Commit Detection | 3 | none |
| US-002 | Intelligent File Staging | 4 | US-001 |
| US-003 | Atomic Commit Enforcement | 4 | US-002 |
| US-004 | Conventional Commit Message Generation | 5 | US-003 |
| US-005 | Build and Test Pre-Flight | 5 | US-004 |
| US-006 | Workflow State Awareness | 3 | none |
| US-007 | Command File with Proper Frontmatter | 3 | none |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Clean tree → "Nothing to commit" | US-001 |
| AC-001.2 | .gitignore-only untracked → clean | US-001 |
| AC-001.3 | Merge conflicts → block | US-001 |
| AC-002.1 | Respect existing staging | US-002 |
| AC-002.2 | Session context enrichment for staging | US-002 |
| AC-002.3 | git status fallback hypothesis | US-002 |
| AC-002.4 | User confirms/adjusts staging | US-002 |
| AC-003.1 | Single concern → no warning | US-003 |
| AC-003.2 | Multi-concern → warn + suggest split | US-003 |
| AC-003.3 | User confirms multi-concern → proceed | US-003 |
| AC-003.4 | User splits → guide unstaging | US-003 |
| AC-004.1 | Message format: type[(scope)]: desc | US-004 |
| AC-004.2 | Valid type tokens | US-004 |
| AC-004.3 | Scope inferred from directory | US-004 |
| AC-004.4 | Scope omitted when ambiguous | US-004 |
| AC-004.5 | User accept/edit/reject message | US-004 |
| AC-005.1 | state.json toolchain → use it | US-005 |
| AC-005.2 | Cargo.toml fallback | US-005 |
| AC-005.3 | package.json fallback | US-005 |
| AC-005.4 | Pre-flight pass → proceed | US-005 |
| AC-005.5 | Pre-flight fail → block | US-005 |
| AC-006.1 | Implement phase → warn | US-006 |
| AC-006.2 | User confirms → proceed | US-006 |
| AC-006.3 | No implement phase → no warning | US-006 |
| AC-007.1 | ecc validate passes | US-007 |
| AC-007.2 | Proper frontmatter | US-007 |
| AC-007.3 | $ARGUMENTS as initial message | US-007 |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| Ambiguity | PASS (round 2) | AC-001.3 for conflicts, AC-007.3 for arguments, explicit paths |
| Edge cases | PASS (round 2) | Merge conflict detection added |
| Scope | PASS | Well-bounded, thorough non-requirements |
| Dependencies | PASS (round 2) | Explicit state.json paths |
| Testability | PASS | ACs follow Given/When/Then |
| Decisions | PASS | All 6 decisions justified |
| Rollback | PASS | Single new file, trivial revert |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-27-commit-slash-command/spec.md | Full spec + phase summary |
