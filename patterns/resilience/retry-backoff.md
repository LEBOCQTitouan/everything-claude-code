---
name: retry-backoff
category: resilience
tags: [resilience, retry, exponential-backoff, jitter]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Automatically retry transient failures with increasing delays and randomised jitter, giving the downstream service time to recover without creating a thundering herd.

## Problem

Transient failures (network blips, brief overloads, temporary unavailability) resolve on their own if the caller waits and retries. Immediate retries amplify load on a struggling service. Fixed-interval retries synchronise callers into periodic spikes.

## Solution

Retry failed operations with exponential backoff (delay doubles each attempt) plus random jitter. Cap the maximum delay, limit the total number of attempts, and only retry on retryable error classes.

## Language Implementations

### Rust

```rust
use std::time::Duration;
use rand::Rng;

fn retry_with_backoff<T, E>(
    max_attempts: u32,
    base_ms: u64,
    mut op: impl FnMut() -> Result<T, E>,
) -> Result<T, E> {
    let mut rng = rand::thread_rng();
    for attempt in 0..max_attempts {
        match op() {
            Ok(v) => return Ok(v),
            Err(e) if attempt + 1 == max_attempts => return Err(e),
            Err(_) => {
                let delay = base_ms * 2u64.pow(attempt);
                let jitter = rng.gen_range(0..=delay / 2);
                std::thread::sleep(Duration::from_millis(delay + jitter));
            }
        }
    }
    unreachable!()
}
```

### Go

```go
func RetryWithBackoff(maxAttempts int, baseMs int64, op func() error) error {
    for attempt := 0; attempt < maxAttempts; attempt++ {
        if err := op(); err == nil {
            return nil
        } else if attempt+1 == maxAttempts {
            return err
        }
        delay := baseMs * (1 << attempt)
        jitter := rand.Int63n(delay / 2)
        time.Sleep(time.Duration(delay+jitter) * time.Millisecond)
    }
    return nil
}
```

### Python

```python
import random
import time

def retry_with_backoff(max_attempts: int, base_s: float, op):
    for attempt in range(max_attempts):
        try:
            return op()
        except Exception as e:
            if attempt + 1 == max_attempts:
                raise
            delay = base_s * (2 ** attempt)
            jitter = random.uniform(0, delay / 2)
            time.sleep(delay + jitter)
```

### Typescript

```typescript
async function retryWithBackoff<T>(
  maxAttempts: number,
  baseMs: number,
  op: () => Promise<T>,
): Promise<T> {
  for (let attempt = 0; attempt < maxAttempts; attempt++) {
    try {
      return await op();
    } catch (e) {
      if (attempt + 1 === maxAttempts) throw e;
      const delay = baseMs * 2 ** attempt;
      const jitter = Math.random() * delay / 2;
      await new Promise((r) => setTimeout(r, delay + jitter));
    }
  }
  throw new Error("unreachable");
}
```

## When to Use

- When calling external services prone to transient failures.
- When the operation is idempotent (safe to repeat).
- When you want automatic recovery without manual intervention.

## When NOT to Use

- When the operation is not idempotent (e.g., non-idempotent POST without deduplication).
- When failures are permanent (invalid input, authentication errors).
- When latency budgets are tight and retries would exceed acceptable response times.

## Anti-Patterns

- Retrying without backoff, flooding the failing service.
- Retrying non-retryable errors (4xx client errors, validation failures).
- Unbounded retries with no maximum attempt limit.
- Missing jitter, causing synchronised retry storms.

## Related Patterns

- [resilience/circuit-breaker](circuit-breaker.md) -- stop retrying once the circuit opens.
- [resilience/timeout](timeout.md) -- bound each attempt duration.
- [resilience/fallback](fallback.md) -- use a fallback after all retries are exhausted.

## References

- AWS Architecture Blog, "Exponential Backoff And Jitter": https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/
- **Rust**: `backoff`, `again`, `tokio-retry`
- **Go**: `cenkalti/backoff`, `avast/retry-go`
- **Python**: `tenacity`, `backoff`, `stamina`
- **Java/Kotlin**: Resilience4j Retry
- **TypeScript**: `p-retry`, `cockatiel`
