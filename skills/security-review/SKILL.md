---
name: security-review
description: Use this skill when adding authentication, handling user input, working with secrets, creating API endpoints, or implementing payment/sensitive features. Provides comprehensive security checklist and patterns.
origin: ECC
---

# Security Review

## When to Activate

- Auth, user input, file uploads, API endpoints
- Secrets, payments, sensitive data, third-party APIs

## 1. Secrets Management

```typescript
const apiKey = process.env.OPENAI_API_KEY
if (!apiKey) throw new Error('OPENAI_API_KEY not configured')
// NEVER hardcode secrets in source code
```

## 2. Input Validation

```typescript
import { z } from 'zod'
const CreateUserSchema = z.object({
  email: z.string().email(),
  name: z.string().min(1).max(100),
  age: z.number().int().min(0).max(150)
})
```

File uploads: Check size (5MB max), type whitelist, extension whitelist.

## 3. SQL Injection Prevention

```typescript
// SAFE: parameterized
const { data } = await supabase.from('users').select('*').eq('email', userEmail)
await db.query('SELECT * FROM users WHERE email = $1', [userEmail])
// NEVER: string concatenation in SQL
```

## 4. Auth & Authorization

```typescript
// httpOnly cookies, NOT localStorage
res.setHeader('Set-Cookie', `token=${token}; HttpOnly; Secure; SameSite=Strict; Max-Age=3600`)
```

```sql
-- Supabase RLS
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
CREATE POLICY "Users view own data" ON users FOR SELECT USING (auth.uid() = id);
```

## 5. XSS Prevention

```typescript
import DOMPurify from 'isomorphic-dompurify'
const clean = DOMPurify.sanitize(html, { ALLOWED_TAGS: ['b', 'i', 'em', 'strong', 'p'] })
```

Configure CSP headers: `default-src 'self'`

## 6. CSRF: Verify X-CSRF-Token header, SameSite=Strict on cookies.

## 7. Rate Limiting

```typescript
const limiter = rateLimit({ windowMs: 15 * 60 * 1000, max: 100 })
app.use('/api/', limiter)
```

## 8. Data Exposure

- Never log passwords/tokens/secrets
- Generic errors to users, detailed in server logs
- No stack traces in responses

## 9. Dependencies

```bash
npm audit && npm audit fix && npm ci
```

## Pre-Deployment Checklist

| Area | Check |
|------|-------|
| Secrets | No hardcoded, all in env vars |
| Input | All validated with schemas |
| SQL | All queries parameterized |
| XSS | Content sanitized, CSP configured |
| Auth | httpOnly cookies, RLS enabled |
| Rate Limiting | All endpoints throttled |
| HTTPS | Enforced in production |
| Deps | Up to date, no vulnerabilities |
