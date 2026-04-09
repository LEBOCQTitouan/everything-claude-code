# Spec Adversary Report

## Summary
Verdict: PASS (avg: 79/100)
Rounds: 2

## Dimension Results
| # | Dimension | Score | Verdict | Critical Findings |
|---|-----------|-------|---------|-------------------|
| 1 | Ambiguity | 72 | PASS | Slug length still ambiguous (slug vs full filename); Decision #1 still claims "same pattern" but now actually follows it |
| 2 | Edge Cases | 65 | CONDITIONAL | No AC for exploration failure or plan size boundary |
| 3 | Scope Creep Risk | 88 | PASS | Tight constraints, specific non-requirements |
| 4 | Dependency Gaps | 85 | PASS | DAG is clean; US-003 correctly depends on US-002 |
| 5 | Testability | 68 | CONDITIONAL | Behavioral ACs (flow steps, plan content) inherently untestable for skill artifacts |
| 6 | Decision Completeness | 65 | CONDITIONAL | Missing decisions on template versioning and Decision #1 wording fix |
| 7 | Rollback & Failure | 92 | PASS | Trivially revertible two-file change |

## Detailed Findings

### 1. Ambiguity

- **Finding**: AC-003.1 still says "kebab-case slug, max 40 chars" without clarifying whether the limit applies to the slug portion or the full filename including `-plan.md`. This is the same ambiguity flagged in round 1 (inherited from BL-012's adversary report). Two engineers could enforce different limits.
- **Evidence**: `AC-003.1: Output to docs/refactors/{name}-plan.md (kebab-case slug, max 40 chars)`
- **Recommendation**: Clarify: "slug portion (the `{name}` segment before `-plan.md`) must not exceed 40 characters." Non-blocking -- any reasonable interpretation produces a working implementation, and the BL-012 precedent already shipped with the same ambiguity.

- **Finding**: Decision #1 still reads "Same pattern as BL-012" which is now accurate (3 content stories + phase-gate + backlog = 5 stories vs BL-012's 4 stories + no backlog = 4 stories), but the rationale text was not updated from round 1. The structural split was the primary round-1 fix; this cosmetic issue no longer causes implementation divergence.
- **Evidence**: Decision #1: "Same pattern as BL-012 -- Self-contained, under 500 words, AskUserQuestion, file output"
- **Recommendation**: Cosmetic only. The claim is now substantively true.

### 2. Edge Cases

- **Finding**: No AC addresses what happens when codebase exploration (step 2) reveals the refactoring target does not exist or is infeasible. The skill assumes exploration always succeeds. For a planning skill, silently proceeding with a plan based on non-existent code is a waste of effort.
- **Evidence**: AC-002.1 step "(2) explore codebase" has no failure/abort path.
- **Recommendation**: Add to US-002: "If exploration reveals the target does not exist or is infeasible, the skill reports findings and asks the user to revise scope before proceeding." Non-blocking for a skill artifact -- the skill body will naturally handle this conversationally, but the spec does not guarantee it.

- **Finding**: No AC addresses plan size boundaries. A user could request a 50-commit refactoring plan. The spec imposes no upper bound on commit count or guidance for when to suggest splitting.
- **Evidence**: No constraint on plan size or commit count anywhere in the spec.
- **Recommendation**: Add guidance constraint: "If decomposition exceeds 20 commits, suggest splitting into multiple refactoring plans." Non-blocking -- the skill author will likely add this naturally, but it is not specified.

### 3. Scope Creep Risk

- **Finding**: Non-requirements are specific and defensible. The three-way split of stories reduced the risk of scope creep within any single US. "No automated commit execution" and "No Rust source code changes beyond phase-gate" are clear fences.
- **Evidence**: Non-Requirements section lines 109-113; Constraints section lines 104-107.
- **Recommendation**: None.

### 4. Dependency Gaps

- **Finding**: The dependency DAG is now clean. US-001 has no deps. US-002 depends on US-001. US-003 depends on US-002. US-004 has no deps (phase-gate is independent). US-005 depends on US-001. No cycles, no missing references.
- **Evidence**: Each US has an explicit Dependencies section with valid references.
- **Recommendation**: None.

- **Finding**: US-005 (backlog update) depends on US-001 but not US-003. This means "implemented" triggers when the skill file exists, not when the plan output template is also in place. Acceptable -- the skill file contains the template instructions; they are not separable artifacts.
- **Evidence**: `US-005 Dependencies: Depends on: US-001`
- **Recommendation**: None. Consistent with BL-012 pattern where backlog update depends on the structural US.

### 5. Testability

- **Finding**: Deterministically testable ACs: AC-001.1 (frontmatter fields), AC-001.2 (validator pass), AC-001.3 (trigger phrases present in file), AC-004.1 (phase-gate prefix), AC-004.2 (existing tests pass), AC-004.3 (new test), AC-005.1 (backlog status). That is 7 of 14 ACs fully testable.
- **Evidence**: Phase-gate tests are Rust unit tests; skill validation is `ecc validate skills`.
- **Recommendation**: None for the testable ACs.

- **Finding**: Untestable ACs: AC-002.1 (6-step flow defined -- verifiable by reading skill but subjective on step boundaries), AC-002.2 (AskUserQuestion usage -- behavioral), AC-002.3 (Read/Grep/Glob usage -- behavioral), AC-003.1 (output path -- behavioral), AC-003.2 (template fields -- behavioral), AC-003.3 (green-state notation -- content), AC-003.4 (directory creation -- behavioral), AC-003.5 (overwrite prompt -- behavioral). These describe skill behavior that manifests only during interactive use.
- **Evidence**: All US-002 and US-003 ACs describe runtime behavior of a Markdown skill file.
- **Recommendation**: Inherent limitation of skill-type specs. The split into separate stories makes each untestable AC at least *scoped* to one concern, which is an improvement over round 1's monolithic US. The testability ratio (7/14 = 50%) is comparable to BL-012 (7/13 = 54%). Non-blocking.

### 6. Decision Completeness

- **Finding**: The round-1 finding about missing "commit ordering strategy" decision is resolved -- the spec now includes step (5) "order commits by dependency" in AC-002.1, matching the backlog's 6-step flow. No separate decision needed since the spec adopted the backlog's design wholesale.
- **Evidence**: AC-002.1: "(5) order commits by dependency"
- **Recommendation**: None.

- **Finding**: No decision on plan template versioning. AC-003.2 defines a per-commit template (Change Description, Affected Files, Risk Level, Rollback Instruction) that is implicitly a contract. Unlike BL-012 where the template has a declared downstream consumer (BL-016), no consumer exists here. But if one is added later, the template becomes a breaking-change risk.
- **Evidence**: No downstream consumer declared; no versioning posture stated.
- **Recommendation**: Add Decision #5: "Plan template is unversioned -- no downstream parser exists. Template may evolve freely until a consumer is declared." Low-risk omission.

- **Finding**: Risk Level vocabulary is now defined inline in AC-003.2. This resolves the round-1 finding but could have been recorded as a Decision table entry for traceability. Minor -- the definition is clear and unambiguous in the AC itself.
- **Evidence**: AC-003.2: "Risk Level (LOW/MEDIUM/HIGH -- LOW: no behavior change, MEDIUM: behavior change with test coverage, HIGH: behavior change in untested area)"
- **Recommendation**: Cosmetic only. The inline definition is sufficient.

### 7. Rollback & Failure

- **Finding**: Unchanged from round 1. Two-file change: one new Markdown skill directory, one line addition to Rust allowlist. Rollback is trivial. No data migrations, no schema changes, no breaking API changes. The phase-gate change is additive-only.
- **Evidence**: Affected Modules table: 2 files.
- **Recommendation**: None.

## Suggested ACs

These would strengthen the spec but are not blocking:

1. **Clarify slug length in AC-003.1**: "slug portion (`{name}` before `-plan.md`) must not exceed 40 characters"

2. **Add exploration failure path to US-002**: "If codebase exploration reveals the refactoring target does not exist or is infeasible, the skill reports findings and asks the user to revise scope before proceeding to decomposition"

3. **Add plan size guidance**: "If decomposition produces more than 20 commits, the skill suggests splitting into multiple refactoring plans" (as a constraint or AC on US-003)

4. **Add Decision #5**: "Plan template is unversioned -- no downstream parser exists; template may evolve freely until a consumer is declared"

## Verdict Rationale

The three critical round-1 findings are resolved. US-001 is now split into three focused stories (structure, flow, template) that genuinely mirror BL-012. The 6-step flow matches the backlog. Risk Level has a defined vocabulary with clear criteria. The dependency graph is clean. Testability is at parity with BL-012 for this artifact type.

Remaining gaps are non-blocking: slug length ambiguity is inherited from BL-012 and has not caused implementation problems there; exploration failure and plan size are edge cases the skill author will handle conversationally even without explicit ACs; template versioning is a forward-looking concern with no current consumer. No single dimension falls below 50. The average score of 79 clears the PASS threshold, and no finding warrants blocking implementation.
