---
name: rate-limiter
category: resilience
tags: [resilience, rate-limiting, throttling, token-bucket]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Control the rate of requests to a resource, preventing overload and ensuring fair usage across consumers by enforcing a maximum throughput.

## Problem

Uncontrolled request rates can overwhelm a service, cause resource exhaustion, violate API rate limits, or trigger upstream throttling. Without rate limiting, a burst of traffic from one consumer can degrade service for all others.

## Solution

Use a token bucket or sliding window algorithm to track and limit request rates. Each request consumes a token; tokens refill at a fixed rate. When the bucket is empty, requests are either rejected, queued, or delayed until tokens are available.

## Language Implementations

### Rust

```rust
use std::sync::Mutex;
use std::time::Instant;

struct TokenBucket {
    capacity: u32,
    tokens: Mutex<f64>,
    refill_rate: f64, // tokens per second
    last_refill: Mutex<Instant>,
}

impl TokenBucket {
    fn new(capacity: u32, refill_rate: f64) -> Self {
        Self {
            capacity,
            tokens: Mutex::new(capacity as f64),
            refill_rate,
            last_refill: Mutex::new(Instant::now()),
        }
    }

    fn try_acquire(&self) -> bool {
        let mut tokens = self.tokens.lock().unwrap();
        let mut last = self.last_refill.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(*last).as_secs_f64();
        *tokens = (*tokens + elapsed * self.refill_rate).min(self.capacity as f64);
        *last = now;
        if *tokens >= 1.0 { *tokens -= 1.0; true } else { false }
    }
}
```

### Go

```go
import "golang.org/x/time/rate"

limiter := rate.NewLimiter(rate.Limit(10), 20) // 10 req/s, burst 20

func Handler(w http.ResponseWriter, r *http.Request) {
    if !limiter.Allow() {
        http.Error(w, "rate limit exceeded", http.StatusTooManyRequests)
        return
    }
    // handle request
}
```

### Python

```python
import time

class TokenBucket:
    def __init__(self, capacity: int, refill_rate: float) -> None:
        self._capacity = capacity
        self._tokens = float(capacity)
        self._refill_rate = refill_rate
        self._last_refill = time.monotonic()

    def try_acquire(self) -> bool:
        now = time.monotonic()
        elapsed = now - self._last_refill
        self._tokens = min(self._capacity, self._tokens + elapsed * self._refill_rate)
        self._last_refill = now
        if self._tokens >= 1.0:
            self._tokens -= 1.0
            return True
        return False
```

### Typescript

```typescript
class TokenBucket {
  private tokens: number;
  private lastRefill: number;

  constructor(
    private readonly capacity: number,
    private readonly refillRate: number, // tokens per second
  ) {
    this.tokens = capacity;
    this.lastRefill = Date.now();
  }

  tryAcquire(): boolean {
    const now = Date.now();
    const elapsed = (now - this.lastRefill) / 1000;
    this.tokens = Math.min(this.capacity, this.tokens + elapsed * this.refillRate);
    this.lastRefill = now;
    if (this.tokens >= 1) { this.tokens--; return true; }
    return false;
  }
}
```

## When to Use

- When protecting APIs from abuse or accidental overload.
- When enforcing fair usage across multiple consumers.
- When complying with upstream API rate limits.

## When NOT to Use

- When all consumers are trusted and traffic is naturally bounded.
- When you need complex quota management (use a dedicated API gateway instead).

## Anti-Patterns

- Setting limits without monitoring rejection rates.
- Using only server-side limiting without client-side awareness of rate limits.
- Applying a global rate limit when per-consumer limits are needed.

## Related Patterns

- [resilience/bulkhead](bulkhead.md) -- limits concurrency rather than rate.
- [resilience/circuit-breaker](circuit-breaker.md) -- stops traffic entirely on repeated failures.
- [resilience/retry-backoff](retry-backoff.md) -- clients should back off when rate-limited.

## References

- Token bucket algorithm: https://en.wikipedia.org/wiki/Token_bucket
- **Rust**: `governor`, `tower::limit::RateLimit`
- **Go**: `golang.org/x/time/rate`
- **Python**: `limits`, `aiolimiter`, `slowapi` (FastAPI)
- **Java/Kotlin**: Resilience4j RateLimiter, Bucket4j
- **TypeScript**: `bottleneck`, `rate-limiter-flexible`
