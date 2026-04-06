---
name: e2e-testing
description: Playwright E2E testing patterns, Page Object Model, configuration, CI/CD integration, artifact management, and flaky test strategies.
origin: ECC
---

# E2E Testing Patterns

## Test File Organization

```
tests/e2e/
├── auth/ (login, logout, register)
├── features/ (browse, search, create)
├── api/ (endpoints)
├── fixtures/ (auth, data)
└── playwright.config.ts
```

## Page Object Model

```typescript
export class ItemsPage {
  readonly page: Page
  readonly searchInput: Locator
  readonly itemCards: Locator

  constructor(page: Page) {
    this.page = page
    this.searchInput = page.locator('[data-testid="search-input"]')
    this.itemCards = page.locator('[data-testid="item-card"]')
  }

  async goto() {
    await this.page.goto('/items')
    await this.page.waitForLoadState('networkidle')
  }

  async search(query: string) {
    await this.searchInput.fill(query)
    await this.page.waitForResponse(resp => resp.url().includes('/api/search'))
  }

  async getItemCount() { return await this.itemCards.count() }
}
```

### Humble Object Rules

- **Page Objects**: Locators + actions only, **zero assertions** (never `expect()`)
- **Test files**: Assertions + orchestration only, **zero raw selectors** (never `page.locator()` directly)
- `data-testid` literals only in POM files

## Test Structure

```typescript
test.describe('Item Search', () => {
  let itemsPage: ItemsPage

  test.beforeEach(async ({ page }) => {
    itemsPage = new ItemsPage(page)
    await itemsPage.goto()
  })

  test('should search by keyword', async () => {
    await itemsPage.search('test')
    expect(await itemsPage.getItemCount()).toBeGreaterThan(0)
  })
})
```

## Configuration

```typescript
export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  use: {
    baseURL: process.env.BASE_URL || 'http://localhost:3000',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'firefox', use: { ...devices['Desktop Firefox'] } },
    { name: 'mobile-chrome', use: { ...devices['Pixel 5'] } },
  ],
  webServer: { command: 'npm run dev', url: 'http://localhost:3000', reuseExistingServer: !process.env.CI },
})
```

## Flaky Test Patterns

```typescript
test.fixme(true, 'Flaky - Issue #123')  // Quarantine
test.skip(process.env.CI, 'Flaky in CI')  // Conditional skip
```

```bash
npx playwright test tests/search.spec.ts --repeat-each=10  # Identify flakiness
```

**Fixes**: Use auto-wait locators (not `page.click()`), wait for responses (not `waitForTimeout`), wait for stability before clicking animated elements.

## Artifacts

```typescript
await page.screenshot({ path: 'artifacts/after-login.png' })
await page.screenshot({ path: 'artifacts/full-page.png', fullPage: true })
// Config: video: 'retain-on-failure', trace: 'on-first-retry'
```

## Boundary Classification

Tag E2E journeys with boundaries crossed for risk scoring.

| Boundary | Risk Weight |
|----------|-------------|
| HTTP API | 1 |
| Database | 2 |
| External Auth | 3 |
| Payment Gateway | 4 |
| Message Queue | 2 |
| Third-party API | 3 |

Risk score = sum of weights. LOW (0-2): standard. MEDIUM (3-5): add retries. HIGH (6+): mock externals, add traces.

## Visual Testing

For vision-based UI validation, see the dedicated [visual-testing](../visual-testing/SKILL.md) skill.
