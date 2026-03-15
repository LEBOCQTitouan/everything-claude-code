---
name: typescript-testing
description: TypeScript testing patterns using Vitest/Jest, Testing Library, MSW for API mocking, and Playwright for E2E testing.
origin: ECC
---

# TypeScript Testing Patterns

Testing patterns for TypeScript applications using modern testing tools.

## When to Activate

- Writing tests for TypeScript code
- Setting up test infrastructure for TypeScript projects
- Testing React/Vue/Svelte components
- API mocking with MSW
- E2E testing with Playwright

## Vitest (Preferred)

### Setup

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'jsdom',
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html'],
      thresholds: { lines: 80, branches: 80 },
    },
  },
});
```

### Unit Tests

```typescript
import { describe, it, expect, vi } from 'vitest';

describe('UserService', () => {
  it('returns user by id', async () => {
    const repo = { findById: vi.fn().mockResolvedValue({ id: '1', name: 'John' }) };
    const service = new UserService(repo);

    const user = await service.getById('1');

    expect(user).toEqual({ id: '1', name: 'John' });
    expect(repo.findById).toHaveBeenCalledWith('1');
  });

  it('throws when user not found', async () => {
    const repo = { findById: vi.fn().mockResolvedValue(null) };
    const service = new UserService(repo);

    await expect(service.getById('99')).rejects.toThrow('not found');
  });
});
```

## Testing Library

```typescript
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';

describe('LoginForm', () => {
  it('submits credentials', async () => {
    const onSubmit = vi.fn();
    render(<LoginForm onSubmit={onSubmit} />);

    await userEvent.type(screen.getByLabelText('Email'), 'user@test.com');
    await userEvent.type(screen.getByLabelText('Password'), 'secret');
    await userEvent.click(screen.getByRole('button', { name: 'Sign in' }));

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith({
        email: 'user@test.com',
        password: 'secret',
      });
    });
  });
});
```

## MSW (Mock Service Worker)

```typescript
import { setupServer } from 'msw/node';
import { http, HttpResponse } from 'msw';

const server = setupServer(
  http.get('/api/users/:id', ({ params }) => {
    return HttpResponse.json({ id: params.id, name: 'John' });
  }),
  http.post('/api/users', async ({ request }) => {
    const body = await request.json();
    return HttpResponse.json({ id: '1', ...body }, { status: 201 });
  }),
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

## Playwright (E2E)

```typescript
import { test, expect } from '@playwright/test';

test('user can log in and see dashboard', async ({ page }) => {
  await page.goto('/login');
  await page.getByLabel('Email').fill('admin@test.com');
  await page.getByLabel('Password').fill('password');
  await page.getByRole('button', { name: 'Sign in' }).click();

  await expect(page).toHaveURL('/dashboard');
  await expect(page.getByRole('heading')).toHaveText('Welcome back');
});
```

## Running Tests

```bash
# Vitest
npx vitest                    # Watch mode
npx vitest run                # Single run
npx vitest --coverage         # With coverage

# Jest
npx jest
npx jest --coverage

# Playwright
npx playwright test
npx playwright test --ui      # Interactive mode
```

## Quick Reference

| Tool | Purpose |
|------|---------|
| Vitest | Fast unit/integration test runner |
| Jest | Established test runner |
| Testing Library | Component testing |
| MSW | API mocking |
| Playwright | E2E browser testing |
| Storybook | Visual component testing |
| c8/v8 | Coverage providers |
