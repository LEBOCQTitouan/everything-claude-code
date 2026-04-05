---
name: rate-limiting
category: api-design
tags: [api-design, rate-limiting, throttling, resilience]
languages: [all]
difficulty: intermediate
---

## Intent

Protect API servers from abuse and ensure fair resource allocation by limiting the number of requests a client can make within a time window.

## Problem

Without rate limiting, a single client (malicious or buggy) can monopolize server resources, degrade service for all users, and amplify costs. Denial-of-service attacks become trivial, and capacity planning becomes impossible.

## Solution

Apply rate limits using token bucket or sliding window algorithms. Communicate limits to clients via standard HTTP headers. Return 429 Too Many Requests when limits are exceeded, including a Retry-After header.

## Language Implementations

### HTTP Headers (Protocol-Agnostic)

```
# Response headers (IETF draft-polli-ratelimit-headers)
RateLimit-Limit: 100
RateLimit-Remaining: 42
RateLimit-Reset: 1620000060

# When exceeded
HTTP/1.1 429 Too Many Requests
Retry-After: 30
Content-Type: application/json

{
  "error": {
    "code": "RATE_LIMITED",
    "message": "Rate limit exceeded. Retry after 30 seconds.",
    "retry_after": 30
  }
}
```

### Token Bucket Algorithm (Pseudocode)

```
struct TokenBucket {
    capacity: int       # max tokens
    tokens: float       # current tokens
    refill_rate: float  # tokens per second
    last_refill: time
}

fn allow_request(bucket) -> bool:
    now = current_time()
    elapsed = now - bucket.last_refill
    bucket.tokens = min(bucket.capacity, bucket.tokens + elapsed * bucket.refill_rate)
    bucket.last_refill = now
    if bucket.tokens >= 1.0:
        bucket.tokens -= 1.0
        return true
    return false
```

### Tiered Limits

```yaml
rate_limits:
  anonymous:  { requests: 60,   window: 1m }
  free_tier:  { requests: 1000, window: 1h }
  pro_tier:   { requests: 10000, window: 1h }
  enterprise: { requests: 100000, window: 1h }
```

## When to Use

- Every public-facing API endpoint without exception.
- Internal APIs shared across teams or services.
- Expensive operations (search, ML inference) even behind authentication.

## When NOT to Use

- Internal single-service calls within a trusted mesh where backpressure mechanisms exist.
- Health check endpoints that monitoring systems poll frequently.

## Anti-Patterns

- Rate limiting only by IP address — shared NATs make this punish innocent users.
- Not including Retry-After in 429 responses — clients cannot implement polite backoff.
- Setting identical limits for read and write operations despite vastly different costs.
- Implementing rate limiting per-instance without shared state, allowing bypass via round-robin.

## Related Patterns

- [api-gateway](api-gateway.md) — centralize rate limiting at the gateway layer.
- [idempotency-keys](idempotency-keys.md) — safe retries after rate limit backoff.
- [rest-resources](rest-resources.md) — rate limits apply per resource or per client.

## References

- IETF RateLimit Headers: https://datatracker.ietf.org/doc/draft-ietf-httpapi-ratelimit-headers/
- Stripe Rate Limiting: https://stripe.com/docs/rate-limits
- Token Bucket Algorithm: https://en.wikipedia.org/wiki/Token_bucket
