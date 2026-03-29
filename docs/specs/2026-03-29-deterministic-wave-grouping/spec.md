# Spec: Deterministic Wave Grouping Algorithm (BL-070)

## Problem Statement

The `/implement` command's wave-dispatch phase requires grouping Pass Conditions into parallel execution waves based on file overlap. Currently, the LLM manually evaluates file changes, identifies overlaps, and forms groups — a process that takes 5-10 seconds, is error-prone (the LLM often under-parallelizes), and produces non-deterministic results across sessions. This is a pure graph algorithm perfectly suited for deterministic code.

## Research Summary

- **Graph coloring maps directly to file-overlap grouping**: PCs sharing files are modeled as adjacent vertices in a conflict graph; greedy coloring assigns waves. Crates like `heuristic_graph_coloring` exist but are overkill for our small n.
- **Left-to-right scan with non-adjacent grouping is O(n²)**: sufficient when PC counts are modest (tens, not thousands). The overlap matrix check is the dominant operation.
- **Deterministic test sharding ensures reproducibility**: same input always produces same output — critical for session re-entry.
- **Sequential merge after parallel execution preserves correctness**: waves execute in order, each wave's results merge before the next starts.
- **Push-based scheduling is the dominant pattern**: completing a wave exposes the next wave. Our wave-plan pre-computes all waves upfront, which is simpler and sufficient.
- **File path normalization is essential**: backtick-wrapped paths in markdown tables must be stripped for accurate overlap detection.

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Non-adjacent greedy bin-packing grouping | Better parallelism than adjacent-only scan; user expanded scope | Yes (ADR-032) |
| 2 | Domain algorithm in `ecc-domain::spec::wave` | Pure logic, zero I/O, follows hexagonal pattern | Yes (ADR-032) |
| 3 | CLI subcommand in `ecc-workflow` | Matches all other implement-phase commands (transition, scope-check, etc.) | Yes (ADR-032) |
| 4 | `max_per_wave` as function parameter (default 4) | Keeps domain generic and testable | No |
| 5 | PCs with no file matches treated as independent | Safest default — unknown files don't block parallelism | No |
| 6 | Hard requirement on binary (no manual fallback) | Eliminates inconsistency between deterministic and LLM grouping | No |
| 7 | Path validation via canonicalize+starts_with | Consistent with tasks subcommands security pattern | No |
| 8 | Backtick stripping in PC-to-files mapping | File Changes table wraps paths in backticks; must normalize for overlap | No |

## User Stories

### US-001: PC-to-Files Mapping

**As a** domain developer, **I want** a function that builds a `PcId -> Vec<String>` mapping from PCs and FileChanges via AC cross-reference, **so that** wave grouping and ordering checks share the same logic.

#### Acceptance Criteria

- AC-001.1: Given PCs and FileChanges with AC cross-references, when `build_pc_file_map` is called, then returns HashMap<PcId, Vec<String>> with correct mappings
- AC-001.2: Given a FileChange with spec_ref "AC-001.1, AC-002.1" and PCs verifying those ACs, then both PCs include that file
- AC-001.3: Given file paths wrapped in backticks, then backticks are stripped
- AC-001.4: Given a PC with no AC matches in FileChanges, then the PC maps to an empty Vec
- AC-001.5: Given duplicate files for the same PC (same file via multiple ACs), then deduplicated
- AC-001.6: Given FileChanges with duplicate file paths having different spec_refs, then both spec_refs contribute to the PC mapping
- AC-001.7: Given a FileChange with non-parseable spec_ref (e.g., "US-001" instead of "AC-NNN.N"), then that entry maps to no PCs (silently skipped)

#### Dependencies

- Depends on: none

### US-002: Wave Grouping Algorithm

**As a** workflow automation developer, **I want** a pure domain function that computes wave groups from PCs and file overlap, **so that** wave grouping is deterministic and testable.

#### Acceptance Criteria

- AC-002.1: Given PCs with no file overlaps, when `compute_wave_plan` is called, then groups into waves of max 4 (non-adjacent allowed)
- AC-002.2: Given PCs where all share at least one file, then each PC gets its own wave (fully sequential)
- AC-002.3: Given a single PC, then result is one wave with one PC
- AC-002.4: Given an empty PC list, then returns empty wave plan
- AC-002.5: Given PCs [A(file1), B(file1,file2), C(file2), D(file3)], then A and D can be in the same wave (non-adjacent grouping)
- AC-002.6: Given >4 independent PCs, then split into multiple waves of max 4
- AC-002.7: Given `max_per_wave` parameter, then respects the configured maximum
- AC-002.8: Given PCs [A, B, C, D, E] in input order where all are independent, then wave 1 contains [A, B, C, D] and wave 2 contains [E], preserving input order
- AC-002.9: Given PCs that share identical file sets, then assigned to different waves with earlier-indexed PC in earlier wave

#### Dependencies

- Depends on: US-001

### US-003: `ecc-workflow wave-plan` Subcommand

**As a** `/implement` command, **I want** `ecc-workflow wave-plan <design-path>` that outputs a JSON wave plan, **so that** wave grouping is instant and deterministic.

#### Acceptance Criteria

- AC-003.1: Given valid design.md with both tables, when run, then outputs JSON with `waves` array and metadata, exits 0
- AC-003.2: Given JSON output, then each wave has `id` (1-based), `pcs` (array of PC-NNN strings), `files` (union of files)
- AC-003.3: Given design.md with no PC table, then exits with block status and error
- AC-003.4: Given nonexistent path, then exits with block status
- AC-003.5: Given path traversal attempt, then rejected
- AC-003.6: Given design.md with no File Changes table, then outputs JSON with `status: "warn"`, all PCs grouped in waves of 4, and exit code 0

#### Dependencies

- Depends on: US-002

### US-004: `/implement` Command Integration

**As a** `/implement` user, **I want** Phase 2 to call `ecc-workflow wave-plan` and Phase 3 to use its output, **so that** wave dispatch is deterministic.

#### Acceptance Criteria

- AC-004.1: Phase 2 calls `!ecc-workflow wave-plan <design-path>` and parses JSON output
- AC-004.2: Phase 3 dispatches waves according to the computed plan
- AC-004.3: wave-analysis skill updated to reference `ecc-workflow wave-plan`
- AC-004.4: No manual fallback — binary is required, error if unavailable

#### Dependencies

- Depends on: US-003

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `ecc-domain/src/spec/wave.rs` (new) | Domain | Wave, WavePlan, build_pc_file_map, compute_wave_plan |
| `ecc-domain/src/spec/mod.rs` | Domain | Add `pub mod wave` |
| `ecc-workflow/src/commands/wave_plan.rs` (new) | Adapter/CLI | wave-plan subcommand handler |
| `ecc-workflow/src/main.rs` | Adapter/CLI | Add WavePlan variant to Commands enum |
| `ecc-workflow/src/commands/mod.rs` | Adapter/CLI | Add `pub mod wave_plan` |
| `skills/wave-analysis/SKILL.md` | Skill | Replace manual algorithm with CLI call |
| `commands/implement.md` | Command | Update Phase 2 to use wave-plan |

## Constraints

- `ecc-domain` must have zero I/O imports — all logic is pure `(&[PassCondition], &[FileChange]) -> WavePlan`
- File paths must be normalized (backticks stripped, whitespace trimmed)
- `max_per_wave` defaults to 4, configurable as function parameter
- <50ms execution target for all inputs up to 100 PCs
- 100% branch coverage for all ecc-domain wave logic
- Reuse existing `parse_pcs()` and `parse_file_changes()` from `ecc-domain::spec`

## Non-Requirements

- Graph coloring libraries (overkill for small n)
- Runtime wave rebalancing during execution
- Fixing `scope_check.rs` duplicated parsing (separate backlog item)
- Adjacent-only grouping (user expanded to non-adjacent bin-packing)
- Optimal minimum-wave-count solution (greedy is sufficient)

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| ecc-workflow CLI | New subcommand | Smoke-test wave-plan JSON output |
| /implement command | Behavioral change | Full implement dry-run needed |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New CLI subcommand | CLAUDE.md | CLI Commands | Add `ecc-workflow wave-plan` |
| New domain types | MODULE-SUMMARIES.md | ecc-domain | Add wave module summary |
| Architecture decision | docs/adr/ | New ADR-032 | Deterministic wave grouping |
| Skill update | skills/wave-analysis/ | SKILL.md | Replace algorithm with CLI call |
| CHANGELOG | CHANGELOG.md | project | Add BL-070 entry |
| Test count | CLAUDE.md | project | Update test count |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope boundaries | Allow non-adjacent PC grouping for better parallelism | User |
| 2 | Edge cases (no file match) | Treat as independent, can group freely | Recommended |
| 3 | Test strategy | 100% branch coverage for all ecc-domain code | User |
| 4 | Performance | <50ms target, O(n²) fine for small n | Recommended |
| 5 | Security | Path validation via canonicalize+starts_with | Recommended |
| 6 | Breaking changes | Hard requirement on binary, no manual fallback | User |
| 7 | Domain concepts | Wave, WavePlan, File Overlap | Recommended |
| 8 | ADR | One ADR for deterministic wave grouping | Recommended |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | PC-to-Files Mapping | 7 | none |
| US-002 | Wave Grouping Algorithm | 9 | US-001 |
| US-003 | `ecc-workflow wave-plan` Subcommand | 6 | US-002 |
| US-004 | `/implement` Command Integration | 4 | US-003 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | PC-to-files mapping via AC cross-reference | US-001 |
| AC-001.2 | Multi-AC spec_ref maps to multiple PCs | US-001 |
| AC-001.3 | Backtick stripping on file paths | US-001 |
| AC-001.4 | No-match PC maps to empty Vec | US-001 |
| AC-001.5 | Duplicate files deduplicated | US-001 |
| AC-001.6 | Duplicate file paths across FileChanges | US-001 |
| AC-001.7 | Non-parseable spec_ref silently skipped | US-001 |
| AC-002.1 | No-overlap PCs grouped max 4 (non-adjacent) | US-002 |
| AC-002.2 | All-overlap PCs fully sequential | US-002 |
| AC-002.3 | Single PC → one wave | US-002 |
| AC-002.4 | Empty PC list → empty plan | US-002 |
| AC-002.5 | Non-adjacent grouping example | US-002 |
| AC-002.6 | >4 independent split into multiple waves | US-002 |
| AC-002.7 | max_per_wave parameter respected | US-002 |
| AC-002.8 | Input order preserved in waves | US-002 |
| AC-002.9 | Identical file sets → different waves | US-002 |
| AC-003.1 | Valid design → JSON waves + metadata | US-003 |
| AC-003.2 | Wave has id, pcs, files | US-003 |
| AC-003.3 | No PC table → block | US-003 |
| AC-003.4 | Nonexistent path → block | US-003 |
| AC-003.5 | Path traversal → rejected | US-003 |
| AC-003.6 | No File Changes → warn with default waves | US-003 |
| AC-004.1 | Phase 2 calls wave-plan | US-004 |
| AC-004.2 | Phase 3 uses computed waves | US-004 |
| AC-004.3 | Skill updated | US-004 |
| AC-004.4 | No manual fallback | US-004 |

### Adversary Findings

| Dimension | Score | Verdict | Key Rationale |
|-----------|-------|---------|---------------|
| Ambiguity | 72 | PASS | AC-002.8 clarified with concrete example |
| Edge Cases | 78 | PASS | 5 ACs added in round 2 |
| Scope | 82 | PASS | Well-bounded by Non-Requirements |
| Dependencies | 85 | PASS | Clean linear chain |
| Testability | 78 | PASS | AC-004.x inherently non-automatable |
| Decisions | 88 | PASS | Non-adjacent grouping well-justified |
| Rollback | 75 | PASS | Additive changes, git-revertible |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| docs/specs/2026-03-29-deterministic-wave-grouping/spec.md | Full spec |
