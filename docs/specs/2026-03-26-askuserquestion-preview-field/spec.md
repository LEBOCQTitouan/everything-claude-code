# Spec: AskUserQuestion Preview Field for Architecture Comparisons

## Problem Statement

Currently zero ECC commands, skills, or agents use AskUserQuestion's `preview` field — all interactive questions use only `label` + `description`. When presenting architecture alternatives (e.g., during grill-me interviews, design reviews, or interface comparisons), users must mentally reconstruct visual differences from text descriptions. The `preview` field renders Markdown in a monospace box for side-by-side visual comparison, which would significantly improve decision quality for architecture-related questions.

## Research Summary

- **Preview = dry-run for decisions** — showing concrete deltas before committing reduces rework (Terraform plan, git dry-run pattern)
- **AskUserQuestion has a 60-second timeout** — preview content must be digestible at a glance (2-3 alternatives, concise descriptions)
- **Mermaid diagrams are diffable text** — ideal for before/after architecture comparisons in preview boxes
- **Sequential presentation > side-by-side** in terminals due to width constraints — preview handles this natively
- **Always label the delta** — comparisons without "what changed and why" annotation are hard to evaluate
- **Stick to 2-3 alternatives** to avoid information overload
- **Prior art**: Terraform plan output, GitHub PR diff view, ArgoCD diff-preview all use structured comparison interfaces

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Inline preview instructions in each file (no shared skill) | YAGNI — extract shared skill later if preview spreads to 5+ files | No |
| 2 | Preview only when 2+ visual alternatives exist | Avoid forced usage on textual questions; matches web research on decision overload | No |
| 3 | Add AskUserQuestion to design.md's allowed-tools | design.md is the only pipeline command missing it; fixes asymmetry with spec-* commands | No |
| 4 | All 4 user stories in scope | User wants comprehensive coverage within ECC | No |

## User Stories

### US-001: Add Preview Guidance to grill-me Skill

**As a** spec pipeline user, **I want** the grill-me skill to instruct use of AskUserQuestion's `preview` field when presenting visual alternatives, **so that** I can compare approaches via Mermaid diagrams, code snippets, or ASCII layouts.

#### Acceptance Criteria

- AC-001.1: Given the grill-me skill's AskUserQuestion Enforcement section, when the skill is loaded, then it includes instructions to use `preview` when 2+ distinct approaches have structural differences.
- AC-001.2: Given a grill-me question in any stage that involves choosing between visual alternatives, when AskUserQuestion is called, then each option's preview contains a Markdown code block (Mermaid, code snippet, or ASCII layout).
- AC-001.3: Given a grill-me question that is purely textual (no visual alternatives), when AskUserQuestion is called, then no `preview` field is included.
- AC-001.4: Given AskUserQuestion is unavailable (fallback mode), when preview content exists, then it is shown inline as Markdown in the conversational fallback.

#### Dependencies

- Depends on: none

### US-002: Add Preview to /design Architecture Alternatives

**As a** design pipeline user, **I want** the `/design` command and interface-designer agent to use `preview` when presenting architecture alternatives, **so that** I can visually compare design options.

#### Acceptance Criteria

- AC-002.1: Given `design.md`'s frontmatter, when the file is read, then `AskUserQuestion` is present in the `allowed-tools` list.
- AC-002.2: Given the interface-designer agent's Phase 7 (User Synthesis), when presenting "Which design best fits your primary use case?", then each option includes a `preview` showing the interface signature and usage example.
- AC-002.3: Given the design command presents architecture alternatives during planning, when multiple approaches exist, then AskUserQuestion with preview is used.
- AC-002.4: Given only one viable design approach, when the design proceeds, then no forced AskUserQuestion is injected.

#### Dependencies

- Depends on: none

### US-003: Add Preview to /spec-* Grill-Me Invocations

**As a** spec pipeline user, **I want** the `/spec-dev`, `/spec-fix`, and `/spec-refactor` commands to instruct preview usage for architecture-related grill-me questions, **so that** alternatives are visually comparable during the interview.

#### Acceptance Criteria

- AC-003.1: Given `/spec-dev` Phase 6 mandatory questions, when questions involve architecture comparisons (from architect agent output), then the command instructs use of `preview` showing each approach's structure.
- AC-003.2: Given `/spec-fix` mandatory question 2 ("minimal vs proper fix"), when both approaches exist, then the command instructs use of `preview` showing the minimal patch vs structural fix.
- AC-003.3: Given `/spec-refactor` mandatory question 2 ("target architecture"), when current vs target states differ visually, then the command instructs use of `preview` showing before/after architecture.
- AC-003.4: Given any mandatory question that is purely textual, when AskUserQuestion is called, then no preview is forced.

#### Dependencies

- Depends on: none

### US-004: Add Preview to configure-ecc and interviewer

**As a** user choosing between configuration or design alternatives, **I want** configure-ecc and the interviewer agent to use `preview` for single-select questions with visual differences, **so that** preview usage is consistent across all ECC interactive surfaces.

#### Acceptance Criteria

- AC-004.1: Given configure-ecc presents a single-select question with structurally different options (e.g., installation level with different directory trees), when AskUserQuestion is called, then each option includes a `preview` showing the resulting file structure.
- AC-004.2: Given the interviewer agent reaches a stage with visual alternatives (e.g., Desired State with 2+ approaches), when a single-select question is used, then `preview` shows relevant code or architecture comparisons.
- AC-004.3: Given a multiSelect question (e.g., skill category selection), when AskUserQuestion is called, then no `preview` is used (preview is single-select only).

#### Dependencies

- Depends on: none

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `skills/grill-me/SKILL.md` | Content (outside Rust hexagon) | Add preview guidance to AskUserQuestion Enforcement section |
| `commands/design.md` | Content | Add AskUserQuestion to allowed-tools + preview instruction |
| `commands/spec-dev.md` | Content | Add preview instruction to Phase 6 grill-me |
| `commands/spec-fix.md` | Content | Add preview instruction to Phase 5 grill-me |
| `commands/spec-refactor.md` | Content | Add preview instruction to Phase 5 grill-me |
| `agents/interface-designer.md` | Content | Add preview instruction to Phase 7 |
| `skills/configure-ecc/SKILL.md` | Content | Add preview guidance for single-select questions |
| `agents/interviewer.md` | Content | Add preview guidance for visual alternative stages |

## Constraints

- Preview is single-select only (AskUserQuestion limitation)
- Keep preview content concise for 60-second AskUserQuestion timeout
- No Rust code changes required or permitted
- No Mermaid rendering — source text only in preview boxes
- `ecc validate` must pass for all modified files after changes
- Prior audit finding [SELF-004]: 4 commands missing allowed-tools — this spec fixes one (design.md)

## Non-Requirements

- Mermaid diagram rendering (we provide source text only)
- /implement Plan Mode preview usage
- Any Rust code changes
- Shared preview-conventions skill (deferred to future if needed)
- Preview for multiSelect questions (not supported by AskUserQuestion)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| None | N/A | No port/adapter surface changed — all changes are content layer |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| Frontmatter fix | Minor | `commands/design.md` | Add AskUserQuestion to allowed-tools |
| Content addition | Minor | 8 Markdown files | Add preview usage instructions |
| Changelog | Minor | `CHANGELOG.md` | Add entry for preview field adoption |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | What is explicitly OUT of scope? | All 4 user stories in scope within ECC. Out of scope: Rust changes, Mermaid rendering, /implement. In other projects, only relevant subset activates. | User |
| 2 | What happens when no visual alternatives exist? | Preview only when 2+ visual alternatives exist. No forced usage on textual questions. | Recommended |
| 3 | Which critical paths need 100% coverage vs 80%? | 100% frontmatter validation via ecc validate. No new Rust tests. Manual preview instruction review. | Recommended |
| 4 | Are there latency/throughput requirements? | No constraints. Keep preview concise for 60s AskUserQuestion timeout. | Recommended |
| 5 | Does this touch auth, user data, or external APIs? | No security implications. Purely instructional Markdown content. | Recommended |
| 6 | Will this change any existing public API or data contract? | No breaking changes. All changes are additive. | Recommended |
| 7 | Are there domain terms that need defining? | No glossary entry. Usage note in affected files instead. | Recommended |
| 8 | Which decisions warrant an ADR? | No ADR. Inline preview instructions. Extract shared skill later if needed (YAGNI). | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Add Preview Guidance to grill-me Skill | 4 | none |
| US-002 | Add Preview to /design Architecture Alternatives | 4 | none |
| US-003 | Add Preview to /spec-* Grill-Me Invocations | 4 | none |
| US-004 | Add Preview to configure-ecc and interviewer | 3 | none |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | grill-me AskUserQuestion Enforcement includes preview instructions for 2+ alternatives | US-001 |
| AC-001.2 | Visual alternative questions include Markdown code block previews | US-001 |
| AC-001.3 | Textual questions skip preview | US-001 |
| AC-001.4 | Fallback mode shows preview content inline | US-001 |
| AC-002.1 | design.md allowed-tools includes AskUserQuestion | US-002 |
| AC-002.2 | interface-designer Phase 7 uses preview for design comparison | US-002 |
| AC-002.3 | design command uses preview when multiple approaches exist | US-002 |
| AC-002.4 | Single viable design skips forced AskUserQuestion | US-002 |
| AC-003.1 | spec-dev Phase 6 instructs preview for architecture comparisons | US-003 |
| AC-003.2 | spec-fix Q2 uses preview for minimal vs proper fix | US-003 |
| AC-003.3 | spec-refactor Q2 uses preview for before/after architecture | US-003 |
| AC-003.4 | Textual mandatory questions skip preview | US-003 |
| AC-004.1 | configure-ecc single-select with structural differences uses preview | US-004 |
| AC-004.2 | interviewer visual alternative stages use preview | US-004 |
| AC-004.3 | multiSelect questions never use preview | US-004 |

### Adversary Findings

| Dimension | Verdict | Key Rationale |
|-----------|---------|---------------|
| Ambiguity | PASS | ACs are concrete with exact file/section references |
| Edge cases | PASS | Textual-only, single approach, multiSelect, fallback all covered |
| Scope | PASS | Tightly bounded to 8 Markdown files, Non-Requirements fences future work |
| Dependencies | PASS | All stories independent, no cross-dependencies |
| Testability | PASS | All ACs follow Given/When/Then with verifiable conditions |
| Decisions | PASS | All 4 decisions have clear rationale (YAGNI, research, audit finding) |
| Rollback | PASS | Additive Markdown edits, git revert restores previous state |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-26-askuserquestion-preview-field/spec.md | Full spec + phase summary |
