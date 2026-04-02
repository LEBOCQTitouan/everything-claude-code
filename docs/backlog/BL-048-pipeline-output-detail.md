---
id: BL-048
title: Comprehensive output summaries for spec → design → implement pipeline
status: "implemented"
created: 2026-03-22
promoted_to: ""
tags: [spec, design, implement, output, summary, artifacts, tables, transparency]
scope: MEDIUM
target_command: /spec-refactor
---

## Optimized Prompt

The `/spec`, `/design`, and `/implement` commands each produce structured artifacts, but their conversation output is sparse and inconsistent — users cannot tell at a glance what was decided, which ACs were accepted, which adversary findings were raised, or which tests passed. Enhance all three commands to emit comprehensive, grouped, table-based summaries at phase-end, and persist those summaries into the relevant artifact files.

**Affected files (detect exact paths from project):**
- `commands/spec-dev.md`, `commands/spec-fix.md`, `commands/spec-refactor.md`
- `commands/design.md`
- `commands/implement.md`
- Artifact schemas under `docs/specs/` and `docs/design/` (frontmatter + section conventions)

**Required output detail — per phase:**

### /spec output (any variant)

Conversation summary table must include:
| Section | Required Rows |
|---------|--------------|
| Grill-me decisions | One row per question asked + answer captured |
| User stories | ID, title, status (accepted/rejected) |
| Acceptance criteria | ID, description, source US |
| Adversary findings | Dimension, verdict (PASS/FAIL), key rationale sentence |
| Artifacts persisted | File path, section written |

### /design output

Conversation summary table must include:
| Section | Required Rows |
|---------|--------------|
| Architecture decisions | Decision title, chosen option, rationale summary |
| Bounded contexts identified | Context name, responsibility |
| Adversary findings | Dimension, verdict, key rationale sentence |
| Diagrams produced | Type, file path or embed |
| Artifacts persisted | File path, section written |

### /implement output

Conversation summary table must include:
| Section | Required Rows |
|---------|--------------|
| Tasks executed | Task ID, description, RED→GREEN status |
| Tests written | File, test name, pass/fail |
| Coverage delta | Before %, after %, threshold met (yes/no) |
| Commits made | Hash (short), message |
| Artifacts persisted | File path, section written |

**Persistence requirements:**
- Each summary table must be appended to its artifact file as a `## Phase Summary` section (or equivalent named section)
- Artifact file paths follow existing conventions: `docs/specs/<work-item-slug>.md`, `docs/design/<work-item-slug>.md`
- If an artifact file already has a `## Phase Summary` section, overwrite it (not append again)
- Summary must be idempotent — running the command twice produces the same artifact state

**Acceptance criteria:**
- AC-1: After `/spec` completes, the conversation shows all five summary tables (grill-me, user stories, ACs, adversary, artifacts)
- AC-2: After `/design` completes, the conversation shows all five summary tables (decisions, bounded contexts, adversary, diagrams, artifacts)
- AC-3: After `/implement` completes, the conversation shows all five summary tables (tasks, tests, coverage, commits, artifacts)
- AC-4: Each summary is also written into the corresponding `docs/specs/` or `docs/design/` artifact file under `## Phase Summary`
- AC-5: No existing phase logic (grill-me interview, adversary review, TDD loop) is altered — only output rendering and artifact persistence are changed
- AC-6: Tables use the existing Markdown table style already present in ECC commands (pipe-delimited, header separator row)

**Out of scope:**
- Do NOT change the logic of any phase (no new steps, no reordering)
- Do NOT add new adversary dimensions or evaluation criteria
- Do NOT change artifact file naming or directory structure
- Do NOT add a new command — this is a change to existing command files only

**Verification:**
1. Run `/spec` on a real task and confirm all five tables appear in conversation output
2. Open the resulting `docs/specs/<slug>.md` and confirm `## Phase Summary` section is present with accurate data
3. Run `/design` and confirm its five tables appear; check `docs/design/<slug>.md` for the persisted section
4. Run `/implement` and confirm task/test/coverage/commit tables appear; check artifact for `## Phase Summary`
5. Run a command twice on the same work item and confirm the artifact section is overwritten, not duplicated

## Original Input

Enhance the output of the spec → design → implement pipeline to display comprehensive, detailed, organized, and grouped summaries of all outcomes. Keep table-based display. Both conversation output AND persisted to artifact files.

## Challenge Log

1. Which phases does this cover?
   - All three phases: /spec, /design, /implement

2. How much detail is "comprehensive"?
   - Full transparency: every artifact produced, every decision made, every test result, every user story, every AC, every adversary finding. Enough detail for the user to judge the output themselves.

3. Where should summaries be persisted?
   - Both conversation output AND persisted to artifact files (docs/specs/, docs/design/, etc.)

Scope confirmed as MEDIUM — structural change touching all three command files plus artifact schemas, but no logic changes.

## Related Backlog Items

- BL-029: Persist specs and designs as versioned file artifacts (implemented — establishes the artifact file convention this entry extends)
- BL-030: Persist tasks.md as trackable artifact (implemented — establish implement-phase artifact; this entry adds a Phase Summary section to it)
- BL-036: Numeric quality scores for adversary agents (open — complementary: scores would appear in the adversary findings summary table)
- BL-034: Capture grill-me decisions in work-item files (open — overlaps with AC-1 for /spec; coordinate to avoid duplicate persistence logic)
