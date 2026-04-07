# Spec Adversary Report

## Summary
Verdict: PASS (avg: 82/100)
Rounds: 1

## Dimension Results
| # | Dimension | Score | Verdict | Critical Findings |
|---|-----------|-------|---------|-------------------|
| 1 | Ambiguity | 78 | PASS | AC-002.1 "6-step flow" step names need exact match to skill body; AC-003.1 "max 40 chars" unclear if slug or full filename |
| 2 | Edge Cases | 72 | PASS | Missing: slug collision, unicode in feature name, empty interview answers |
| 3 | Scope Creep Risk | 90 | PASS | Tight scope, explicit non-requirements, well-bounded |
| 4 | Dependency Gaps | 95 | PASS | DAG is linear and correct, no circular deps |
| 5 | Testability | 75 | PASS | Skill ACs (US-001/002/003) are behavioral — testable only via manual invocation or ecc validate |
| 6 | Decision Completeness | 80 | PASS | Missing decision on PRD template versioning strategy |
| 7 | Rollback & Failure | 85 | PASS | Two-file change, trivially revertible |

## Detailed Findings

### 1. Ambiguity

- **Finding**: AC-003.1 says "using kebab-case slug (max 40 chars)" but does not clarify whether the 40-char limit applies to the slug alone or the full filename `{slug}-prd.md`. The slug `my-very-long-feature-name-that-keeps-going` is 43 chars, but `my-very-long-feature-name-that-keeps-going-prd.md` is 51. An engineer reading this could enforce either boundary.
- **Evidence**: `AC-003.1: PRD written to docs/prds/{feature}-prd.md using kebab-case slug (max 40 chars)`
- **Recommendation**: Clarify: "slug portion (before `-prd.md`) must not exceed 40 characters." Minor — does not warrant FAIL because the intent is obvious enough from context and the skill is Markdown guidance, not compiled code enforcing a constraint.

- **Finding**: AC-002.1 prescribes "6-step flow" with specific step names, but step 5 is "module sketch" while the backlog item BL-012 originally described a 7-step flow including "check module design with user" as a separate step. The spec collapsed steps 5-6 from the backlog into one. This is fine as a design decision, but the spec does not document this compression anywhere.
- **Evidence**: BL-012 action: "(5) Sketch major modules... (6) Check module design with user. (7) Write PRD" vs spec AC-002.1: "(5) module sketch, (6) write PRD"
- **Recommendation**: No action needed — the Decision table already captures "Defer deep module analysis to /design" which explains the compression. The step count mismatch is resolved.

- **Finding**: AC-002.2 says "Each step uses AskUserQuestion for user input (one question at a time)" — but steps like (2) codebase exploration and (6) write PRD may not need user input at all. "Each step" is over-broad.
- **Evidence**: `AC-002.2: Each step uses AskUserQuestion for user input (one question at a time)`
- **Recommendation**: Reword to "Steps requiring user input use AskUserQuestion (one question at a time)." Non-blocking — the skill implementer will naturally skip AskUserQuestion for codebase-reading steps.

### 2. Edge Cases

- **Finding**: No AC addresses what happens when the feature slug contains characters outside `[a-z0-9-]`. The spec says "kebab-case" but does not specify sanitization of user input (e.g., user types "Add OAuth 2.0 support!" which must be slugified).
- **Evidence**: `AC-003.1: PRD written to docs/prds/{feature}-prd.md using kebab-case slug (max 40 chars)`
- **Recommendation**: Non-blocking for a skill file — the skill body will describe slugification. If this were compiled code, this would be a FAIL.

- **Finding**: No AC addresses slug collision. If `docs/prds/oauth-support-prd.md` already exists from a different feature, AC-003.4 handles the "same path" case (overwrite/append), but two different features could theoretically produce the same slug.
- **Evidence**: `AC-003.4: If PRD already exists at path, skill asks to overwrite or append revision`
- **Recommendation**: AC-003.4 already covers this case — a collision would trigger the overwrite/append prompt. Adequate.

- **Finding**: No AC for the case where `AskUserQuestion` returns empty/blank answers. The skill should handle non-responsive users.
- **Evidence**: AC-002.2 and AC-002.4 describe the tool but not degenerate input.
- **Recommendation**: Non-blocking — this is interaction-level behavior that the skill body handles naturally. Not AC-worthy for a 500-word skill.

### 3. Scope Creep Risk

- **Finding**: Scope is tight. Non-requirements are explicit and specific. No porous boundaries detected.
- **Evidence**: Non-requirements list 5 specific exclusions including BL-016, BL-015, adversarial review, Ousterhout analysis, and grill-me coupling. Constraints reinforce with "No new agents, tools, or crate dependencies" and "No tracker/workflow state integration."
- **Recommendation**: None.

### 4. Dependency Gaps

- **Finding**: DAG is clean: US-001 (no deps) -> US-002 -> US-003; US-004 (no deps, independent). No circular dependencies. No implicit shared state between US-001/002/003 and US-004.
- **Evidence**: All `Depends on` references exist within the spec. US-004 is correctly independent since the phase-gate change and the skill file have no coupling.
- **Recommendation**: None.

- **Finding**: The spec references BL-016 as a downstream consumer but does not declare a formal dependency. This is correct — BL-012 is the upstream, not the downstream.
- **Evidence**: `BL-016 (prd-to-plan) is downstream consumer — PRD template is the contract`
- **Recommendation**: None — the spec correctly identifies the relationship direction.

### 5. Testability

- **Finding**: AC-001.2 (`ecc validate skills` passes) is deterministically testable — the validator is already implemented and tested. AC-001.3 (under 500 words) is deterministically testable with `wc -w`. AC-004.1/004.2/004.3 are deterministically testable with Rust unit tests. These are all clean.
- **Evidence**: `crates/ecc-app/src/validate/skills.rs` validates `name`, `description`, `origin` frontmatter fields. Existing test suite covers all validation paths.
- **Recommendation**: None for US-001 and US-004.

- **Finding**: AC-002.1 through AC-002.4 and AC-003.1 through AC-003.5 describe behavioral expectations of a Markdown skill file. These are not testable via automated tests — they require manual invocation of the skill and human verification. This is inherent to the artifact type (skills are instructions, not code).
- **Evidence**: Skills are Markdown documents with no executable test harness.
- **Recommendation**: Acceptable for skill-type artifacts. The Rust code changes (US-004) have full test coverage. The skill file itself is validated structurally by `ecc validate skills`.

### 6. Decision Completeness

- **Finding**: Missing decision on PRD template versioning. The spec says "PRD template sections must be stable for BL-016 consumption" (Constraints) but does not record a decision about what happens if the template needs to evolve. When BL-016 is implemented, will it parse section headers? What if a section is renamed?
- **Evidence**: `PRD template sections must be stable for BL-016 consumption` (Constraints, line 100)
- **Recommendation**: Add a decision: "PRD template sections are v1 — breaking changes require BL-016 coordination." This is low-risk because BL-016 does not exist yet, so any template change before BL-016 implementation has zero blast radius.

- **Finding**: All 6 recorded decisions have substantive rationales. None are vacuous "because it's better" hand-waves.
- **Evidence**: Each decision row has a clear reason tied to project conventions or downstream impact.
- **Recommendation**: None.

### 7. Rollback & Failure

- **Finding**: The change is two files: one new Markdown file and one line addition to a Rust static array. Rollback is trivial — delete the skill directory, remove the line from `allowed_prefixes()`. No data migrations, no schema changes, no breaking changes.
- **Evidence**: Affected Modules table shows exactly 2 files, one new (no existing state to corrupt) and one additive-only change.
- **Recommendation**: None.

- **Finding**: The phase-gate change is additive-only (adding a new prefix). It cannot break existing behavior — existing tests confirm this via AC-004.2.
- **Evidence**: `allowed_prefixes()` returns a `Vec<String>` and the change appends one more entry.
- **Recommendation**: None.

## Suggested ACs

None required. The spec is implementable as written. The minor ambiguities identified (slug length boundary, "each step" over-broadness) are self-correcting during skill file authoring and do not risk divergent implementations.

## Verdict Rationale

The spec is well-scoped for what it is: a Markdown skill file plus a one-line Rust change. The scope is ruthlessly contained by explicit non-requirements and constraints. The dependency graph is clean. The Rust changes are deterministically testable. The skill-behavior ACs are inherently untestable by automation but this is intrinsic to the artifact type, not a spec deficiency. The missing PRD template versioning decision is low-risk because the downstream consumer (BL-016) does not exist yet. No dimension scores below 70. No critical findings that would cause implementation failure. PASS.
