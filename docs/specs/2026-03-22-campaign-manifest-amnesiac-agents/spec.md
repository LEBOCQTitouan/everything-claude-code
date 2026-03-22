# Spec: Campaign Manifest for Amnesiac Agents

## Problem Statement

The ECC pipeline commands (`/spec-*`, `/design`, `/implement`) rely on conversation context as the primary state carrier for intermediate artifacts: grill-me interview answers, agent analysis outputs, adversary verdict history, draft specs/designs, commit SHA accumulators, and detected toolchain commands. If a session is interrupted or context compacted mid-pipeline, this state is lost and must be regenerated. The `/implement` command was hardened with tasks.md persistence and git-log fallbacks (BL-030/BL-031), but spec and design commands remain fragile. Additionally, `scope-check.sh` reads from a never-written file path, the spec command triad duplicates ~60% of its content, and `implement.md` has grown to 477 lines. This refactoring externalizes all state to disk, making conversation a presentation layer only.

## Research Summary

- **Three-File State Management Pattern**: Localized JSON/Markdown files for tasks, logs, and rules — filesystem-based handoffs as message bus between agents (earezki.com)
- **State Externalization**: Separate persistent state (files) from bounded context (conversation window) — agents use workspace files like humans use notebooks (ndeplace/Medium)
- **Persistence Patterns for Agents**: Five patterns — handoff protocols, layered persistence, boot sequences, file-based queues, self-imposed rate limiting (dev.to)
- **Filesystem-Based Agent State**: Durable checkpoints enabling workflow resumption, failure recovery, and tasks exceeding single-session context (agentic-patterns.com)
- **Claude Code Context Challenge**: Starting every session with zero context is the fundamental limitation — explicit handoff to disk is the strongest pattern (GitHub issues #2954, #18417, #14227)
- **Campaign manifest supplements memory**: Work-item-specific state in campaign.md, cross-project preferences in auto-memory — complementary, not competing

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Campaign manifest per work item (`campaign.md`) | Single orientation file for fresh agents; indexes all artifacts + decisions | Yes |
| 2 | Incremental persistence (write draft before adversary PASS) | Eliminates chicken-and-egg fragility; draft recoverable on interruption | No |
| 3 | Toolchain commands in state.json | "Store mentally" is fragile; state.json already tracks other transient state | No |
| 4 | Extract implement.md sub-skills | 477 lines growing +45%; extract wave-analysis, wave-dispatch, progress-tracking, tasks-generation | No |
| 5 | Canonical schema skills (artifact-schemas) | Spec/design/tasks formats spread across 5+ files; single source of truth reduces shotgun surgery | No |
| 6 | Fix scope-check.sh to read artifacts.design_path | Dead code reading never-written `.claude/workflow/solution.md` | No |
| 7 | Add disk fallback to design command's 5 "from conversation context" references | Context compaction drops spec during long design sessions | No |
| 8 | Persist grill-me Q&A to campaign.md | Interview answers lost on interruption; campaign preserves decision trail | No |
| 9 | Persist adversary round counter/verdict history to campaign.md | Round counter context-only; could allow infinite retries after context clear | No |
| 10 | Persist commit SHAs to campaign.md | SHA accumulator context-only in implement; lost on compaction during long TDD loops | No |
| 11 | Reduce spec command co-change by expanding spec-pipeline-shared | 75% co-change lockstep; new shared sections reduce to reference-only changes | No |
| 12 | campaign.md lives in docs/specs/YYYY-MM-DD-<slug>/ alongside existing artifacts | Collocated with spec, design, and tasks files for discoverability | No |
| 13 | campaign.md uses Markdown tables for structured data, key-value headers for scalars | Consistent with spec/design/tasks format. Key-value format: `Key: Value` (e.g., `Status: in-progress`, `Resumption Pointer: Phase 3`) | No |
| 14 | Campaign writes centralized in spec-pipeline-shared, referenced by commands | Reduces co-change surface to one file for campaign behavior | No |
| 15 | campaign.md bootstraps in `.claude/workflow/campaign.md` at Phase 0 and moves to `docs/specs/YYYY-MM-DD-<slug>/` after spec directory creation | Spec directory slug is not known until after grill-me interview; `.claude/workflow/` is the transient staging area | No |
| 16 | Campaign.md writes use Claude's Write tool, not shell atomic writes | campaign.md is append-only during a session and written by spec-pipeline-shared (a skill, not a hook); Write tool is effectively atomic for single-file operations | No |

## User Stories

### US-001: Campaign manifest creation and schema

**As a** pipeline user resuming in a fresh session, **I want** a single `campaign.md` file per work item that indexes all artifacts and decisions, **so that** I can orient instantly without exploring multiple files.

#### Acceptance Criteria

- AC-001.0: Given a workflow initialized, when Phase 0 toolchain detection completes, then campaign.md is created in `.claude/workflow/campaign.md` with minimal schema: `Status: in-progress` header, `Concern: <concern>` header, `Started: <ISO 8601>` header, empty Artifacts table (columns: Type, Path, Status), `Toolchain:` section with test/lint/build values, empty Grill-Me Decisions table (columns: #, Question, Answer, Source), empty Adversary History table (columns: Round, Phase, Verdict, Key Findings), empty Agent Outputs table (columns: Agent, Phase, Summary), empty Commit Trail table (columns: SHA, Message, PC), `Resumption Pointer: Phase 0 complete` header
- AC-001.0b: Given campaign.md exists in `.claude/workflow/`, when the spec directory `docs/specs/YYYY-MM-DD-<slug>/` is created after adversary PASS, then campaign.md is moved to that directory and `artifacts.campaign_path` in state.json is updated to the new path
- AC-001.1: Given a `campaign.md` exists, when the first grill-me question is answered, then the `## Grill-Me Decisions` table has its first row with #, Question, Answer, and Source columns populated
- AC-001.2: Given a `campaign.md` exists, when a new grill-me answer is captured, then the Q&A pair is appended to the `## Grill-Me Decisions` table
- AC-001.3: Given a `campaign.md` exists, when an adversary review round completes, then the verdict is appended to the `## Adversary History` table with Round, Phase, Verdict, and Key Findings
- AC-001.4: Given a `campaign.md` exists, when an agent task completes, then a summary row is appended to the `## Agent Outputs` table with Agent, Phase, and Summary columns
- AC-001.5: Given a `campaign.md` exists, when a TDD commit is made, then the SHA and message are appended to the `## Commit Trail` table by the parent orchestrator (never by subagents directly)
- AC-001.6: Given a `campaign.md` exists, when a pipeline phase completes, then the `## Resumption Pointer` is updated with the current step and next action
- AC-001.7: Given a `campaign-manifest` skill exists, when validated by `ecc validate skills`, then it passes with name, description, and origin fields
- AC-001.8: Given campaign.md exists but is malformed on re-entry (missing any required section header — Status, Artifacts, Grill-Me Decisions, Adversary History, Agent Outputs, Commit Trail, Resumption Pointer — OR any table body contains zero parseable rows when rows are expected based on state.json artifact timestamps), then it is regenerated from state.json artifacts and persisted files with warning
- AC-001.9: Given campaign.md is created or moved, then `artifacts.campaign_path` in state.json is set to the current absolute file path via phase-transition.sh
- AC-001.10: Given parallel TDD waves are executing, then campaign.md writes (Commit Trail, Agent Outputs) are performed by the parent orchestrator after subagent completion, never by subagents directly

#### Dependencies

- Depends on: none

### US-002: Toolchain command persistence

**As a** spec command re-entering after context loss, **I want** detected test/lint/build commands stored in `state.json`, **so that** I don't lose toolchain information on session interruption.

#### Acceptance Criteria

- AC-002.1: Given `workflow-init.sh` creates a new `state.json`, then it includes a `toolchain` object with `test`, `lint`, `build` fields (initially null)
- AC-002.2: Given spec-* Phase 0 detects toolchain commands, when detection completes, then the commands are written to `state.json.toolchain` via `toolchain-persist.sh`
- AC-002.3: Given a spec command re-enters with toolchain already in state.json, when Phase 0 runs, then it reads toolchain from state.json instead of re-detecting
- AC-002.4: Given no spec command contains "Store these commands mentally", then the phrase is fully eliminated from all command files
- AC-002.5: Given jq is unavailable, when `workflow-init.sh` runs, then the toolchain field is omitted from state.json and detection falls back to per-session auto-detection
- AC-002.6: Given jq is unavailable, when `toolchain-persist.sh` runs, then toolchain persistence is skipped with warning "jq unavailable: toolchain not persisted"
- AC-002.7: Given spec-pipeline-shared Project Detection section exists, then it says "Persist detected commands to state.json via toolchain-persist.sh" instead of "Store detected commands for use in spec constraints and pass conditions"

#### Dependencies

- Depends on: none

### US-003: Incremental spec persistence

**As a** user whose session may be interrupted during `/spec-*`, **I want** the draft spec written to disk before adversary review, **so that** the spec is recoverable even if the adversary dispatch fails or context is lost.

#### Acceptance Criteria

- AC-003.1: Given a spec command reaches the spec output phase, when the spec is written, then it is saved as `spec-draft.md` in the campaign.md directory (initially `.claude/workflow/`, later `docs/specs/YYYY-MM-DD-<slug>/`) before adversary dispatch
- AC-003.2: Given the adversary returns PASS, when the spec is persisted, then `spec-draft.md` is renamed/overwritten to `spec.md` in the final spec directory
- AC-003.3: Given spec-pipeline-shared contains a "Draft Spec Persistence" section, when referenced by all 3 spec commands, then the instruction is shared (not inlined 3 times)

#### Dependencies

- Depends on: US-001

### US-004: Design command disk fallback

**As a** design command executing in a long session, **I want** all "from conversation context" references to have disk fallback, **so that** context compaction doesn't silently drop the spec.

#### Acceptance Criteria

- AC-004.1: Given design.md Phase 1, when spec is needed, then it reads from `artifacts.spec_path` if not in conversation context
- AC-004.2: Given design.md Phase 5, when `## E2E Boundaries Affected` is needed, then it reads from the spec file on disk if not in context
- AC-004.3: Given design.md Phase 6, when `## Doc Impact Assessment` is needed, then it reads from the spec file on disk if not in context
- AC-004.4: Given design.md Phase 10 (adversarial review), when spec and solution are needed, then both are read from disk
- AC-004.5: Given design completes, when campaign.md is updated, then the Artifacts table shows design path with status "passed"
- AC-004.6: Given design.md Phase 7 (AC Coverage Verification), when AC identifiers are needed, then they are read from the spec file on disk via `artifacts.spec_path`

#### Dependencies

- Depends on: US-001

### US-005: scope-check.sh fix

**As a** pipeline user finishing `/implement`, **I want** scope-check.sh to actually verify file scope, **so that** scope creep during implementation is detected.

#### Acceptance Criteria

- AC-005.1: Given scope-check.sh runs, when checking file scope, then it reads design path from `artifacts.design_path` in state.json (not hardcoded `.claude/workflow/solution.md`)
- AC-005.2: Given `artifacts.design_path` is not set in state.json, when scope-check runs, then it falls back to the legacy `.claude/workflow/solution.md` path
- AC-005.3: Given scope-check.sh has been fixed, when the old solution.md path does not exist, then it still functions correctly by using the design_path

#### Dependencies

- Depends on: none

### US-006: Implement command fixes and decomposition

**As a** developer maintaining implement.md, **I want** the file decomposed into focused sub-skills, **so that** it stays under 350 lines and each concern is independently maintainable.

#### Acceptance Criteria

- AC-006.1: Given implement.md currently has wave analysis content inline, when extracted to `skills/wave-analysis/SKILL.md`, then implement.md references the skill instead
- AC-006.2: Given implement.md currently has wave dispatch content inline, when extracted to `skills/wave-dispatch/SKILL.md`, then implement.md references the skill instead
- AC-006.3: Given implement.md currently has progress tracking content inline, when extracted to `skills/progress-tracking/SKILL.md`, then implement.md references the skill instead
- AC-006.4: Given implement.md currently has tasks.md generation content inline, when extracted to `skills/tasks-generation/SKILL.md`, then implement.md references the skill instead
- AC-006.5: Given all extractions are complete, then implement.md is under 350 lines
- AC-006.6: Given implement.md Phase 3, when a commit is made, then the SHA is appended to campaign.md's `## Commit Trail` by the parent orchestrator
- AC-006.7: Given implement.md Phase 5, when code review completes, then findings are appended to campaign.md's `## Agent Outputs`
- AC-006.8: Given implement.md Phase 0 re-entry, then campaign.md is read for orientation context (toolchain, decisions, commit trail)

#### Dependencies

- Depends on: US-001

### US-007: Canonical schema skills

**As a** developer modifying artifact formats, **I want** a single source of truth for spec.md, design.md, tasks.md, and state.json schemas, **so that** format changes don't require shotgun surgery across 5+ files.

#### Acceptance Criteria

- AC-007.1: Given an `artifact-schemas` skill exists, then it defines the schema for spec.md, design.md, tasks.md, state.json, and campaign.md
- AC-007.2: Given the skill has correct frontmatter (name, description, origin), then `ecc validate skills` passes
- AC-007.3: Given commands reference the artifact-schemas skill for format definitions, then no command inlines the full schema template without a reference

#### Dependencies

- Depends on: none

### US-008: Spec command co-change reduction

**As a** developer modifying shared pipeline behavior, **I want** new behavioral instructions in `spec-pipeline-shared/SKILL.md`, **so that** changes to grill-me persistence, draft spec writing, and adversary tracking only need editing in one file.

#### Acceptance Criteria

- AC-008.1: Given spec-pipeline-shared contains sections for Campaign Init, Grill-Me Disk Persistence, Draft Spec Persistence, Adversary History Tracking, and Agent Output Tracking, then all 3 spec commands reference these sections
- AC-008.2: Given the phrase "Store these commands mentally" is removed from all spec commands, then toolchain persistence uses the shared instruction
- AC-008.3: Given campaign-related instructions, when grepping spec-dev.md, spec-fix.md, and spec-refactor.md for inline campaign.md write logic, then zero matches found — all campaign writes are in spec-pipeline-shared

#### Dependencies

- Depends on: US-001, US-002, US-003

### US-009: Strategic compact update

**As a** user deciding whether to compact context, **I want** the strategic-compact skill to reflect that campaign.md externalizes all state, **so that** I know compaction is always safe during pipeline execution.

#### Acceptance Criteria

- AC-009.1: Given strategic-compact SKILL.md exists, when updated, then it mentions campaign manifest as state that survives compaction
- AC-009.2: Given strategic-compact skill's Compaction Decision Guide table, then it contains a row with scenario "Mid-pipeline (with campaign)", recommendation "Yes", and rationale mentioning campaign manifest externalizing all state

#### Dependencies

- Depends on: US-001

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| commands/spec-dev.md | CLI/orchestration | Add shared refs for campaign persistence, remove "Store mentally" |
| commands/spec-fix.md | CLI/orchestration | Same as spec-dev |
| commands/spec-refactor.md | CLI/orchestration | Same as spec-dev |
| commands/design.md | CLI/orchestration | Add disk fallbacks, campaign updates |
| commands/implement.md | CLI/orchestration | Extract sub-skills, add campaign persistence |
| .claude/hooks/scope-check.sh | Infrastructure | Fix design_path reading |
| .claude/hooks/workflow-init.sh | Infrastructure | Add toolchain + campaign_path to state.json |
| .claude/hooks/phase-transition.sh | Infrastructure | Support campaign_path artifact |
| .claude/hooks/toolchain-persist.sh | Infrastructure (new) | Write toolchain to state.json |
| skills/spec-pipeline-shared/SKILL.md | Shared skill | Add 5 new sections for campaign behaviors |
| skills/campaign-manifest/SKILL.md | Shared skill (new) | Campaign schema definition |
| skills/artifact-schemas/SKILL.md | Shared skill (new) | Canonical artifact format definitions |
| skills/wave-analysis/SKILL.md | Shared skill (new) | Extracted from implement.md |
| skills/wave-dispatch/SKILL.md | Shared skill (new) | Extracted from implement.md |
| skills/progress-tracking/SKILL.md | Shared skill (new) | Extracted from implement.md |
| skills/tasks-generation/SKILL.md | Shared skill (new) | Extracted from implement.md |
| skills/strategic-compact/SKILL.md | Shared skill | Add campaign awareness |

## Constraints

- All refactoring steps must be behavior-preserving — existing tests pass after each group
- No changes to Rust crates — pure Markdown/shell refactoring
- Campaign manifest supplements (not replaces) existing memory system
- Existing spec directory pattern (spec.md, design.md, tasks.md) unchanged — campaign.md added alongside
- Shell hooks must maintain atomic write pattern (mktemp + mv)
- All commands must maintain narrative conventions
- implement.md must be under 350 lines after extraction
- Each extracted skill committed before corresponding content removed from implement.md
- Toolchain field in state.json requires jq; if jq unavailable, toolchain omitted with fallback to per-session detection

## Non-Requirements

- Rust code changes (no crate modifications)
- Changes to audit commands (out of scope)
- E2E testing of the full pipeline (manual /ecc-test-mode verification deferred)
- Schema validation tooling (canonical schemas are reference-only, no runtime enforcement)
- Memory system changes (campaign supplements memory, doesn't modify it)
- Context clear gates (BL-054) and graceful mid-session exit (BL-055) — separate backlog items

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Hooks (shell) | Modify workflow-init.sh, phase-transition.sh, scope-check.sh; create toolchain-persist.sh | Hook execution during pipeline phases |
| Commands (Markdown) | Modify 5 commands, create 6 skills | Command interpretation during pipeline |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Architecture decision | ADR | docs/adr/0013-campaign-manifest-convention.md | Create new ADR |
| Domain terms | Glossary | docs/domain/glossary.md | Add "Campaign Manifest", "Resumption Pointer" |
| Feature entry | Project | CHANGELOG.md | Add BL-035 entry |
| Backlog status | Project | docs/backlog/BACKLOG.md | Update BL-035 to implemented |
| State schema | Internal | .claude/workflow/README.md | Add toolchain field to state.json docs |

## Open Questions

None — all questions resolved during grill-me interview and adversarial review.
