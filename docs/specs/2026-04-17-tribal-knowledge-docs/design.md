# Solution: Tribal Knowledge Documentation Upgrade (Meta-Style)

## Spec Reference
Concern: `dev` | Feature: BL-152 tribal knowledge doc upgrade — Meta-style module context files

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `skills/tribal-knowledge-extraction/SKILL.md` | create | Five-question framework skill | US-001 |
| 2 | `agents/doc-analyzer.md` | modify | Add tribal-knowledge-extraction to skills list + Step 5b | US-001 |
| 3 | `agents/module-summary-updater.md` | modify | Add tribal-knowledge skill for /implement context | US-001 |
| 4 | `skills/compass-context-gen/SKILL.md` | create | Compass file generation convention | US-002 |
| 5 | `agents/compass-context-writer.md` | create | Haiku agent for compass file authoring | US-002 |
| 6 | `commands/implement.md` | modify | Phase 7.5: add compass-context-writer dispatch | US-002 |
| 7 | `agents/doc-validator.md` | modify | Add `--auto-repair` mode for LOW/MEDIUM drift | US-003 |
| 8 | `hooks/hooks.json` | modify | Register session-start doc validation hook | US-004 |
| 9 | `agents/doc-orchestrator.md` | modify | Add Phase 1.7, 2c, 2d | US-005 |
| 10 | `commands/doc-suite.md` | modify | Add `--auto-repair` + `--phase=tribal-knowledge` | US-005 |
| 11 | `docs/adr/0065-tribal-knowledge-docs.md` | create | ADR for 4 decisions | Doc |
| 12 | `CHANGELOG.md` | modify | feat entry | Doc |
| 13 | `CLAUDE.md` | modify | 3 glossary terms | Doc |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | lint | Tribal knowledge skill exists with frontmatter | AC-001.1 | `test -f skills/tribal-knowledge-extraction/SKILL.md && grep -q '^name:' skills/tribal-knowledge-extraction/SKILL.md && echo PASS` | PASS |
| PC-002 | lint | Five questions documented | AC-001.2 | `grep -c 'configure\|common modification\|failure pattern\|cross-module\|tribal knowledge' skills/tribal-knowledge-extraction/SKILL.md \| awk '{if($1>=5) print "PASS"; else print "FAIL"}'` | PASS |
| PC-003 | lint | Delegates to failure-modes + behaviour-extraction | AC-001.3 | `grep -q 'failure-modes' skills/tribal-knowledge-extraction/SKILL.md && grep -q 'behaviour-extraction' skills/tribal-knowledge-extraction/SKILL.md && echo PASS` | PASS |
| PC-004 | lint | Zero-marker fallback documented | AC-001.4 | `grep -q 'No embedded tribal knowledge' skills/tribal-knowledge-extraction/SKILL.md && echo PASS` | PASS |
| PC-005 | lint | doc-analyzer references skill | AC-001.5 | `grep -q 'tribal-knowledge-extraction' agents/doc-analyzer.md && echo PASS` | PASS |
| PC-006 | lint | Compass skill exists | AC-002.7 | `test -f skills/compass-context-gen/SKILL.md && grep -q '^name:' skills/compass-context-gen/SKILL.md && echo PASS` | PASS |
| PC-007 | lint | Compass agent exists with frontmatter | AC-002.6 | `test -f agents/compass-context-writer.md && grep -q '^name:' agents/compass-context-writer.md && echo PASS` | PASS |
| PC-008 | lint | Compass sections documented | AC-002.2 | `for s in "Quick Commands" "Key Files" "Non-Obvious Patterns" "Cross-References"; do grep -q "$s" skills/compass-context-gen/SKILL.md \|\| exit 1; done && echo PASS` | PASS |
| PC-009 | lint | Line budget documented (25-35) | AC-002.3 | `grep -q '25.*35\|25-35' skills/compass-context-gen/SKILL.md && echo PASS` | PASS |
| PC-010 | lint | Update-in-place documented | AC-002.4 | `grep -qi 'update.*in.*place\|idempotent\|overwrite' skills/compass-context-gen/SKILL.md && echo PASS` | PASS |
| PC-011 | lint | implement.md references compass dispatch | AC-002.5 | `grep -q 'compass-context-writer' commands/implement.md && echo PASS` | PASS |
| PC-012 | lint | Compass covers all component types | AC-002.1 | `grep -qi 'crate\|agent\|command\|skill\|hook\|rule\|team' skills/compass-context-gen/SKILL.md && echo PASS` | PASS |
| PC-013 | lint | Auto-repair LOW documented | AC-003.1 | `grep -qi 'LOW.*auto.*fix\|auto.*repair.*LOW' agents/doc-validator.md && echo PASS` | PASS |
| PC-014 | lint | Auto-repair MEDIUM documented | AC-003.2 | `grep -qi 'MEDIUM.*auto.*fix\|auto.*repair.*MEDIUM\|stale.*count\|path.*resolv' agents/doc-validator.md && echo PASS` | PASS |
| PC-015 | lint | HIGH/CRITICAL flagged only | AC-003.3 | `grep -qi 'HIGH.*flag\|CRITICAL.*flag\|not.*auto.*fix\|manual.*review' agents/doc-validator.md && echo PASS` | PASS |
| PC-016 | lint | Post-repair validation | AC-003.4 | `grep -qi 'post.*repair.*valid\|re.*validate\|validation.*pass' agents/doc-validator.md && echo PASS` | PASS |
| PC-017 | lint | --auto-repair mode documented | AC-003.5 | `grep -q 'auto-repair' agents/doc-validator.md && echo PASS` | PASS |
| PC-018 | lint | Session hook registered | AC-004.1 | `grep -q 'doc.*validation\|drift.*check' hooks/hooks.json && echo PASS` | PASS |
| PC-019 | lint | Warning on HIGH/CRITICAL | AC-004.2 | `grep -qi 'warning\|CRITICAL\|HIGH' hooks/hooks.json && echo PASS` | PASS |
| PC-020 | lint | Timestamp file documented | AC-004.3 | `grep -q 'doc-validation-last-run' hooks/hooks.json && echo PASS` | PASS |
| PC-021 | lint | Kill switch documented | AC-004.4 | `grep -q 'ECC_DOC_VALIDATION_DISABLED' hooks/hooks.json && echo PASS` | PASS |
| PC-022 | lint | Non-blocking (async) | AC-004.5 | `grep -qi 'async.*true\|non.*blocking\|fire.*forget' hooks/hooks.json && echo PASS` | PASS |
| PC-023 | lint | Phase 1.7 in orchestrator | AC-005.1, .5 | `grep -q 'Phase 1.7\|Tribal Knowledge' agents/doc-orchestrator.md && echo PASS` | PASS |
| PC-024 | lint | Phase 2c compass generation | AC-005.2 | `grep -q 'Phase 2c\|compass\|Compass' agents/doc-orchestrator.md && echo PASS` | PASS |
| PC-025 | lint | Phase 2d auto-repair + standalone flag | AC-005.3, .4 | `grep -q 'Phase 2d\|auto-repair\|Auto-Repair' agents/doc-orchestrator.md && grep -q 'tribal-knowledge' commands/doc-suite.md && echo PASS` | PASS |
| PC-026 | e2e | validate agents | — | `cargo run -p ecc-cli --quiet -- validate agents` | exit 0 |
| PC-027 | e2e | validate commands | — | `cargo run -p ecc-cli --quiet -- validate commands` | exit 0 |
| PC-028 | e2e | validate conventions | — | `cargo run -p ecc-cli --quiet -- validate conventions` | exit 0 |

### Coverage Check
All 25 ACs covered by ≥1 PC. Zero uncovered.

### E2E Test Plan
None — content-only, no port/adapter changes.

### E2E Activation Rules
None.

## Test Strategy

1. Phase 1 (US-001): PC-001..005 — tribal knowledge skill + doc-analyzer reference
2. Phase 2 (US-002): PC-006..012 — compass skill + agent + implement integration
3. Phase 3 (US-003): PC-013..017 — auto-repair mode in doc-validator
4. Phase 4 (US-004): PC-018..022 — periodic validation hook
5. Phase 5 (US-005): PC-023..025 — orchestrator integration
6. Final: PC-026..028 — all validators pass

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `docs/adr/0065-tribal-knowledge-docs.md` | architecture | create | 4 decisions: five-question framework, compass-all-components, tiered auto-repair, session-start hook | Decisions 1-4 |
| 2 | `CHANGELOG.md` | project | modify | feat: tribal knowledge doc upgrade (BL-152) | mandatory |
| 3 | `CLAUDE.md` | project | modify | 3 glossary terms: compass context file, tribal knowledge extraction, auto-repair | Doc |

## SOLID Assessment
PASS — content-only, no code changes.

## Robert's Oath Check
CLEAN — additive documentation infrastructure.

## Security Notes
CLEAR — no injection surface in markdown skills/agents. Hook shells out to existing `ecc drift check`.

## Rollback Plan
Delete new files, revert modifications in reverse order: CLAUDE.md → CHANGELOG → ADR → doc-suite → orchestrator → hooks.json → doc-validator → implement.md → compass agent → compass skill → doc-analyzer → module-summary-updater → tribal knowledge skill.

## Bounded Contexts Affected
No bounded contexts affected — zero Rust modules modified. Content-only.
