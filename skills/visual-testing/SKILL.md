---
name: visual-testing
description: Vision-based visual testing patterns for Playwright — screenshot capture, AI-powered assertions, regression detection, pixel-diff guidance, baseline management, and security considerations.
origin: ECC
---

# Visual Testing Patterns

Vision-based UI validation using Claude's native image understanding. Extends Playwright E2E tests with screenshot capture, natural-language assertions, and visual regression detection.

## Screenshot Capture

### VisualCapture Helper

Wrap `page.screenshot()` with structured metadata collection:

```typescript
interface VisualCaptureMetadata {
  timestamp: string    // ISO 8601 format
  url: string
  viewport: { width: number, height: number }
  stepName: string
  filePath: string
}

async function visualCapture(
  page: Page,
  stepName: string,
  outputDir: string = 'visual-artifacts'
): Promise<VisualCaptureMetadata> {
  const filePath = `${outputDir}/${stepName}.png`
  await page.screenshot({ path: filePath, fullPage: false })

  return {
    timestamp: new Date().toISOString(),
    url: page.url(),
    viewport: page.viewportSize() ?? { width: 1280, height: 720 },
    stepName,
    filePath
  }
}
```

### Wait-for-Stable Pattern

Before capturing, ensure the page has settled — animations complete, network idle, dynamic content loaded:

```typescript
async function waitForStable(page: Page): Promise<void> {
  await page.waitForLoadState('networkidle')
  // Wait for CSS animations/transitions to complete
  await page.evaluate(() => {
    return new Promise<void>(resolve => {
      const animations = document.getAnimations()
      if (animations.length === 0) return resolve()
      Promise.all(animations.map(a => a.finished)).then(() => resolve())
    })
  })
}

// Usage: always call before visualCapture
await waitForStable(page)
const capture = await visualCapture(page, 'after-login')
```

### Dynamic Content Masking

For elements with inherently dynamic content (timestamps, counters, ads, user-specific data), mask them before capture to avoid false regression failures:

```typescript
async function maskDynamicContent(page: Page, selectors: string[]): Promise<void> {
  for (const selector of selectors) {
    await page.evaluate((sel) => {
      document.querySelectorAll(sel).forEach(el => {
        (el as HTMLElement).style.visibility = 'hidden'
      })
    }, selector)
  }
}

// Usage: mask timestamps and user avatars before capture
await maskDynamicContent(page, [
  '[data-testid="timestamp"]',
  '[data-testid="user-avatar"]',
  '.ad-banner'
])
await waitForStable(page)
const capture = await visualCapture(page, 'dashboard-masked')
```

### Artifact Organization

Screenshots are stored in `visual-artifacts/` with a `manifest.json`:

```
visual-artifacts/
  after-login.png
  dashboard-main.png
  checkout-confirm.png
  manifest.json
```

#### Manifest Schema

```json
{
  "captures": [
    {
      "timestamp": "2026-04-02T15:30:00.000Z",
      "url": "https://app.example.com/dashboard",
      "viewport": { "width": 1280, "height": 720 },
      "stepName": "dashboard-main",
      "filePath": "visual-artifacts/dashboard-main.png"
    }
  ],
  "testName": "dashboard-visual-test",
  "browser": "chromium",
  "createdAt": "2026-04-02T15:30:00.000Z"
}
```

All `timestamp` and `createdAt` fields use ISO 8601 format. The `viewport` field is an object with `width` and `height` as numbers. The `filePath` is relative to the project root.

## Vision-Based Assertions

### visualAssert Pattern

Use Claude's Read tool to analyze screenshots with natural-language assertions:

```typescript
// In the agent workflow (not Playwright code):
// 1. Capture the screenshot
await waitForStable(page)
const capture = await visualCapture(page, 'login-error')

// 2. The agent reads the screenshot via Read tool and evaluates:
//    Read: visual-artifacts/login-error.png
//    Assertion: "The login form should display a red error message
//               below the password field indicating invalid credentials"
//
// 3. Agent reports: PASS/FAIL with reasoning
```

The agent reads the captured PNG file using the Read tool (which supports image analysis natively) and evaluates the natural-language assertion against what it observes in the image.

### Assertion Report Structure

When a visual assertion fails, the report includes:
- **Screenshot path**: the file path to the captured image
- **Assertion text**: the natural-language expectation
- **Explanation**: Claude's reasoning for why the assertion failed
- **Severity**: cosmetic, functional, or breaking

### Graceful Degradation

If the Read tool fails to process a screenshot (file not found, corrupt image, or tool error), the visual assertion is marked as **skipped** with a warning — never as a failure. This prevents infrastructure issues from blocking test runs.

```
Visual assertion skipped: Could not read screenshot at visual-artifacts/login-error.png
Warning: Read tool returned an error. The assertion was not evaluated.
```

## Visual Regression Detection

### Baseline Comparison Flow

1. **Capture** the current screenshot at a named checkpoint
2. **Look up** the baseline for that checkpoint
3. **Compare** using Claude's vision: read both images and describe differences
4. **Report** findings with severity classification

When no baseline exists for a checkpoint (first run or new checkpoint), the current screenshot is saved as the new baseline with no regression reported.

### Baseline Keying Strategy

Baselines are keyed by a composite path to handle multi-browser and multi-viewport configurations:

```
visual-baselines/
  {test-name}/
    {checkpoint-id}/
      {browser-viewport}.png
```

Example:
```
visual-baselines/
  login-flow/
    after-submit/
      chromium-1280x720.png
      firefox-1280x720.png
      webkit-375x812.png
```

This ensures baselines from different browser/viewport combinations do not conflict.

### Baseline Management

- **Update baseline**: Replace the baseline screenshot after an intentional UI change has been reviewed and approved
- **Approve change**: Mark a detected regression as intentional — update the baseline to the current screenshot
- **Reject change**: Flag the regression as a bug — the baseline remains unchanged
- **Review cadence**: Periodically review baselines to prevent drift (baselines from 6+ months ago may mask gradual changes)

### Severity Classification

| Level | Criteria | Examples | Action |
|-------|----------|----------|--------|
| **cosmetic** | Spacing, font rendering, color shade, anti-aliasing differences | 1px padding shift, font weight variation, sub-pixel color change | Log and review — usually safe to ignore |
| **functional** | Missing elements, wrong content, layout breakage, alignment issues | Button missing, text truncated, sidebar overlapping content | Investigate — likely a real regression |
| **breaking** | Page crash, blank render, navigation failure, critical content missing | White screen, 404 page, login form completely absent | Block — must fix before shipping |

### Regression Report Structure

```markdown
## Visual Regression Report

### Checkpoint: dashboard-main
- **Baseline**: visual-baselines/dashboard-test/dashboard-main/chromium-1280x720.png
- **Current**: visual-artifacts/dashboard-main.png
- **Severity**: functional
- **Description**: The sidebar navigation is shifted 20px to the right,
  causing the main content area to be narrower. The "Settings" menu item
  is partially hidden behind the content panel.
```

## Pixel-Diff Tooling (Supplementary)

For CI pipelines where deterministic, automated comparison is needed, use pixel-diff tools alongside vision-based comparison:

### pixelmatch (Node.js)

```typescript
import pixelmatch from 'pixelmatch'
import { PNG } from 'pngjs'
import fs from 'fs'

function compareScreenshots(
  baselinePath: string,
  currentPath: string,
  diffPath: string,
  threshold: number = 0.1
): { diffPixels: number; totalPixels: number; diffPercent: number } {
  const baseline = PNG.sync.read(fs.readFileSync(baselinePath))
  const current = PNG.sync.read(fs.readFileSync(currentPath))
  const diff = new PNG({ width: baseline.width, height: baseline.height })

  const diffPixels = pixelmatch(
    baseline.data, current.data, diff.data,
    baseline.width, baseline.height,
    { threshold }
  )

  fs.writeFileSync(diffPath, PNG.sync.write(diff))

  const totalPixels = baseline.width * baseline.height
  return { diffPixels, totalPixels, diffPercent: (diffPixels / totalPixels) * 100 }
}
```

### reg-cli (Standalone)

```bash
# Compare directories of screenshots
npx reg-cli ./visual-artifacts ./visual-baselines ./visual-diffs \
  --report ./visual-report.html \
  --json ./visual-report.json \
  --threshold 0.01

# View the HTML report
open visual-report.html
```

### When to Use Pixel-Diff vs Vision

| Scenario | Recommended Approach |
|----------|---------------------|
| Agent-driven interactive testing | Vision (Claude Read tool) |
| CI/CD pipeline automated gate | Pixel-diff (pixelmatch/reg-cli) |
| Design review with stakeholders | Vision (natural-language reports) |
| Precise layout measurement | Pixel-diff (exact pixel counts) |
| Cross-browser comparison | Either — vision handles rendering differences better |

## Security Considerations

### PII in Screenshots

Screenshots may capture sensitive data visible on screen:
- **Passwords** in form fields (even if masked, the field presence reveals auth state)
- **Personal data** — names, emails, addresses, phone numbers rendered in UI
- **Session tokens** in URL bars or developer tool overlays
- **Post-authentication state** — user profiles, account details, transaction history

**Never commit screenshots containing PII or credentials to version control.**

### Recommended .gitignore Patterns

Add these entries to your project's `.gitignore` **before** the first visual test run:

```gitignore
# Visual testing artifacts (may contain PII)
visual-artifacts/
visual-baselines/
visual-diffs/
visual-report.html
visual-report.json
visual-artifacts/manifest.json
```

### Best Practices

- Use **synthetic test accounts** with non-PII data for authenticated visual testing flows
- Capture screenshots of **logged-out or anonymized** states when possible
- Review captured screenshots before sharing or uploading as CI artifacts
- Ensure baseline directories in CI artifact storage have appropriate access controls
- Rotate test credentials independently of production credentials

## Cost and Latency Guidance

Each visual checkpoint incurs overhead when using vision-based analysis:

```
vision_overhead = num_checkpoints * ~3s latency + num_checkpoints * ~1K tokens
```

| Checkpoints | Estimated Latency | Estimated Tokens |
|-------------|-------------------|------------------|
| 5 | ~15s | ~5K tokens |
| 10 | ~30s | ~10K tokens |
| 20 | ~60s | ~20K tokens |

For interactive agent sessions, keeping visual checkpoints under 10 per journey provides a good balance of coverage and responsiveness. For batch/CI runs, higher counts are acceptable since latency is less critical.

Vision assertions are most valuable at **critical decision points** — after login, at checkout, on landing pages — rather than on every page transition.

## Complete Examples

### Example 1: Login Flow with Vision Assertion

```typescript
import { test, expect } from '@playwright/test'
import { LoginPage } from '../pages/LoginPage'

test.describe('Login Visual Tests', () => {
  test('should show error state on invalid login', async ({ page }) => {
    const loginPage = new LoginPage(page)
    await loginPage.goto()

    // Capture baseline state
    await waitForStable(page)
    await visualCapture(page, 'login-initial')

    // Submit invalid credentials
    await loginPage.login('invalid@test.com', 'wrongpassword')
    await waitForStable(page)
    const capture = await visualCapture(page, 'login-error')

    // Vision assertion: agent reads login-error.png and validates
    // Assertion: "The login form should display a visible error message
    //            indicating authentication failed. The email field should
    //            still contain the entered email address."

    // DOM assertion for structural verification
    await expect(page.locator('[data-testid="error-message"]')).toBeVisible()
  })
})
```

### Example 2: Dashboard with Regression Detection

```typescript
test.describe('Dashboard Visual Regression', () => {
  test('dashboard layout matches baseline', async ({ page }) => {
    await page.goto('/dashboard')
    await waitForStable(page)

    // Mask dynamic content before capture
    await maskDynamicContent(page, [
      '[data-testid="last-updated"]',
      '[data-testid="notification-count"]',
      '[data-testid="user-greeting"]'
    ])

    const capture = await visualCapture(page, 'dashboard-main')

    // Agent compares capture against baseline:
    //   Baseline: visual-baselines/dashboard-test/dashboard-main/chromium-1280x720.png
    //   Current:  visual-artifacts/dashboard-main.png
    //
    // Reports: severity (cosmetic/functional/breaking) + description
    // If no baseline exists, saves current as new baseline.
  })
})
```

## End-to-End Journey Spec

A complete journey spec activating visual testing mode:

```json
{
  "name": "checkout-visual-validation",
  "visual": true,
  "scenarios": [
    {
      "name": "happy-path-checkout",
      "steps": [
        { "action": "navigate", "url": "/products", "checkpoint": "product-listing" },
        { "action": "click", "target": "[data-testid='add-to-cart']", "checkpoint": "cart-updated" },
        { "action": "navigate", "url": "/cart", "checkpoint": "cart-page" },
        { "action": "click", "target": "[data-testid='checkout-btn']", "checkpoint": "checkout-form" },
        { "action": "fill", "target": "[data-testid='email']", "value": "test@example.com" },
        { "action": "click", "target": "[data-testid='place-order']", "checkpoint": "order-confirmation" }
      ],
      "visual_assertions": [
        { "checkpoint": "product-listing", "assert": "Product cards are displayed in a grid layout with images, titles, and prices" },
        { "checkpoint": "checkout-form", "assert": "The checkout form has email, shipping address, and payment sections visible" },
        { "checkpoint": "order-confirmation", "assert": "An order confirmation message with an order number is displayed" }
      ]
    }
  ],
  "risk": "HIGH",
  "boundaries_crossed": ["HTTP API", "Database"],
  "baseline_dir": "visual-baselines/checkout"
}
```

When `visual: true` is set, the agent captures screenshots at each `checkpoint`, evaluates `visual_assertions` using the Read tool, and compares against baselines in `baseline_dir`.

## References

- [ADR 0042: Vision-vs-Pixel Comparison](../../docs/adr/0042-vision-vs-pixel-comparison.md)
- [e2e-testing skill](../e2e-testing/SKILL.md) — Playwright patterns, Page Object Model, CI/CD integration
- [Anthropic Best Practices: Round-Trip Screenshot Testing](https://code.claude.com/docs/en/best-practices)
