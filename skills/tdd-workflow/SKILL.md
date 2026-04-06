---
name: tdd-workflow
description: Use this skill when writing new features, fixing bugs, or refactoring code. Enforces test-driven development with 80%+ coverage including unit, integration, and E2E tests.
origin: ECC
---

# Test-Driven Development Workflow

## When to Activate

- Writing new features, fixing bugs, refactoring
- Adding API endpoints or components

## Core Principles

1. **Tests BEFORE Code** — always write tests first
2. **80%+ coverage** (unit + integration + E2E), all edge cases, error scenarios
3. **Test types**: Unit (functions, utilities), Integration (APIs, DB), E2E (Playwright user flows)

## TDD Steps

1. **Write user journeys**: `As a [role], I want to [action], so that [benefit]`
2. **Generate test cases** covering happy path, edge cases, error paths, fallbacks
3. **Run tests — they should FAIL** (RED)
4. **Implement minimal code** using Transformation Priority Premise (GREEN):

| Priority | Transformation |
|----------|---------------|
| 1 | `{} -> nil` |
| 2 | `nil -> constant` |
| 3 | `constant -> variable` |
| 4 | `add computation` |
| 5 | `unconditional -> selection` |
| 6 | `scalar -> collection` |
| 7 | `selection -> iteration` |

Try lowest-priority transformation first. Jumping to complex transformations = over-engineering.

5. **Run tests — they should PASS** (GREEN)
6. **Refactor**: Remove duplication, improve naming, optimize
7. **Verify coverage**: `npm run test:coverage` (80%+)

## Unit Test Pattern

```typescript
import { render, screen, fireEvent } from '@testing-library/react'

describe('Button Component', () => {
  it('renders with correct text', () => {
    render(<Button>Click me</Button>)
    expect(screen.getByText('Click me')).toBeInTheDocument()
  })
  it('calls onClick when clicked', () => {
    const handleClick = jest.fn()
    render(<Button onClick={handleClick}>Click</Button>)
    fireEvent.click(screen.getByRole('button'))
    expect(handleClick).toHaveBeenCalledTimes(1)
  })
})
```

## API Integration Test

```typescript
describe('GET /api/markets', () => {
  it('returns markets successfully', async () => {
    const request = new NextRequest('http://localhost/api/markets')
    const response = await GET(request)
    expect(response.status).toBe(200)
    const data = await response.json()
    expect(data.success).toBe(true)
  })
  it('validates query parameters', async () => {
    const request = new NextRequest('http://localhost/api/markets?limit=invalid')
    const response = await GET(request)
    expect(response.status).toBe(400)
  })
})
```

## E2E Test (Playwright)

```typescript
test('user can search and filter markets', async ({ page }) => {
  await page.goto('/')
  await page.click('a[href="/markets"]')
  await expect(page.locator('h1')).toContainText('Markets')
  await page.fill('input[placeholder="Search markets"]', 'election')
  await page.waitForTimeout(600)
  const results = page.locator('[data-testid="market-card"]')
  await expect(results).toHaveCount(5, { timeout: 5000 })
})
```

## Mocking

```typescript
jest.mock('@/lib/supabase', () => ({
  supabase: { from: jest.fn(() => ({ select: jest.fn(() => ({ eq: jest.fn(() => Promise.resolve({ data: [{ id: 1, name: 'Test' }], error: null })) })) })) }
}))
```

## Coverage Thresholds

```json
{ "jest": { "coverageThresholds": { "global": { "branches": 80, "functions": 80, "lines": 80, "statements": 80 } } } }
```

## Best Practices

1. Write tests first (TDD)
2. One assert per test, descriptive names
3. Arrange-Act-Assert structure
4. Mock external dependencies only
5. Test edge cases and error paths
6. Keep tests fast (< 50ms unit, < 30s suite)
7. Independent tests (no shared state)
8. Semantic selectors (`data-testid`, not CSS classes)
