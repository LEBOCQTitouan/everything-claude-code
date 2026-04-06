---
name: visual-testing
description: Vision-based visual testing patterns for Playwright — screenshot capture, AI-powered assertions, regression detection, pixel-diff guidance, baseline management, and security considerations.
origin: ECC
---

# Visual Testing Patterns

Vision-based UI validation using Claude's image understanding. Extends Playwright E2E tests with screenshot capture, natural-language assertions, and visual regression detection.

## Screenshot Capture

### VisualCapture Helper

```typescript
interface VisualCaptureMetadata {
  timestamp: string; url: string; viewport: { width: number; height: number }
  stepName: string; filePath: string
}

async function visualCapture(page: Page, stepName: string, outputDir = 'visual-artifacts'): Promise<VisualCaptureMetadata> {
  const filePath = `${outputDir}/${stepName}.png`
  await page.screenshot({ path: filePath, fullPage: false })
  return { timestamp: new Date().toISOString(), url: page.url(), viewport: page.viewportSize() ?? { width: 1280, height: 720 }, stepName, filePath }
}
```

### Wait-for-Stable

```typescript
async function waitForStable(page: Page): Promise<void> {
  await page.waitForLoadState('networkidle')
  await page.evaluate(() => {
    return new Promise<void>(resolve => {
      const animations = document.getAnimations()
      if (animations.length === 0) return resolve()
      Promise.all(animations.map(a => a.finished)).then(() => resolve())
    })
  })
}
```

### Dynamic Content Masking

```typescript
async function maskDynamicContent(page: Page, selectors: string[]): Promise<void> {
  for (const selector of selectors) {
    await page.evaluate((sel) => {
      document.querySelectorAll(sel).forEach(el => { (el as HTMLElement).style.visibility = 'hidden' })
    }, selector)
  }
}
```

## Vision-Based Assertions

The agent reads captured PNG via Read tool and evaluates natural-language assertions against what it observes. If Read fails, assertion is **skipped** (never fails on infra issues).

Failure report includes: screenshot path, assertion text, explanation, severity (cosmetic/functional/breaking).

## Visual Regression Detection

### Flow

1. Capture screenshot at named checkpoint
2. Look up baseline for that checkpoint
3. Compare using Claude's vision (read both images)
4. Report severity classification

No baseline = save current as new baseline.

### Baseline Keying

```
visual-baselines/{test-name}/{checkpoint-id}/{browser-viewport}.png
```

### Severity

| Level | Criteria | Action |
|-------|----------|--------|
| cosmetic | Spacing, font rendering, color shade | Log and review |
| functional | Missing elements, wrong content, layout breakage | Investigate |
| breaking | Page crash, blank render, critical content missing | Block |

## Pixel-Diff (Supplementary)

```typescript
import pixelmatch from 'pixelmatch'
import { PNG } from 'pngjs'

function compareScreenshots(baselinePath: string, currentPath: string, diffPath: string, threshold = 0.1) {
  const baseline = PNG.sync.read(fs.readFileSync(baselinePath))
  const current = PNG.sync.read(fs.readFileSync(currentPath))
  const diff = new PNG({ width: baseline.width, height: baseline.height })
  const diffPixels = pixelmatch(baseline.data, current.data, diff.data, baseline.width, baseline.height, { threshold })
  fs.writeFileSync(diffPath, PNG.sync.write(diff))
  return { diffPixels, totalPixels: baseline.width * baseline.height, diffPercent: (diffPixels / (baseline.width * baseline.height)) * 100 }
}
```

| Scenario | Approach |
|----------|----------|
| Agent-driven interactive | Vision (Claude Read tool) |
| CI/CD automated gate | Pixel-diff (pixelmatch/reg-cli) |
| Precise layout measurement | Pixel-diff |
| Cross-browser comparison | Vision (handles rendering diffs better) |

## Security

- **Never commit screenshots containing PII** to version control
- Add `visual-artifacts/`, `visual-baselines/`, `visual-diffs/` to `.gitignore`
- Use synthetic test accounts with non-PII data

## Cost Guidance

| Checkpoints | Latency | Tokens |
|-------------|---------|--------|
| 5 | ~15s | ~5K |
| 10 | ~30s | ~10K |
| 20 | ~60s | ~20K |

Keep under 10 checkpoints per journey for interactive sessions. Focus on critical decision points (login, checkout, landing).

## Journey Spec

```json
{
  "name": "checkout-visual-validation",
  "visual": true,
  "scenarios": [{
    "name": "happy-path-checkout",
    "steps": [
      { "action": "navigate", "url": "/products", "checkpoint": "product-listing" },
      { "action": "click", "target": "[data-testid='checkout-btn']", "checkpoint": "checkout-form" },
      { "action": "click", "target": "[data-testid='place-order']", "checkpoint": "order-confirmation" }
    ],
    "visual_assertions": [
      { "checkpoint": "product-listing", "assert": "Product cards in grid with images, titles, prices" },
      { "checkpoint": "order-confirmation", "assert": "Order confirmation with order number displayed" }
    ]
  }],
  "baseline_dir": "visual-baselines/checkout"
}
```

When `visual: true`, capture screenshots at each `checkpoint`, evaluate `visual_assertions` via Read tool, compare against baselines.
