---
name: timeout
category: resilience
tags: [resilience, timeout, bounded-latency]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Bound the duration of any operation so that slow or unresponsive dependencies cannot hold resources indefinitely, ensuring predictable latency for callers.

## Problem

Without explicit timeouts, a call to a slow or hung service blocks the caller indefinitely, tying up threads, connections, and memory. Cascading waits propagate upstream and eventually exhaust system capacity.

## Solution

Wrap every external call with a deadline. If the operation does not complete within the allowed duration, cancel it and return an error. Use context-based cancellation (Go), async timeouts (Rust/Python), or AbortController (TypeScript) for cooperative cancellation.

## Language Implementations

### Rust

```rust
use tokio::time::{timeout, Duration};

async fn fetch_with_timeout(url: &str) -> Result<String, FetchError> {
    timeout(Duration::from_secs(5), async {
        reqwest::get(url).await?.text().await.map_err(FetchError::from)
    })
    .await
    .map_err(|_| FetchError::Timeout)?
}
```

### Go

```go
func FetchWithTimeout(ctx context.Context, url string) (string, error) {
    ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
    defer cancel()

    req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
    if err != nil {
        return "", err
    }
    resp, err := http.DefaultClient.Do(req)
    if err != nil {
        return "", err
    }
    defer resp.Body.Close()
    body, err := io.ReadAll(resp.Body)
    return string(body), err
}
```

### Python

```python
import asyncio

async def fetch_with_timeout(url: str, timeout_s: float = 5.0) -> str:
    async with asyncio.timeout(timeout_s):
        async with aiohttp.ClientSession() as session:
            async with session.get(url) as resp:
                return await resp.text()
```

### Typescript

```typescript
async function fetchWithTimeout(url: string, timeoutMs = 5000): Promise<string> {
  const controller = new AbortController();
  const id = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const resp = await fetch(url, { signal: controller.signal });
    return await resp.text();
  } finally {
    clearTimeout(id);
  }
}
```

## When to Use

- On every call to an external service (HTTP, gRPC, database, message broker).
- When you need bounded latency to meet SLA requirements.
- When upstream callers have their own deadlines that must be respected.

## When NOT to Use

- For long-running batch jobs where completion matters more than latency.
- When the operation already has built-in deadline support at a lower layer.

## Anti-Patterns

- Using excessively generous timeouts that provide no real protection.
- Not propagating deadlines through the call chain (each layer adds its own full timeout).
- Ignoring timeout errors instead of treating them as failures for circuit breaker tracking.

## Related Patterns

- [resilience/circuit-breaker](circuit-breaker.md) -- timeouts count as failures toward the trip threshold.
- [resilience/retry-backoff](retry-backoff.md) -- retry after a timeout with backoff.
- [resilience/bulkhead](bulkhead.md) -- timeouts release bulkhead slots promptly.

## References

- Go context package: https://pkg.go.dev/context
- Tokio timeout: https://docs.rs/tokio/latest/tokio/time/fn.timeout.html
- **Rust**: `tokio::time::timeout`, `tower::timeout::Timeout`
- **Go**: `context.WithTimeout`, `context.WithDeadline`
- **Python**: `asyncio.timeout` (3.11+), `aiohttp` client timeout
- **Java/Kotlin**: Resilience4j TimeLimiter
- **TypeScript**: `AbortController`, `p-timeout`
