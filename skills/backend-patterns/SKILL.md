---
name: backend-patterns
description: Backend architecture patterns, API design, database optimization, and server-side best practices for Node.js, Express, and Next.js API routes.
origin: ECC
---

# Backend Development Patterns

## When to Activate

- Designing REST/GraphQL APIs, service layers, repositories
- Optimizing DB queries, caching, background jobs
- Building middleware (auth, logging, rate limiting)

## API Design

```typescript
GET    /api/markets                 # List
GET    /api/markets/:id             # Get
POST   /api/markets                 # Create
PUT    /api/markets/:id             # Replace
PATCH  /api/markets/:id             # Update
DELETE /api/markets/:id             # Delete
GET    /api/markets?status=active&sort=volume&limit=20&offset=0
```

## Repository Pattern

```typescript
interface MarketRepository {
  findAll(filters?: MarketFilters): Promise<Market[]>
  findById(id: string): Promise<Market | null>
  create(data: CreateMarketDto): Promise<Market>
  update(id: string, data: UpdateMarketDto): Promise<Market>
  delete(id: string): Promise<void>
}
```

## Service Layer

```typescript
class MarketService {
  constructor(private marketRepo: MarketRepository) {}

  async searchMarkets(query: string, limit = 10): Promise<Market[]> {
    const embedding = await generateEmbedding(query)
    const results = await this.vectorSearch(embedding, limit)
    const markets = await this.marketRepo.findByIds(results.map(r => r.id))
    return markets.sort((a, b) => {
      const scoreA = results.find(r => r.id === a.id)?.score || 0
      const scoreB = results.find(r => r.id === b.id)?.score || 0
      return scoreA - scoreB
    })
  }
}
```

## Middleware

```typescript
export function withAuth(handler: NextApiHandler): NextApiHandler {
  return async (req, res) => {
    const token = req.headers.authorization?.replace('Bearer ', '')
    if (!token) return res.status(401).json({ error: 'Unauthorized' })
    try {
      req.user = await verifyToken(token)
      return handler(req, res)
    } catch { return res.status(401).json({ error: 'Invalid token' }) }
  }
}
```

## N+1 Prevention

```typescript
// BAD: N queries for N markets
for (const market of markets) { market.creator = await getUser(market.creator_id) }

// GOOD: Batch fetch
const creators = await getUsers(markets.map(m => m.creator_id))
const creatorMap = new Map(creators.map(c => [c.id, c]))
markets.forEach(m => { m.creator = creatorMap.get(m.creator_id) })
```

## Caching (Cache-Aside)

```typescript
class CachedMarketRepository implements MarketRepository {
  constructor(private baseRepo: MarketRepository, private redis: RedisClient) {}

  async findById(id: string): Promise<Market | null> {
    const cached = await this.redis.get(`market:${id}`)
    if (cached) return JSON.parse(cached)
    const market = await this.baseRepo.findById(id)
    if (market) await this.redis.setex(`market:${id}`, 300, JSON.stringify(market))
    return market
  }
}
```

## Error Handling

```typescript
class ApiError extends Error {
  constructor(public statusCode: number, public message: string, public isOperational = true) {
    super(message)
  }
}

export function errorHandler(error: unknown, req: Request): Response {
  if (error instanceof ApiError)
    return NextResponse.json({ success: false, error: error.message }, { status: error.statusCode })
  if (error instanceof z.ZodError)
    return NextResponse.json({ success: false, error: 'Validation failed', details: error.errors }, { status: 400 })
  console.error('Unexpected error:', error)
  return NextResponse.json({ success: false, error: 'Internal server error' }, { status: 500 })
}
```

## Retry with Backoff

```typescript
async function fetchWithRetry<T>(fn: () => Promise<T>, maxRetries = 3): Promise<T> {
  let lastError: Error
  for (let i = 0; i < maxRetries; i++) {
    try { return await fn() }
    catch (error) {
      lastError = error as Error
      if (i < maxRetries - 1) await new Promise(r => setTimeout(r, Math.pow(2, i) * 1000))
    }
  }
  throw lastError!
}
```

## Auth & RBAC

```typescript
export function verifyToken(token: string): JWTPayload {
  try { return jwt.verify(token, process.env.JWT_SECRET!) as JWTPayload }
  catch { throw new ApiError(401, 'Invalid token') }
}

const rolePermissions: Record<string, Permission[]> = {
  admin: ['read', 'write', 'delete', 'admin'],
  moderator: ['read', 'write', 'delete'],
  user: ['read', 'write']
}

export function requirePermission(permission: Permission) {
  return (handler: (req: Request, user: User) => Promise<Response>) =>
    async (request: Request) => {
      const user = await requireAuth(request)
      if (!rolePermissions[user.role].includes(permission)) throw new ApiError(403, 'Insufficient permissions')
      return handler(request, user)
    }
}
```

## Rate Limiting

```typescript
class RateLimiter {
  private requests = new Map<string, number[]>()

  async checkLimit(identifier: string, maxRequests: number, windowMs: number): Promise<boolean> {
    const now = Date.now()
    const recent = (this.requests.get(identifier) || []).filter(t => now - t < windowMs)
    if (recent.length >= maxRequests) return false
    recent.push(now)
    this.requests.set(identifier, recent)
    return true
  }
}
```

## Structured Logging

```typescript
class Logger {
  log(level: 'info' | 'warn' | 'error', message: string, context?: Record<string, unknown>) {
    console.log(JSON.stringify({ timestamp: new Date().toISOString(), level, message, ...context }))
  }
  info(msg: string, ctx?: Record<string, unknown>) { this.log('info', msg, ctx) }
  error(msg: string, err: Error, ctx?: Record<string, unknown>) {
    this.log('error', msg, { ...ctx, error: err.message, stack: err.stack })
  }
}
```
