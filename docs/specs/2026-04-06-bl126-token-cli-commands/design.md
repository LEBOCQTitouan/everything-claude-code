# Solution: BL-126 — 6 Token-Saving CLI Commands

## Spec Reference
Concern: dev, Feature: bl126-token-cli-commands

## File Changes (32 files across 4 waves)

### Wave 1: Domain (11 files)
| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1-3 | crates/ecc-domain/src/drift/{mod,error}.rs + lib.rs | create/modify | US-001 |
| 4-7 | crates/ecc-domain/src/docs/{mod,coverage,module_summary,diagram_triggers,claude_md,error}.rs | create | US-002,003,004,006 |
| 8-9 | crates/ecc-domain/src/analyze/commit_lint.rs + mod.rs | create/modify | US-005 |

### Wave 2: App (6 files)
| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 10-15 | crates/ecc-app/src/{drift_check,docs_update_summary,docs_coverage,diagram_triggers,commit_lint,validate_claude_md}.rs + lib.rs | create/modify | US-001-006 |

### Wave 3: CLI (7 files)
| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 16-20 | crates/ecc-cli/src/commands/{drift,docs,diagram,commit}.rs + validate.rs + mod.rs + main.rs | create/modify | US-001-006 |

### Wave 4: Agent/command updates (7 files)
| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 21-26 | agents/{drift-checker,module-summary-updater,doc-reporter,diagram-updater,doc-validator}.md + commands/commit.md | modify | US-001-006 |
| 27 | CLAUDE.md | modify | Doc plan |

## Pass Conditions (54 PCs)
See planner output in conversation context for full PC table.

All 39 ACs covered by 54 PCs. Zero uncovered.

## Test Strategy
TDD: domain (6 modules, ~40 tests) → app (6 use cases, ~30 tests) → CLI (5 modules, ~15 tests) → agent updates (grep checks) → build gate

## Doc Update Plan
| # | Doc File | Action | Spec Ref |
|---|----------|--------|----------|
| 1 | CLAUDE.md | Add 6 CLI commands | Doc plan |
| 2 | CHANGELOG.md | Add BL-126 entry | Doc plan |

## SOLID Assessment
PASS — follows established hexagonal pattern. Domain pure, ports reused, no new abstractions.

## Robert's Oath Check
CLEAN — 54 PCs, atomic commits, TDD order.

## Security Notes
CLEAR — local filesystem + git only, no secrets, no network.

## Rollback Plan
Reverse wave order: delete agent updates → delete CLI → delete app → delete domain modules.

## Bounded Contexts Affected
| Context | Role | Files |
|---------|------|-------|
| drift (new) | Workflow compliance | drift/{mod,error}.rs |
| docs (new) | Documentation analysis | docs/{coverage,module_summary,diagram_triggers,claude_md}.rs |
| analyze (existing) | Commit analysis | analyze/commit_lint.rs |
