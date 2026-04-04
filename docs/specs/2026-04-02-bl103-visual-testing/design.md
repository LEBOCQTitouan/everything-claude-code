# Solution: BL-103 Autonomous Visual Testing Integration

## Spec Reference
Concern: dev, Feature: BL-103 autonomous visual testing integration

## File Changes (dependency order)

| # | File | Action | Rationale | Spec Ref |
|---|------|--------|-----------|----------|
| 1 | `docs/adr/0042-vision-vs-pixel-comparison.md` | Create | Record architectural decision: AI vision as primary comparison, pixel-diff as supplementary | Decision #3, AC-003.5 |
| 2 | `skills/visual-testing/SKILL.md` | Create | New skill with screenshot capture patterns, vision assertions, regression detection, baseline management, pixel-diff guidance, security warnings, cost/latency guidance, wait-for-stable, dynamic content masking, and complete examples | AC-001.3-5, AC-002.3-4, AC-003.4-7, AC-004.3, AC-004.6-10 |
| 3 | `agents/e2e-runner.md` | Modify | Add `visual-testing` to skills list, `visual: true` input contract field, `visual_results` output contract, visual mode workflow sections | AC-001.1-2, AC-002.1-2, AC-003.1-3, AC-004.1-2, AC-004.4-5 |
| 4 | `skills/e2e-testing/SKILL.md` | Modify | Add cross-reference to visual-testing skill | Spec affected modules |
| 5 | `docs/domain/glossary.md` | Modify | Add 4 visual testing glossary terms | Grill-me #7 |
| 6 | `CHANGELOG.md` | Modify | Add feat entry for BL-103 | — |

## Pass Conditions

| ID | Type | Description | Verifies AC | Command | Expected |
|----|------|-------------|-------------|---------|----------|
| PC-001 | content | ADR 0042 exists with Status: Accepted, Context, Decision, Consequences | Decision #3 | `grep -q "Status: Accepted" docs/adr/0042-vision-vs-pixel-comparison.md && grep -q "## Context" docs/adr/0042-vision-vs-pixel-comparison.md && grep -q "## Decision" docs/adr/0042-vision-vs-pixel-comparison.md && grep -q "## Consequences" docs/adr/0042-vision-vs-pixel-comparison.md && echo PASS || echo FAIL` | PASS |
| PC-002 | content | Skill frontmatter valid (name, description, origin) | AC-004.5 | `grep -q "^name: visual-testing$" skills/visual-testing/SKILL.md && grep -q "^origin: ECC$" skills/visual-testing/SKILL.md && grep -q "^description:" skills/visual-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-003 | content | VisualCapture helper pattern in skill | AC-001.3 | `grep -q "VisualCapture" skills/visual-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-004 | content | Manifest schema with typed fields (ISO 8601, viewport, url, stepName, filePath) | AC-001.4, AC-001.5 | `grep -q "manifest.json" skills/visual-testing/SKILL.md && grep -q "visual-artifacts/" skills/visual-testing/SKILL.md && grep -q "ISO 8601" skills/visual-testing/SKILL.md && grep -q "stepName" skills/visual-testing/SKILL.md && grep -q "filePath" skills/visual-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-005 | content | visualAssert() pattern documented | AC-002.3 | `grep -q "visualAssert" skills/visual-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-006 | content | Graceful skip on Read failure | AC-002.4 | `grep -qi "skipped" skills/visual-testing/SKILL.md && grep -qi "warning" skills/visual-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-007 | content | Severity classification: cosmetic/functional/breaking | AC-003.6 | `grep -q "cosmetic" skills/visual-testing/SKILL.md && grep -q "functional" skills/visual-testing/SKILL.md && grep -q "breaking" skills/visual-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-008 | content | Baseline keying: {test-name}/{checkpoint-id}/{browser-viewport} | AC-003.4, AC-003.7 | `grep -q "baseline" skills/visual-testing/SKILL.md && grep -q "test-name" skills/visual-testing/SKILL.md && grep -q "checkpoint-id" skills/visual-testing/SKILL.md && grep -q "browser-viewport" skills/visual-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-009 | content | pixelmatch/reg-cli patterns | AC-003.5 | `grep -q "pixelmatch" skills/visual-testing/SKILL.md && grep -q "reg-cli" skills/visual-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-010 | content | Two examples: login flow + dashboard regression | AC-004.3 | `grep -q "login" skills/visual-testing/SKILL.md && grep -q "dashboard" skills/visual-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-011 | content | Security section: PII, credentials, .gitignore | AC-004.6 | `grep -qi "PII" skills/visual-testing/SKILL.md && grep -qi "credential" skills/visual-testing/SKILL.md && grep -q ".gitignore" skills/visual-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-012 | content | Wait-for-stable pattern with examples | AC-004.7 | `grep -qi "wait.*stable\|waitForLoadState\|animation" skills/visual-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-013 | content | End-to-end example journey spec with visual: true | AC-004.8 | `grep -q 'visual: true' skills/visual-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-014 | content | CSS masking for dynamic content | AC-004.9 | `grep -qi "mask" skills/visual-testing/SKILL.md && grep -qi "dynamic content\|timestamp\|ads" skills/visual-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-015 | content | Cost/latency formula: ~3s, ~1K tokens, 10 checkpoints | AC-004.10 | `grep -q "3s" skills/visual-testing/SKILL.md && grep -q "1K tokens" skills/visual-testing/SKILL.md && grep -q "10" skills/visual-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-016 | content | e2e-runner skills list includes visual-testing | AC-004.5 | `grep -q 'visual-testing' agents/e2e-runner.md && echo PASS || echo FAIL` | PASS |
| PC-017 | content | e2e-runner input contract has visual field | AC-004.1 | `grep -q 'visual' agents/e2e-runner.md && echo PASS || echo FAIL` | PASS |
| PC-018 | content | e2e-runner output contract has visual_results | AC-004.4 | `grep -q 'visual_results' agents/e2e-runner.md && echo PASS || echo FAIL` | PASS |
| PC-019 | content | Backward compatibility documented | AC-004.2 | `grep -qi "when.*visual.*not specified\|without visual\|visual mode is not" agents/e2e-runner.md && echo PASS || echo FAIL` | PASS |
| PC-020 | content | Screenshot metadata (viewport, timestamp) in agent | AC-001.1, AC-001.2 | `grep -q "viewport" agents/e2e-runner.md && grep -q "timestamp" agents/e2e-runner.md && echo PASS || echo FAIL` | PASS |
| PC-021 | content | Vision assertion pass/fail with reasoning in agent | AC-002.1 | `grep -qi "read.*screenshot\|Read tool.*image\|vision.*analys" agents/e2e-runner.md && grep -qi "pass.*fail\|pass/fail" agents/e2e-runner.md && echo PASS || echo FAIL` | PASS |
| PC-022 | content | Failed assertion report includes path, text, explanation | AC-002.2 | `grep -qi "screenshot.*path\|assertion.*text\|explanation\|reasoning" agents/e2e-runner.md && echo PASS || echo FAIL` | PASS |
| PC-023 | content | Baseline comparison flow in agent | AC-003.1 | `grep -qi "compare.*baseline\|baseline.*compare\|previous.*run" agents/e2e-runner.md && echo PASS || echo FAIL` | PASS |
| PC-024 | content | Regression report with severity in agent | AC-003.2 | `grep -qi "regression.*report\|cosmetic.*functional.*breaking\|severity" agents/e2e-runner.md && echo PASS || echo FAIL` | PASS |
| PC-025 | content | No-baseline handling in agent | AC-003.3 | `grep -qi "no baseline\|new baseline\|first.*run\|baseline.*not.*exist" agents/e2e-runner.md && echo PASS || echo FAIL` | PASS |
| PC-026 | content | Agent Browser compatibility statement | — | `grep -qi "Agent Browser\|Playwright" agents/e2e-runner.md && echo PASS || echo FAIL` | PASS |
| PC-027 | content | e2e-testing cross-references visual-testing | Spec modules | `grep -q "visual-testing" skills/e2e-testing/SKILL.md && echo PASS || echo FAIL` | PASS |
| PC-028 | content | Glossary has 4 visual testing terms | Grill-me #7 | `grep -q "Visual Checkpoint" docs/domain/glossary.md && grep -q "Visual Assertion" docs/domain/glossary.md && grep -q "Visual Baseline" docs/domain/glossary.md && grep -q "Visual Regression" docs/domain/glossary.md && echo PASS || echo FAIL` | PASS |
| PC-029 | content | ADR mentions vision and pixel in decision | Decision #3 | `grep -qi "vision" docs/adr/0042-vision-vs-pixel-comparison.md && grep -qi "pixel" docs/adr/0042-vision-vs-pixel-comparison.md && echo PASS || echo FAIL` | PASS |
| PC-030 | content | CHANGELOG has BL-103 entry | — | `grep -q "BL-103\|visual testing" CHANGELOG.md && echo PASS || echo FAIL` | PASS |
| PC-031 | content | Skill file under 800 lines | Coding style | `test $(wc -l < skills/visual-testing/SKILL.md) -lt 800 && echo PASS || echo FAIL` | PASS |
| PC-032 | validation | ecc validate agents passes | Structural | `cargo run -- validate agents && echo PASS || echo FAIL` | PASS |
| PC-033 | validation | ecc validate skills passes | Structural | `cargo run -- validate skills && echo PASS || echo FAIL` | PASS |

### Coverage Check

All 26 ACs covered:

| AC | Covered By |
|----|-----------|
| AC-001.1 | PC-017, PC-020 |
| AC-001.2 | PC-020 |
| AC-001.3 | PC-003 |
| AC-001.4 | PC-004 |
| AC-001.5 | PC-004 |
| AC-002.1 | PC-021 |
| AC-002.2 | PC-022 |
| AC-002.3 | PC-005 |
| AC-002.4 | PC-006 |
| AC-003.1 | PC-023 |
| AC-003.2 | PC-007, PC-024 |
| AC-003.3 | PC-025 |
| AC-003.4 | PC-008 |
| AC-003.5 | PC-009 |
| AC-003.6 | PC-007 |
| AC-003.7 | PC-008 |
| AC-004.1 | PC-017 |
| AC-004.2 | PC-019 |
| AC-004.3 | PC-010 |
| AC-004.4 | PC-018 |
| AC-004.5 | PC-016 |
| AC-004.6 | PC-011 |
| AC-004.7 | PC-012 |
| AC-004.8 | PC-013 |
| AC-004.9 | PC-014 |
| AC-004.10 | PC-015 |

Uncovered ACs: none.

### E2E Test Plan

| # | Boundary | Adapter | Port | Test Description | Default State | Run When |
|---|----------|---------|------|------------------|---------------|----------|
| — | None | — | — | No Rust code boundaries affected — content-layer only | — | — |

### E2E Activation Rules

No E2E tests to activate. All verification is via content validation (grep + `ecc validate`).

## Test Strategy

TDD order (dependency-driven):

1. **PC-001** — ADR (standalone, establishes rationale)
2. **PC-002** — Skill scaffold (frontmatter must exist before content)
3. **PC-003** — VisualCapture (capture before assertions)
4. **PC-004** — Manifest schema (completes capture protocol)
5. **PC-005** — visualAssert (assertions depend on capture)
6. **PC-006** — Graceful skip (error handling for assertions)
7. **PC-007** — Severity classification (before baseline management)
8. **PC-008** — Baseline management + keying (depends on severity)
9. **PC-009** — pixelmatch/reg-cli (supplementary to vision comparison)
10. **PC-012** — Wait-for-stable (cross-cutting, before examples)
11. **PC-014** — CSS masking (cross-cutting, before examples)
12. **PC-015** — Cost/latency formula (guidance, before examples)
13. **PC-011** — Security section (must exist before examples reference it)
14. **PC-010** — Two examples (integrates all patterns above)
15. **PC-013** — E2E journey spec (final skill smoke test)
16. **PC-016** — Agent skills list update
17. **PC-017** — Agent input contract
18. **PC-018** — Agent output contract
19. **PC-019** — Backward compatibility
20. **PC-020** — Screenshot metadata in agent
21. **PC-021** — Vision assertion pass/fail with reasoning
22. **PC-022** — Failed assertion report structure
23. **PC-023** — Baseline comparison flow
24. **PC-024** — Regression report with severity
25. **PC-025** — No-baseline handling
26. **PC-026** — Agent Browser compatibility statement
27. **PC-027** — e2e-testing cross-reference
28. **PC-028** — Glossary terms
29. **PC-029** — ADR content quality (vision + pixel keywords)
30. **PC-030** — CHANGELOG entry
31. **PC-031** — Skill file under 800 lines
32. **PC-032** — Validate agents
33. **PC-033** — Validate skills

## Doc Update Plan

| # | Doc File | Level | Action | Content Summary | Spec Ref |
|---|----------|-------|--------|-----------------|----------|
| 1 | `skills/visual-testing/SKILL.md` | Content | Create | Full visual testing skill — capture, assertions, regression, pixel-diff, security, examples | US-001-004 |
| 2 | `agents/e2e-runner.md` | Content | Extend | Visual mode workflow, input/output contracts, skills list | US-001-004 |
| 3 | `skills/e2e-testing/SKILL.md` | Content | Update | Cross-reference to visual-testing | Spec modules |
| 4 | `docs/adr/0042-vision-vs-pixel-comparison.md` | Documentation | Create | ADR: vision primary, pixel-diff supplementary | Decision #3 |
| 5 | `docs/domain/glossary.md` | Documentation | Extend | Visual Checkpoint, Visual Assertion, Visual Baseline, Visual Regression | Grill-me #7 |
| 6 | `CHANGELOG.md` | Documentation | Update | `feat: add autonomous visual testing to e2e-runner agent (BL-103)` | — |

## SOLID Assessment

**Verdict: PASS** — 2 LOW findings (advisory):
1. LOW: e2e-runner's multi-responsibility baseline predates BL-103; design correctly delegates visual concern to its own skill
2. LOW/WATCH: Shared input/output contract between visual and non-visual modes — gate visual-specific fields with explicit mode discriminant

## Robert's Oath Check

**Verdict: CLEAN** — 2 non-blocking observations:
1. AC-004.6's `.gitignore` patterns must be concrete and copy-pasteable, not advisory
2. Monitor visual-testing skill file size — stay under 800 lines; extract to runbooks if approaching limit

## Security Notes

**Verdict: CLEAR** — 2 LOW findings:
1. LOW: `.gitignore` guidance must include concrete patterns for `visual-artifacts/` and baseline directories
2. LOW: Post-authentication screenshots may contain PII beyond form inputs — recommend synthetic test users for authenticated visual testing

## Rollback Plan

Reverse dependency order (undo in this order if implementation fails):

1. Revert `CHANGELOG.md` entry
2. Revert `docs/domain/glossary.md` (remove 4 terms)
3. Revert `skills/e2e-testing/SKILL.md` (remove cross-reference)
4. Revert `agents/e2e-runner.md` (restore original without visual mode)
5. Delete `skills/visual-testing/SKILL.md`
6. Delete `docs/adr/0042-vision-vs-pixel-comparison.md`

## Phase Summary

### Design Reviews

| Review Type | Verdict | Finding Count |
|-------------|---------|---------------|
| SOLID (uncle-bob) | PASS | 2 LOW |
| Robert | CLEAN | 2 observations |
| Security | CLEAR | 2 LOW |

### Adversary Findings

| Dimension | R1 Score | R2 Score | Verdict | Key Rationale |
|-----------|----------|----------|---------|---------------|
| Completeness | — | 78 | PASS | All 26 ACs mapped, CHANGELOG and ADR quality PCs added |
| Correctness | — | 75 | PASS | PC commands valid, coverage table correct |
| Fragility | 55 | 70 | PASS | Split monolithic PCs, added file size check |
| Dependency Order | — | 82 | PASS | ADR -> skill -> agent -> cross-refs -> validation |
| Testability | — | 75 | PASS | 33 PCs with granular verification |
| Rollback Safety | — | 85 | PASS | Content-layer, trivially reversible |
| SOLID / Clean Craft | — | 72 | PASS | Proper separation of concerns |
| Spec Fidelity | — | 80 | PASS | All constraints respected |

### File Changes Summary

| # | File | Action | Spec Ref |
|---|------|--------|----------|
| 1 | `docs/adr/0042-vision-vs-pixel-comparison.md` | Create | Decision #3 |
| 2 | `skills/visual-testing/SKILL.md` | Create | US-001-004 |
| 3 | `agents/e2e-runner.md` | Modify | US-001-004 |
| 4 | `skills/e2e-testing/SKILL.md` | Modify | Cross-ref |
| 5 | `docs/domain/glossary.md` | Modify | Grill-me #7 |
| 6 | `CHANGELOG.md` | Modify | — |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-02-bl103-visual-testing/spec.md` | Full spec |
| `docs/specs/2026-04-02-bl103-visual-testing/design.md` | Full design + Phase Summary |
