---
name: coding-standards
description: Universal coding standards, best practices, and patterns for TypeScript, JavaScript, React, and Node.js development.
origin: ECC
---

# Coding Standards

## When to Activate

- Starting/reviewing code for quality
- Enforcing naming, formatting, structural consistency

## Principles

- **Readability First**: Clear names, self-documenting code, consistent formatting
- **KISS**: Simplest solution that works, no premature optimization
- **DRY**: Extract common logic, share utilities
- **YAGNI**: Don't build before needed, start simple

## TypeScript/JavaScript

### Naming

```typescript
// Variables: descriptive camelCase
const marketSearchQuery = 'election'
const isUserAuthenticated = true

// Functions: verb-noun pattern
async function fetchMarketData(marketId: string) { }
function calculateSimilarity(a: number[], b: number[]) { }
function isValidEmail(email: string): boolean { }
```

### Immutability (CRITICAL)

```typescript
const updatedUser = { ...user, name: 'New Name' }
const updatedArray = [...items, newItem]
// NEVER: user.name = 'x' or items.push(x)
```

### Error Handling

```typescript
async function fetchData(url: string) {
  try {
    const response = await fetch(url)
    if (!response.ok) throw new Error(`HTTP ${response.status}: ${response.statusText}`)
    return await response.json()
  } catch (error) {
    console.error('Fetch failed:', error)
    throw new Error('Failed to fetch data')
  }
}
```

### Async/Await

```typescript
// Parallel when possible
const [users, markets, stats] = await Promise.all([fetchUsers(), fetchMarkets(), fetchStats()])
```

### Type Safety

```typescript
interface Market {
  id: string
  name: string
  status: 'active' | 'resolved' | 'closed'
  created_at: Date
}
// NEVER use 'any'
```

## React

### Component Structure

```typescript
interface ButtonProps {
  children: React.ReactNode
  onClick: () => void
  disabled?: boolean
  variant?: 'primary' | 'secondary'
}

export function Button({ children, onClick, disabled = false, variant = 'primary' }: ButtonProps) {
  return <button onClick={onClick} disabled={disabled} className={`btn btn-${variant}`}>{children}</button>
}
```

### State Updates

```typescript
setCount(prev => prev + 1)  // Functional update (correct)
// NOT: setCount(count + 1)  // Can be stale
```

### Conditional Rendering

```typescript
{isLoading && <Spinner />}
{error && <ErrorMessage error={error} />}
{data && <DataDisplay data={data} />}
```

## API Design

```
GET    /api/markets              # List
POST   /api/markets              # Create
GET    /api/markets/:id          # Get
PUT    /api/markets/:id          # Update (full)
PATCH  /api/markets/:id          # Update (partial)
DELETE /api/markets/:id          # Delete
GET    /api/markets?status=active&limit=10&offset=0
```

### Response Format

```typescript
interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
  meta?: { total: number; page: number; limit: number }
}
```

### Input Validation

```typescript
import { z } from 'zod'

const CreateMarketSchema = z.object({
  name: z.string().min(1).max(200),
  description: z.string().min(1).max(2000),
  endDate: z.string().datetime(),
  categories: z.array(z.string()).min(1)
})
```

## File Organization

```
src/
├── app/api/           # API routes
├── components/ui/     # Generic UI components
├── hooks/             # Custom hooks
├── lib/utils/         # Helpers
├── types/             # TypeScript types
└── styles/            # Global styles
```

File naming: `Button.tsx` (components), `useAuth.ts` (hooks), `formatDate.ts` (utils).

## Comments

```typescript
// GOOD: Explain WHY
// Use exponential backoff to avoid overwhelming the API during outages
const delay = Math.min(1000 * Math.pow(2, retryCount), 30000)

// BAD: State the obvious
// Increment counter
count++
```

## Code Smells

- **Long functions**: Split into < 50 lines each
- **Deep nesting**: Use early returns
- **Magic numbers**: Use named constants (`const MAX_RETRIES = 3`)

## Testing (AAA Pattern)

```typescript
test('calculates similarity correctly', () => {
  // Arrange
  const vector1 = [1, 0, 0], vector2 = [0, 1, 0]
  // Act
  const similarity = calculateCosineSimilarity(vector1, vector2)
  // Assert
  expect(similarity).toBe(0)
})
```

Test names: `'returns empty array when no markets match query'` (not `'works'`).
