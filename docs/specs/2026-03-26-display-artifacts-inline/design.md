# Implementation Plan: BL-062 Display Full Artifacts Inline in Terminal

## Overview

Insert a standardized "read and display artifact" instruction block into the final "Present and STOP" phase of 5 command files (spec-dev, spec-fix, spec-refactor, design, implement), immediately before the existing summary tables. Pure Markdown content changes with no code or test impact.

## Requirements

- Full artifact content displayed inline before summary tables in each command's final phase
- Artifact path read from `state.json` via `artifacts.<type>_path`
- Fallback: warn and skip if path is null or file does not exist
- Existing summary tables remain unchanged (BL-048 format preserved)
- File path shown after tables for reference
- Consistent template text across all 5 files (per AC-004.2)

## Architecture Changes

- `commands/spec-dev.md` Phase 10 — insert inline display block before summary tables
- `commands/spec-fix.md` Phase 9 — insert inline display block before summary tables
- `commands/spec-refactor.md` Phase 9 — insert inline display block before summary tables
- `commands/design.md` Phase 11 — insert inline display block before summary tables
- `commands/implement.md` Phase 8 — insert inline display block before summary tables

Layers: [Framework] (command definition files only — no domain/usecase/adapter changes)

## Template Block

The following instruction block is inserted in each file. The `<ARTIFACT_KEY>` and `<ARTIFACT_LABEL>` placeholders are substituted per command:

```markdown
### Full Artifact Display

Read the full artifact from `artifacts.<ARTIFACT_KEY>` in `.claude/workflow/state.json` using the Read tool. Display the complete file content inline in conversation. If the path is null or the file does not exist, emit a warning and skip to the summary tables.

> **Fallback**: If the artifact path is unavailable, display:
> "Warning: `artifacts.<ARTIFACT_KEY>` not found in state.json or file does not exist. Skipping inline display."
```

Substitution table:

| Command | ARTIFACT_KEY | ARTIFACT_LABEL |
|---------|-------------|----------------|
| spec-dev | `spec_path` | spec |
| spec-fix | `spec_path` | spec |
| spec-refactor | `spec_path` | spec |
| design | `design_path` | design |
| implement | `tasks_path` | tasks |

## Implementation Steps

### Phase 1: spec-dev.md (1 file)

1. **Insert inline display block in spec-dev.md** (File: `commands/spec-dev.md`)
   - Action: Insert the template block (with `spec_path`) between the Phase 10 heading + intro line and the `### Grill-Me Decisions` table
   - Insertion point: After line 279 ("Display a comprehensive Phase Summary using these tables:"), before line 281 ("### Grill-Me Decisions")
   - Also: Add a `### Artifact File Path` section after the last table (after "Phase Summary Persistence" block, before "Then STOP") showing `artifacts.spec_path` for reference
   - Why: First file establishes the pattern; spec-dev is the most-used spec command
   - Dependencies: None
   - Risk: Low — pure Markdown insertion

#### Pass Conditions for Phase 1

- PC-001: `grep -c "Read the full artifact from" commands/spec-dev.md` returns `1`
- PC-002: `grep -c "artifacts.spec_path" commands/spec-dev.md` returns at least `1`
- PC-003: `grep -c "Fallback" commands/spec-dev.md` returns at least `1`
- PC-004: `grep -c "### Grill-Me Decisions" commands/spec-dev.md` returns `1` (tables unchanged)
- PC-005: `grep -c "### Full Artifact Display" commands/spec-dev.md` returns `1`

### Phase 2: spec-fix.md and spec-refactor.md (2 files)

2. **Insert inline display block in spec-fix.md** (File: `commands/spec-fix.md`)
   - Action: Insert the template block (with `spec_path`) between the Phase 9 heading + intro line and the `### Grill-Me Decisions` table
   - Insertion point: After line 263, before line 265
   - Also: Add artifact file path reference section
   - Dependencies: Phase 1 (pattern established)
   - Risk: Low

3. **Insert inline display block in spec-refactor.md** (File: `commands/spec-refactor.md`)
   - Action: Identical to step 2 but in spec-refactor.md Phase 9
   - Insertion point: After line 275, before line 277
   - Also: Add artifact file path reference section
   - Dependencies: Phase 1
   - Risk: Low

#### Pass Conditions for Phase 2

- PC-006: `grep -c "Read the full artifact from" commands/spec-fix.md` returns `1`
- PC-007: `grep -c "artifacts.spec_path" commands/spec-fix.md` returns at least `1`
- PC-008: `grep -c "### Full Artifact Display" commands/spec-fix.md` returns `1`
- PC-009: `grep -c "### Grill-Me Decisions" commands/spec-fix.md` returns `1`
- PC-010: `grep -c "Read the full artifact from" commands/spec-refactor.md` returns `1`
- PC-011: `grep -c "artifacts.spec_path" commands/spec-refactor.md` returns at least `1`
- PC-012: `grep -c "### Full Artifact Display" commands/spec-refactor.md` returns `1`
- PC-013: `grep -c "### Grill-Me Decisions" commands/spec-refactor.md` returns `1`

### Phase 3: design.md (1 file)

4. **Insert inline display block in design.md** (File: `commands/design.md`)
   - Action: Insert the template block (with `design_path`) between the Phase 11 heading + intro line and the `### Design Reviews` table
   - Insertion point: After line 263, before line 265
   - Also: Add artifact file path reference section
   - Dependencies: Phase 1
   - Risk: Low

#### Pass Conditions for Phase 3

- PC-014: `grep -c "Read the full artifact from" commands/design.md` returns `1`
- PC-015: `grep -c "artifacts.design_path" commands/design.md` returns at least `1`
- PC-016: `grep -c "### Full Artifact Display" commands/design.md` returns `1`
- PC-017: `grep -c "### Design Reviews" commands/design.md` returns `1`

### Phase 4: implement.md (1 file)

5. **Insert inline display block in implement.md** (File: `commands/implement.md`)
   - Action: Insert the template block (with `tasks_path`) between "Display a comprehensive Phase Summary using these tables:" and the `### Tasks Executed` table
   - Insertion point: After line 330, before line 332
   - Also: Add artifact file path reference section
   - Dependencies: Phase 1
   - Risk: Low

#### Pass Conditions for Phase 4

- PC-018: `grep -c "Read the full artifact from" commands/implement.md` returns `1`
- PC-019: `grep -c "artifacts.tasks_path" commands/implement.md` returns at least `1`
- PC-020: `grep -c "### Full Artifact Display" commands/implement.md` returns `1`
- PC-021: `grep -c "### Tasks Executed" commands/implement.md` returns `1`

### Phase 5: Cross-file validation and build verification

6. **Validate consistency across all 5 files**
   - Action: Verify the template text is identical (modulo key substitution) across all 5 files
   - Dependencies: Phases 1-4
   - Risk: Low

#### Pass Conditions for Phase 5

- PC-022: `grep -rc "### Full Artifact Display" commands/spec-dev.md commands/spec-fix.md commands/spec-refactor.md commands/design.md commands/implement.md | grep -c ":1"` returns `5`
- PC-023: `grep -rc "emit a warning and skip to the summary tables" commands/spec-dev.md commands/spec-fix.md commands/spec-refactor.md commands/design.md commands/implement.md | grep -c ":1"` returns `5`
- PC-024: `npm run lint` passes (Markdown linting)
- PC-025: `cargo build` passes (no Rust impact)
- PC-026: `cargo test` passes (no test impact)
- PC-027: `ecc validate commands` passes (if applicable)

## Exact Edit Specifications

### For spec-dev.md, spec-fix.md, spec-refactor.md (artifact key: `spec_path`)

Insert after the line "Display a comprehensive Phase Summary using these tables:" and before the first `###` table heading:

```markdown

### Full Artifact Display

Read the full artifact from `artifacts.spec_path` in `.claude/workflow/state.json` using the Read tool. Display the complete file content inline in conversation. If the path is null or the file does not exist, emit a warning and skip to the summary tables.

> **Fallback**: If the artifact path is unavailable, display:
> "Warning: `artifacts.spec_path` not found in state.json or file does not exist. Skipping inline display."

```

Insert after the "Phase Summary Persistence" paragraph and before "Then STOP":

```markdown

### Artifact File Path

After the tables, display the artifact file path for reference:

> **Spec file**: `<path from artifacts.spec_path>`

```

### For design.md (artifact key: `design_path`)

Same structure but with `design_path` replacing `spec_path`, and "Design file" replacing "Spec file".

### For implement.md (artifact key: `tasks_path`)

Same structure but with `tasks_path` replacing `spec_path`, and "Tasks file" replacing "Spec file".

## E2E Assessment

- **Touches user-facing flows?** Yes — CLI command output display
- **Crosses 3+ modules end-to-end?** No — only Markdown command definitions
- **New E2E tests needed?** No — changes are command instructions (Markdown), not executable code. Verification is via grep pass conditions and existing `ecc validate`.

## Testing Strategy

- Unit tests: N/A (no code changes)
- Integration tests: N/A (no code changes)
- E2E tests: Run existing suite only
- Validation: grep-based pass conditions per phase + `npm run lint` + `cargo build` + `cargo test`

## Commit Cadence

Since this is pure Markdown with no TDD cycle, the commit pattern is:

1. `docs: display full spec inline in spec-dev final phase` (Phase 1)
2. `docs: display full spec inline in spec-fix and spec-refactor final phases` (Phase 2)
3. `docs: display full design inline in design final phase` (Phase 3)
4. `docs: display full tasks inline in implement final phase` (Phase 4)

## Risks & Mitigations

- **Risk**: Markdown linting fails due to heading hierarchy or spacing
  - Mitigation: Follow existing heading patterns in each file; run `npm run lint` after each phase
- **Risk**: Inconsistent template text across files due to manual editing
  - Mitigation: PC-022 and PC-023 verify exact text match across all 5 files
- **Risk**: Existing summary tables accidentally modified
  - Mitigation: PC-004, PC-009, PC-013, PC-017, PC-021 verify table headings still exist

## Success Criteria

- [ ] All 5 command files contain `### Full Artifact Display` section in their final phase
- [ ] Template instruction text matches AC-004.2 in all files
- [ ] Each file references the correct artifact key (spec_path / design_path / tasks_path)
- [ ] Fallback instruction present in all 5 files
- [ ] Existing summary tables unchanged (all original `###` headings still present)
- [ ] `### Artifact File Path` section present after tables in all 5 files
- [ ] `npm run lint` passes
- [ ] `cargo build` passes
- [ ] `cargo test` passes
- [ ] All 27 pass conditions GREEN
