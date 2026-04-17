# Spec: Tribal Knowledge Documentation Upgrade (Meta-Style)

Source: BL-152 | Scope: HIGH | Content-only

## Problem Statement

ECC's documentation pipeline produces comprehensive MODULE-SUMMARIES but misses tribal knowledge — non-obvious failure patterns, hidden cross-module dependencies, and undocumented conventions buried in code comments. Meta's 2026 approach achieved 100% AI context coverage and 40% fewer tool calls via compact "compass" context files. ECC should adopt: five-question extraction, compass files, auto-repair, and periodic validation.

## Research Summary

- **Meta Engineering (2026-04-06)**: 50+ agents, five-question framework, 59 compass files (25-35 lines), 40% fewer tool calls, research time 2 days → 30 min
- **"Compass not encyclopedia"**: compact orientation beats comprehensive reference for agent context
- **ECC gap**: MODULE-SUMMARIES.md unbounded narrative, no tribal knowledge extraction, no auto-repair, `/audit-doc` manual-only
- **Existing infrastructure**: doc-orchestrator pipeline (analyzer→generator→validator→reporter) already implements explorer→analyst→writer→critic pattern
- **Auto-repair precedent**: doc-validator already has CLAUDE.md auto-fix for non-controversial items

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Five-question framework as a skill | Reuses doc-analyzer enrichment pipeline | Yes |
| 2 | Compass files for ALL component types | User: 9 crates + agents + commands + skills + hooks + rules + teams | Yes |
| 3 | Compass files should "transpire into every file" | Per-file tribal knowledge at install time — deferred to follow-up for transpiration mechanism | Yes |
| 4 | Tiered auto-repair: LOW/MEDIUM auto, HIGH/CRITICAL flag | Prevents risky auto-changes | Yes |
| 5 | Periodic validation via session-start hook | ECC-native pattern. 7-day configurable interval. | No |
| 6 | Content-only implementation | Zero Rust changes | No |

## User Stories

### US-001: Five-Question Tribal Knowledge Extraction Skill
- AC-001.1: `skills/tribal-knowledge-extraction/SKILL.md` exists with valid frontmatter
- AC-001.2: Answers five questions: configure/provide, common mods, failure patterns, hidden deps, comment knowledge
- AC-001.3: Delegates to `failure-modes` and `behaviour-extraction` for Q3/Q4
- AC-001.4: Zero-marker modules output "No embedded tribal knowledge detected"
- AC-001.5: Referenced by `doc-analyzer` in Step 5b enrichment
**Dependencies:** none

### US-002: Compass Context File Generation
- AC-002.1: `docs/context/<component>.md` for all 9 crates + component directories
- AC-002.2: Sections: Quick Commands, Key Files (3-5), Non-Obvious Patterns, Cross-References
- AC-002.3: 25-35 lines, <1000 tokens each
- AC-002.4: Update in place on re-run
- AC-002.5: `/implement` Phase 7.5 regenerates for affected crates
- AC-002.6: `agents/compass-context-writer.md` exists
- AC-002.7: `skills/compass-context-gen/SKILL.md` exists
**Dependencies:** US-001

### US-003: Doc Drift Auto-Repair Mode
- AC-003.1: LOW drift auto-fixed with commit
- AC-003.2: MEDIUM drift auto-fixed (paths, counts)
- AC-003.3: HIGH/CRITICAL flagged only
- AC-003.4: Post-repair validation pass
- AC-003.5: `doc-validator` updated with `--auto-repair` mode
**Dependencies:** none

### US-004: Periodic Validation Session Hook
- AC-004.1: Session-start hook runs if >7 days since last
- AC-004.2: HIGH/CRITICAL → console warning
- AC-004.3: Timestamp to `.doc-validation-last-run`
- AC-004.4: Kill switch `ECC_DOC_VALIDATION_DISABLED=1`
- AC-004.5: Non-blocking
**Dependencies:** none

### US-005: Doc-Orchestrator Integration
- AC-005.1: Phase 1.7 tribal knowledge extraction
- AC-005.2: Phase 2c compass generation
- AC-005.3: Phase 2d auto-repair
- AC-005.4: `/doc-suite --phase=tribal-knowledge` standalone
- AC-005.5: `doc-orchestrator` updated
**Dependencies:** US-001, US-002, US-003

## Affected Modules

| Module | Layer | Change |
|--------|-------|--------|
| `skills/tribal-knowledge-extraction/SKILL.md` (new) | content | Five-question skill |
| `skills/compass-context-gen/SKILL.md` (new) | content | Compass skill |
| `agents/compass-context-writer.md` (new) | content | Compass agent |
| `agents/doc-analyzer.md` | content | Skill reference |
| `agents/doc-validator.md` | content | Auto-repair mode |
| `agents/doc-orchestrator.md` | content | Phase 1.7, 2c, 2d |
| `agents/module-summary-updater.md` | content | Tribal-knowledge skill |
| `commands/doc-suite.md` | content | `--auto-repair` flag |
| `commands/implement.md` | content | Phase 7.5 compass |
| `hooks/hooks.json` | content | Validation hook |

Zero Rust code changes.

## Constraints

Content-only. Compass 25-35 lines. Auto-repair tiered. Periodic non-blocking. No MODULE-SUMMARIES duplication.

## Non-Requirements

GUI/TUI. External hosting. Real-time editing. Per-file transpiration (follow-up). Compass CLI linting (follow-up).

## E2E Boundaries Affected

None.

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| ADR | Architecture | `docs/adr/0065-tribal-knowledge-docs.md` | Create |
| CHANGELOG | Project | CHANGELOG.md | feat entry |
| CLAUDE.md | Project | CLAUDE.md | 3 glossary terms |

## Open Questions

None.
