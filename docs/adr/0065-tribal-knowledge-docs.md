# ADR 0065 — Tribal Knowledge Documentation

**Status**: Accepted

## Context

ECC has accumulated significant undocumented tribal knowledge: non-obvious configuration patterns, common modification workflows, failure modes, hidden dependencies, and wisdom embedded in code comments. Standard doc generation tools (doc-generator, doc-analyzer) produce structural documentation but miss the operational knowledge that makes modules understandable to newcomers. Additionally, module-level context (purpose, commands, key files, cross-references) is scattered or absent, forcing readers to infer orientation from source code alone.

BL-152 introduces a tribal knowledge extraction framework, compass context files, auto-repair mode, and periodic validation to address these gaps.

## Decision 1: Five-Question Framework as Skill (Not Agent)

The tribal knowledge extraction methodology is packaged as `skills/tribal-knowledge-extraction/SKILL.md` rather than a standalone agent. The five questions (configure/provide, common mods, failure patterns, hidden deps, comment knowledge) are a reusable extraction protocol invoked by `doc-analyzer` rather than a fixed pipeline.

**Rationale**: Skills compose with existing agents (doc-analyzer, doc-orchestrator) without adding agent proliferation. The extraction logic benefits from doc-analyzer's source traversal context rather than re-reading files independently.

## Decision 2: Compass Files for All Component Types

`compass-context-writer` generates `docs/context/<component>.md` compass files for every crate and component directory, not just top-level modules. Each compass is 25-35 lines covering Quick Commands, Key Files, Non-Obvious Patterns, and Cross-References.

**Rationale**: All component types — crates, agent directories, skill directories, command collections — benefit from orientation context. Limiting to top-level would leave most components without entry-point documentation.

## Decision 3: Deferred Transpiration Mechanism

Compass content should ultimately appear in every file that references the component (inline orientation context), but direct transpiration is deferred. Phase 2c generates standalone `docs/context/` files. Inline injection into source files is a future concern tracked separately.

**Rationale**: Transpiration into source files risks noisy diffs and conflicts with code formatters. Standalone compass files deliver the orientation value immediately without tooling risk. Transpiration can be added as an opt-in flag when the mechanism is proven.

## Decision 4: Tiered Auto-Repair

Doc drift repair is tiered: LOW and MEDIUM severity drift is auto-fixed by `doc-validator --auto-repair` (Phase 2d). HIGH and CRITICAL findings are flagged with a warning and require manual review.

**Rationale**: LOW/MEDIUM drift (stale examples, minor inaccuracies, missing links) is safe to repair automatically with low risk of incorrect changes. HIGH/CRITICAL drift (architectural contradictions, security-relevant inaccuracies) requires human judgment before correction. The periodic validation hook enforces this by emitting warnings on HIGH/CRITICAL findings at session start.

## Consequences

- `doc-orchestrator` gains three new phases (1.7, 2c, 2d) increasing full pipeline runtime
- `doc-suite` gains `--phase=tribal-knowledge` and `--auto-repair` flags for standalone execution
- `docs/context/` directory is the canonical location for compass files
- Periodic validation hook (`session:start:doc-validation`) runs drift check every 7 days; respects `ECC_DOC_VALIDATION_DISABLED=1` kill switch
