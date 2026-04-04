# Spec: BL-103 Autonomous Visual Testing Integration

## Problem Statement

ECC's e2e-runner agent uses Playwright for browser automation but lacks vision-based UI validation. Users actively request AI agents that can launch apps, interact with UIs, and validate visual results (Claude Code issue #31532). The agent can already capture screenshots via Playwright, but has no guidance for using Claude's vision capabilities to analyze them, nor patterns for visual regression detection across test runs.

## Research Summary

- **Round-trip screenshot testing** is the proven pattern: agent captures screenshots via Playwright, reads them via Claude's Read tool for vision analysis, and self-corrects — endorsed by Anthropic best practices as "the single highest-leverage thing you can do"
- **AI visual diffing beats pixel-matching** for regression detection — layout-aware semantic analysis reduces false positives from anti-aliasing and font rendering while catching real structural regressions
- **Emerging tools**: Zerostep (ai() for Playwright), Playwright Mind (.aiAssert), Browser Use (50K+ stars), Playwright MCP (snapshot + vision modes)
- **Best practice**: Use natural-language test descriptions rather than brittle CSS selectors; capture baselines after design approval; be selective about where to pay vision latency cost
- **Key pitfalls**: Over-reliance on vision for everything (snapshot/accessibility-tree mode is faster); non-deterministic rendering causes false failures; each vision call adds ~2-5s latency and ~1K tokens cost
- **Dual-mode strategy consensus**: accessibility tree for 80% of cases, vision for the 20% where "looks right" matters
- **Prior art**: Applitools Eyes (commercial, Visual AI), Percy/BrowserStack (cloud), Chromatic (Storybook), pixelmatch (open-source pixel-diff)

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | Use Claude Read tool for vision analysis (not separate API) | Claude Code's Read tool already supports images natively — no external integration needed | No |
| 2 | Create separate visual-testing skill | Keep e2e-testing focused; visual-testing can be reused by other agents; follows "many small files" convention | No |
| 3 | Vision-based comparison as primary, pixel-diff as supplementary | AI vision provides semantic understanding of changes; pixel-diff tools (pixelmatch) for CI pipelines when exact pixel comparison is needed | Yes |
| 4 | Wait-for-stable pattern for dynamic content | Wait for animations/transitions to complete before capture, rather than masking or post-hoc filtering | No |
| 5 | No prescribed limits on vision usage | Document cost/latency tradeoffs but let users decide per test | No |

## User Stories

### US-001: Screenshot Capture Protocol

**As a** developer using the e2e-runner agent, **I want** structured screenshot capture at key interaction points with metadata, **so that** I have a reliable visual record of UI state for debugging and visual assertions.

#### Acceptance Criteria

- AC-001.1: Given the e2e-runner agent definition, when the `visual: true` field is specified in the journey spec, then the agent instructs tests to capture screenshots at navigation, form submission, and assertion points
- AC-001.2: Given a screenshot is captured, when stored as an artifact, then it includes metadata: URL, viewport dimensions, timestamp, and test step name
- AC-001.3: Given the visual-testing skill, when visual mode is active, then it provides a `VisualCapture` helper pattern that wraps `page.screenshot()` with metadata collection
- AC-001.4: Given screenshots are captured, when the test completes, then artifacts are organized in a `visual-artifacts/` subdirectory with a `manifest.json` listing all captures and their metadata
- AC-001.5: Given the visual-testing skill, when the manifest schema is documented, then it includes a concrete JSON schema example with typed fields: `timestamp` (ISO 8601), `viewport` (`{width: number, height: number}`), `url` (string), `stepName` (string), `filePath` (string)

#### Dependencies

- Depends on: none

### US-002: Vision-Based Assertions

**As a** developer using the e2e-runner agent, **I want** to write natural language assertions validated by Claude's vision capabilities against screenshots, **so that** I can verify complex UI states without brittle selector-based checks.

#### Acceptance Criteria

- AC-002.1: Given a screenshot and a natural language assertion (e.g., "the login form should show a red error message"), when the e2e-runner processes the visual assertion, then it reads the screenshot via the Read tool and reports pass/fail with reasoning
- AC-002.2: Given a visual assertion fails, when the report is generated, then it includes the screenshot path, the assertion text, and Claude's explanation of why it failed
- AC-002.3: Given the visual-testing skill, when visual assertions are documented, then it provides a `visualAssert()` pattern showing how to integrate vision checks into test flows
- AC-002.4: Given a visual assertion, when the Read tool fails to process the screenshot, then the assertion is marked as "skipped" with a warning, not as a failure

#### Dependencies

- Depends on: US-001

### US-003: Visual Regression Detection

**As a** developer using the e2e-runner agent, **I want** to compare screenshots across test runs and detect unexpected visual changes, **so that** I catch unintended UI regressions before they reach production.

#### Acceptance Criteria

- AC-003.1: Given a baseline screenshot exists from a previous run, when a new screenshot is captured at the same checkpoint, then the agent compares them using Claude's vision and reports whether significant visual differences exist
- AC-003.2: Given visual differences are detected, when the comparison completes, then the agent generates a report listing: checkpoint name, baseline path, current path, description of changes, and severity (cosmetic/functional/breaking)
- AC-003.3: Given no baseline exists for a checkpoint, when a screenshot is captured, then it is saved as the new baseline with no regression reported
- AC-003.4: Given the visual-testing skill, when visual regression is documented, then it includes a baseline management section explaining how to update, approve, or reject baseline changes
- AC-003.5: Given the visual-testing skill, when pixel-diff tooling is documented, then it provides pixelmatch/reg-cli patterns as a supplementary approach for CI pipelines
- AC-003.6: Given the visual-testing skill, when severity classification is documented, then it defines explicit criteria for each level: **cosmetic** (spacing, font rendering, color shade), **functional** (missing elements, wrong content, layout breakage), **breaking** (page crash, blank render, navigation failure)
- AC-003.7: Given the visual-testing skill, when baseline management is documented, then it specifies the baseline keying strategy: baselines are keyed by `{test-name}/{checkpoint-id}/{browser-viewport}` to handle multi-browser and multi-viewport configurations

#### Dependencies

- Depends on: US-001

### US-004: Agent and Skill Integration

**As a** developer using the e2e-runner agent, **I want** a `visual: true` option that activates visual testing mode and comprehensive skill documentation with examples, **so that** visual testing is opt-in and easy to learn.

#### Acceptance Criteria

- AC-004.1: Given the e2e-runner agent's input contract, when `visual: true` is specified in the journey spec, then the agent activates screenshot capture, vision assertions, and regression detection
- AC-004.2: Given the e2e-runner agent, when visual mode is not specified, then behavior is identical to today (no visual testing overhead)
- AC-004.3: Given the visual-testing skill, when visual testing examples are added, then at least two complete examples exist: (a) a login flow with vision assertion and (b) a dashboard page with regression detection
- AC-004.4: Given the e2e-runner agent's output contract, when visual mode is active, then `visual_results` is added containing: screenshots captured count, vision assertions passed/failed/skipped, regressions detected count
- AC-004.5: Given the e2e-runner agent's frontmatter, when visual testing is integrated, then the description is updated and `skills` list includes `visual-testing`
- AC-004.6: Given the visual-testing skill, when security is documented, then it includes warnings about PII in screenshots, credential exposure in baselines, and guidance on .gitignore patterns
- AC-004.7: Given the visual-testing skill, when the wait-for-stable pattern is documented, then it includes examples of waiting for animations/transitions to complete before screenshot capture
- AC-004.8: Given the visual-testing skill, when the skill is complete, then it includes one complete end-to-end example journey spec with `visual: true` that can be used as a smoke test for the agent's visual testing workflow
- AC-004.9: Given the visual-testing skill, when dynamic content handling is documented, then it includes patterns for excluding regions from comparison (e.g., timestamps, ads, user-specific content) via CSS-based masking before screenshot capture
- AC-004.10: Given the visual-testing skill, when cost/latency guidance is documented, then it includes an estimation formula: `vision_overhead = num_checkpoints * ~3s latency + num_checkpoints * ~1K tokens`, with a recommendation to keep visual checkpoints under 10 per journey for interactive use

#### Dependencies

- Depends on: US-001, US-002, US-003

## Affected Modules

| Module | Layer | Change Type |
|--------|-------|-------------|
| `agents/e2e-runner.md` | Content (agent) | Extend — add visual mode workflow, update frontmatter, update input/output contracts |
| `skills/visual-testing/SKILL.md` | Content (skill) — **NEW** | Create — screenshot capture patterns, vision assertions, regression detection, pixel-diff guidance, security warnings |
| `skills/e2e-testing/SKILL.md` | Content (skill) | Minor update — cross-reference to visual-testing skill |
| `docs/adr/` | Documentation | New ADR — vision-vs-pixel comparison decision |
| `docs/domain/bounded-contexts.md` | Documentation | Add visual testing glossary terms |

## Constraints

- No Rust code changes — content-layer only
- e2e-runner agent's existing behavior must be unchanged when `visual: true` is not specified
- No new tools required in agent frontmatter (Read already supports images)
- No external service dependencies beyond Claude itself
- Screenshots containing PII/credentials must never be committed

## Non-Requirements

- Baseline management infrastructure (Git LFS, CI artifact storage) — document the concept only
- Separate `/e2e` slash command — the e2e-runner is invoked directly
- Model tier change for vision work — stay on Sonnet
- Automated baseline approval workflow

## E2E Boundaries Affected

| Port/Adapter | Change Type | E2E Consequence |
|--------------|-------------|-----------------|
| Playwright screenshot API | Existing — enhanced usage | No new boundary; structured capture instead of ad-hoc |
| Claude Read tool (vision) | Existing — new usage pattern | No new boundary; uses existing tool for image analysis |
| File system (baselines) | New convention | Baseline storage adds state across test runs |

## Doc Impact Assessment

| Change Type | Level | Target Doc | Action |
|-------------|-------|------------|--------|
| New skill | Content | `skills/visual-testing/SKILL.md` | Create |
| Agent update | Content | `agents/e2e-runner.md` | Extend |
| Cross-reference | Content | `skills/e2e-testing/SKILL.md` | Add pointer |
| ADR | Documentation | `docs/adr/NNNN-vision-vs-pixel-comparison.md` | Create |
| Glossary | Documentation | `docs/domain/bounded-contexts.md` | Add 4 terms |
| CLAUDE.md | Onboarding | `CLAUDE.md` | No changes needed |

## Open Questions

None — all resolved during grill-me interview.

## Phase Summary

### Grill-Me Decisions

| # | Question | Answer | Source |
|---|----------|--------|--------|
| 1 | Scope boundaries | Include pixel-diff guidance alongside vision. New visual-testing skill. No Rust, no /e2e command, no baseline infra | User |
| 2 | Edge cases | Wait-for-stable pattern before capture | User |
| 3 | Test strategy | Provide both patterns (DOM + vision), let users decide per test | User |
| 4 | Performance | Document cost/latency tradeoffs only, no prescribed limits | Recommended |
| 5 | Security | Add security warnings section about PII in screenshots | Recommended |
| 6 | Breaking changes | No breaking changes — purely additive | Recommended |
| 7 | Domain concepts | Add 4 terms to docs/domain glossary | User |
| 8 | ADR decisions | ADR for vision-vs-pixel comparison decision | User |

### User Stories

| ID | Title | AC Count | Dependencies |
|----|-------|----------|--------------|
| US-001 | Screenshot Capture Protocol | 5 | none |
| US-002 | Vision-Based Assertions | 4 | US-001 |
| US-003 | Visual Regression Detection | 7 | US-001 |
| US-004 | Agent and Skill Integration | 10 | US-001, US-002, US-003 |

### Acceptance Criteria

| AC ID | Description | Source US |
|-------|-------------|----------|
| AC-001.1 | Visual mode triggers screenshot capture at key points | US-001 |
| AC-001.2 | Screenshots include metadata (URL, viewport, timestamp, step) | US-001 |
| AC-001.3 | VisualCapture helper pattern in skill | US-001 |
| AC-001.4 | Artifacts organized in visual-artifacts/ with manifest.json | US-001 |
| AC-001.5 | Manifest JSON schema with typed fields | US-001 |
| AC-002.1 | Vision assertion via Read tool, pass/fail with reasoning | US-002 |
| AC-002.2 | Failed assertion report includes path, text, explanation | US-002 |
| AC-002.3 | visualAssert() pattern in skill | US-002 |
| AC-002.4 | Graceful skip on Read tool failure | US-002 |
| AC-003.1 | Vision-based comparison against baseline | US-003 |
| AC-003.2 | Regression report with severity classification | US-003 |
| AC-003.3 | New baseline when none exists | US-003 |
| AC-003.4 | Baseline management documentation | US-003 |
| AC-003.5 | pixelmatch/reg-cli supplementary patterns | US-003 |
| AC-003.6 | Severity criteria: cosmetic/functional/breaking | US-003 |
| AC-003.7 | Baseline keying: {test-name}/{checkpoint-id}/{browser-viewport} | US-003 |
| AC-004.1 | visual: true activates all visual features | US-004 |
| AC-004.2 | Backward compatibility when visual not specified | US-004 |
| AC-004.3 | Two complete examples (login + dashboard) | US-004 |
| AC-004.4 | visual_results in output contract | US-004 |
| AC-004.5 | Updated frontmatter with visual-testing skill | US-004 |
| AC-004.6 | Security warnings (PII, credentials, .gitignore) | US-004 |
| AC-004.7 | Wait-for-stable pattern examples | US-004 |
| AC-004.8 | Complete end-to-end example journey spec | US-004 |
| AC-004.9 | Dynamic content CSS masking patterns | US-004 |
| AC-004.10 | Cost/latency estimation formula | US-004 |

### Adversary Findings

| Dimension | R1 Score | R2 Score | Verdict | Key Rationale |
|-----------|----------|----------|---------|---------------|
| Ambiguity | 55 | 72 | PASS | Fixed by AC-001.5 (schema), AC-003.6 (severity), AC-003.7 (keying) |
| Edge Cases | 50 | 70 | PASS | Fixed by AC-004.9 (masking), AC-003.7 (viewport), AC-003.6 (severity) |
| Scope | 72 | 72 | PASS | Well-bounded content-layer change |
| Dependencies | 78 | 78 | PASS | No external dependencies beyond Claude |
| Testability | 45 | 68 | PASS | Fixed by AC-004.8 (example journey), AC-001.5 (typed schema) |
| Decisions | 70 | 70 | PASS | Missing baseline keying fixed by AC-003.7 |
| Rollback | 80 | 80 | PASS | Content-layer + opt-in = trivial rollback |

### Artifacts Persisted

| File Path | Section Written |
|-----------|-----------------|
| `docs/specs/2026-04-02-bl103-visual-testing/spec.md` | Full spec + Phase Summary |
