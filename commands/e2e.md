---
description: Generate and run end-to-end tests with Playwright. Creates test journeys, runs tests, captures screenshots/videos/traces, and uploads artifacts.
---

# E2E Command

### Phase 0: Prompt Refinement

Before executing, analyze the user's input using the `prompt-optimizer` skill:
1. Identify intent and match to available ECC skills/commands/agents
2. Check for ambiguity or missing context
3. Rewrite the task description for clarity and specificity
4. Display the refined prompt to the user

If the refined prompt differs significantly, show both original and refined versions.
Proceed with the refined version unless the user objects.

**FIRST ACTION**: Unless `--skip-plan` is passed, call the `EnterPlanMode` tool immediately. This enters Claude Code plan mode which restricts tools to read-only exploration while you analyze the codebase and draft an E2E test plan. After presenting the plan, call `ExitPlanMode` to proceed with execution after user approval.

This command orchestrates E2E test generation and execution across multiple agents. The command itself is the orchestrator — it delegates to specialized agents per phase.

## Orchestrator Responsibilities

- **Phase sequencing** — execute phases in order, gate on failures
- **Journey isolation** — one `e2e-runner` invocation per journey (1-agent-per-task)
- **Cross-review** — delegate generated tests to `code-reviewer` for quality
- **Commit enforcement** — commit after each journey and each review fix
- **Recap production** — summarize results across all journeys

## What This Command Does

0. **Plan** — Analyze codebase, identify journeys, classify by risk, present manifest, wait for approval
1. **Generate + Execute** — Delegate to `e2e-runner` agent per journey, commit after each
2. **Code Review** — Delegate generated tests to `code-reviewer`, fix findings, commit each fix
3. **Recap Report** — Produce summary (journeys tested, pass/fail, flaky rate, artifacts, review findings)

## Commit Cadence

| Trigger | Commit Message |
|---------|---------------|
| After each journey generated + executed | `test: add E2E tests for <journey>` |
| After each code review fix | `fix: <review finding> in E2E tests` |

## Arguments

- `--skip-plan` — skip the planning phase and proceed directly to test generation

## When to Use

Use `/e2e` when:
- Testing critical user journeys (login, trading, payments)
- Verifying multi-step flows work end-to-end
- Testing UI interactions and navigation
- Validating integration between frontend and backend
- Preparing for production deployment

## How It Works

### Phase 0: Plan

1. Call `EnterPlanMode`
2. Explore the codebase (read-only) to identify critical user journeys
3. Classify journeys by risk: **HIGH** (financial, auth), **MEDIUM** (search, nav), **LOW** (UI polish)
4. Present journey manifest with scenarios per journey
5. Wait for user approval, then call `ExitPlanMode`

### Phase 1: Generate + Execute

For each approved journey (one at a time):
1. Invoke `e2e-runner` agent with journey spec (name, scenarios, risk level, target dir)
2. Agent generates Playwright tests using Page Object Model pattern
3. Agent runs tests across browsers, captures artifacts on failure
4. Agent returns structured results (files created, pass/fail, flaky tests, artifacts)
5. **Commit**: `test: add E2E tests for <journey>`

### Phase 2: Code Review

1. Invoke `code-reviewer` agent on all generated test files
2. Review for: test isolation, flaky patterns, hardcoded values, missing assertions, selector quality
3. For each finding: apply fix, then **commit**: `fix: <review finding> in E2E tests`

### Phase 3: Recap Report

Produce a summary across all journeys:
- Journeys tested (with risk levels)
- Pass/fail counts per journey
- Overall flaky rate
- Artifacts generated (screenshots, videos, traces, reports)
- Code review findings addressed

## Example Usage

```
User: /e2e Test the market search and trading flows

[Phase 0: Plan mode]
  Journey Manifest:
  1. market-search-and-view (HIGH) — search, filter, view details
  2. place-trade (HIGH) — select market, place order, confirm
  3. portfolio-overview (MEDIUM) — view positions, P&L

  Approve? [y/n]

User: y

[Phase 1: Generate + Execute]
  Journey 1/3: market-search-and-view (HIGH)
    → e2e-runner: generated 3 tests in tests/e2e/markets/search-and-view.spec.ts
    → 3/3 passed (4.2s), 0 flaky
    → commit: test: add E2E tests for market-search-and-view

  Journey 2/3: place-trade (HIGH)
    → e2e-runner: generated 4 tests in tests/e2e/trading/place-trade.spec.ts
    → 4/4 passed (6.1s), 0 flaky
    → commit: test: add E2E tests for place-trade

  Journey 3/3: portfolio-overview (MEDIUM)
    → e2e-runner: generated 2 tests in tests/e2e/portfolio/overview.spec.ts
    → 2/2 passed (3.0s), 0 flaky
    → commit: test: add E2E tests for portfolio-overview

[Phase 2: Code Review]
  → code-reviewer: reviewed 3 test files
  → Finding: hardcoded wait in place-trade.spec.ts → replaced with waitForResponse
  → commit: fix: replace hardcoded wait with waitForResponse in E2E tests

[Phase 3: Recap]
  Journeys:  3/3 passed
  Tests:     9 total, 9 passed, 0 failed
  Flaky:     0 (0%)
  Artifacts: 6 screenshots, 1 HTML report
  Review:    1 finding addressed
```

## Test Artifacts

When tests run, the following artifacts are captured:

**On All Tests:**
- HTML Report with timeline and results
- JUnit XML for CI integration

**On Failure Only:**
- Screenshot of the failing state
- Video recording of the test
- Trace file for debugging (step-by-step replay)
- Network logs
- Console logs

## Viewing Artifacts

```bash
# View HTML report in browser
npx playwright show-report

# View specific trace file
npx playwright show-trace artifacts/trace-abc123.zip

# Screenshots are saved in artifacts/ directory
open artifacts/search-results.png
```

## Flaky Test Detection

If a test fails intermittently:

```
⚠️  FLAKY TEST DETECTED: tests/e2e/markets/trade.spec.ts

Test passed 7/10 runs (70% pass rate)

Common failure:
"Timeout waiting for element '[data-testid="confirm-btn"]'"

Recommended fixes:
1. Add explicit wait: await page.waitForSelector('[data-testid="confirm-btn"]')
2. Increase timeout: { timeout: 10000 }
3. Check for race conditions in component
4. Verify element is not hidden by animation

Quarantine recommendation: Mark as test.fixme() until fixed
```

## Browser Configuration

Tests run on multiple browsers by default:
- ✅ Chromium (Desktop Chrome)
- ✅ Firefox (Desktop)
- ✅ WebKit (Desktop Safari)
- ✅ Mobile Chrome (optional)

Configure in `playwright.config.ts` to adjust browsers.

## CI/CD Integration

Add to your CI pipeline:

```yaml
# .github/workflows/e2e.yml
- name: Install Playwright
  run: npx playwright install --with-deps

- name: Run E2E tests
  run: npx playwright test

- name: Upload artifacts
  if: always()
  uses: actions/upload-artifact@v3
  with:
    name: playwright-report
    path: playwright-report/
```

## Best Practices

**DO:**
- ✅ Use Page Object Model for maintainability
- ✅ Use data-testid attributes for selectors
- ✅ Wait for API responses, not arbitrary timeouts
- ✅ Test critical user journeys end-to-end
- ✅ Run tests before merging to main
- ✅ Review artifacts when tests fail

**DON'T:**
- ❌ Use brittle selectors (CSS classes can change)
- ❌ Test implementation details
- ❌ Run tests against production
- ❌ Ignore flaky tests
- ❌ Skip artifact review on failures
- ❌ Test every edge case with E2E (use unit tests)

## Integration with Other Commands

- Use `/plan` to identify critical journeys to test and run TDD per phase
- Use `/e2e` for integration and user journey tests
- Use `/verify` to run code review, architecture review, and coverage analysis

## Related Agents

This command orchestrates:
- `e2e-runner` agent — test generation and execution (one invocation per journey)
- `code-reviewer` agent — review generated test files for quality and patterns

## Quick Commands

```bash
# Run all E2E tests
npx playwright test

# Run specific test file
npx playwright test tests/e2e/markets/search.spec.ts

# Run in headed mode (see browser)
npx playwright test --headed

# Debug test
npx playwright test --debug

# Generate test code
npx playwright codegen http://localhost:3000

# View report
npx playwright show-report
```
