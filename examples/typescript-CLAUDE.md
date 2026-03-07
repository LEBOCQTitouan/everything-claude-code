# TypeScript — Project CLAUDE.md

> Generic TypeScript/Node.js project template.
> Copy this to your project root and customize for your project.

## Project Overview

**Stack:** TypeScript, Node.js, Jest/Vitest, ESLint, tsc

**Architecture:** [Describe your architecture — layered, hexagonal, feature-based, etc.]

## Critical Rules

### TypeScript Conventions

- Strict mode enabled — no `any`, no `ts-ignore` without explanation
- Prefer `type` over `interface` for unions and mapped types; use `interface` for object shapes that may be extended
- Explicit return types on all exported functions
- Use `unknown` instead of `any` for untyped external data; narrow with type guards
- No non-null assertions (`!`) — handle nullability explicitly

### Code Style

- No emojis in code, comments, or documentation
- Immutability first — `const` over `let`, prefer readonly properties
- No `console.log` in production code — use a logger
- Proper error handling — typed errors, not string matching
- Validate all external input (user input, API responses, env vars)

### Testing

- TDD: Write tests first
- 80% minimum coverage
- Unit tests for business logic and utilities
- Integration tests for external dependencies (DB, APIs)

### Security

- No hardcoded secrets — use environment variables
- Validate and sanitize all user inputs
- Parameterized queries — never string-interpolate SQL
- Keep dependencies up to date; audit with `npm audit`

## File Structure

```
src/
  domain/          # Business types, interfaces, entities
  services/        # Business logic
  repositories/    # Data access
  handlers/        # HTTP/CLI entry points
  lib/             # Shared utilities
  types/           # Global type declarations
tests/
  unit/
  integration/
```

## Key Patterns

### Typed Error Handling

```typescript
class AppError extends Error {
  constructor(
    message: string,
    public readonly code: string,
    public readonly statusCode: number = 500,
  ) {
    super(message)
    this.name = 'AppError'
  }
}

function isAppError(err: unknown): err is AppError {
  return err instanceof AppError
}
```

### Environment Variables

```typescript
import { z } from 'zod'

const envSchema = z.object({
  NODE_ENV: z.enum(['development', 'test', 'production']),
  DATABASE_URL: z.string().url(),
  PORT: z.coerce.number().default(3000),
})

export const env = envSchema.parse(process.env)
```

## Environment Variables

```bash
# Required
DATABASE_URL=
NODE_ENV=development

# Optional
PORT=3000
LOG_LEVEL=info
```

## Available Commands

```bash
# Build
npx tsc --noEmit        # Type check without emitting
npx tsc                 # Compile to dist/

# Test
npx jest                # Run all tests
npx jest --watch        # Watch mode
npx jest --coverage     # With coverage report

# Lint
npx eslint src/         # Lint source files
npx eslint src/ --fix   # Auto-fix

# Dev
npm run dev             # Start with ts-node or tsx watch
npm run build           # Production build
npm start               # Run compiled output
```

## ECC Workflow

```bash
/tdd           # Test-driven development workflow
/plan          # Implementation planning
/code-review   # Quality review
/build-fix     # Fix TypeScript / build errors
```

## Git Workflow

- Conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`
- Never commit to main directly
- PRs require review and passing CI
- CI: `tsc --noEmit`, `eslint`, `jest --coverage`
