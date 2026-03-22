# Solution: Create design-an-interface skill + agent (BL-014)

## Spec Reference
Concern: dev, Feature: Create design-an-interface skill + agent (BL-014)

## File Changes (dependency order)
| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | tests/hooks/test-interface-designer.sh | create | Bash test suite — validates skill+agent content | US-001–008 |
| 2 | skills/design-an-interface/SKILL.md | create | Methodology skill — triggers, constraints, dimensions, anti-patterns | US-001 |
| 3 | agents/interface-designer.md | create | Orchestration agent — parallel sub-agents, comparison, synthesis | US-002–006 |
| 4 | commands/design.md | modify | Mention interface-designer as optional in Phase 1 | US-007 |
| 5 | docs/domain/glossary.md | modify | Add Interface Designer term | AC-008.1 |
| 6 | CHANGELOG.md | modify | Add BL-014 feature entry | AC-008.2 |
| 7 | docs/adr/0008-designs-directory-convention.md | create | ADR for docs/designs/ convention | AC-008.3 |

## Pass Conditions
43 PCs covering 33 ACs. PC-001–010 (skill), PC-011–035 (agent), PC-036–037 (/design), PC-038–041 (docs), PC-042–043 (lint/build).

## Test Strategy
TDD order: skill tests → skill creation → agent tests → agent creation → command tests → command edit → doc tests → doc edits → build verification.

## Doc Update Plan
Glossary (Interface Designer), CHANGELOG (BL-014), ADR 0008 (docs/designs/ convention). No additional ADRs.

## SOLID Assessment
PASS

## Robert's Oath Check
CLEAN

## Security Notes
CLEAR

## Rollback Plan
Delete test-interface-designer.sh, delete SKILL.md, delete interface-designer.md, revert design.md, revert glossary, revert CHANGELOG, delete ADR 0008.
