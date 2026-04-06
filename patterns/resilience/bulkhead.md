---
name: bulkhead
category: resilience
tags: [resilience, isolation, resource-management]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Isolate resources into independent pools so that a failure or overload in one component cannot exhaust resources shared by others, limiting the blast radius of failures.

## Problem

When all outbound calls share a single thread pool or connection pool, one slow dependency can consume all available resources, starving healthy dependencies of capacity. The entire system degrades even though only one downstream is unhealthy.

## Solution

Partition resources (threads, connections, semaphores) into isolated compartments -- one per dependency or criticality tier. Each compartment has a fixed capacity; once full, new requests are rejected immediately rather than queuing behind the slow dependency.

## Language Implementations

### Rust

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

struct Bulkhead {
    semaphore: Arc<Semaphore>,
}

impl Bulkhead {
    fn new(max_concurrent: usize) -> Self {
        Self { semaphore: Arc::new(Semaphore::new(max_concurrent)) }
    }

    async fn execute<T, F, Fut>(&self, op: F) -> Result<T, BulkheadError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let permit = self.semaphore.try_acquire()
            .map_err(|_| BulkheadError::Full)?;
        let result = op().await;
        drop(permit);
        Ok(result)
    }
}

#[derive(Debug)]
enum BulkheadError { Full }
```

### Go

```go
type Bulkhead struct {
    sem chan struct{}
}

func NewBulkhead(maxConcurrent int) *Bulkhead {
    return &Bulkhead{sem: make(chan struct{}, maxConcurrent)}
}

func (b *Bulkhead) Execute(fn func() error) error {
    select {
    case b.sem <- struct{}{}:
        defer func() { <-b.sem }()
        return fn()
    default:
        return ErrBulkheadFull
    }
}
```

### Python

```python
import asyncio

class Bulkhead:
    def __init__(self, max_concurrent: int) -> None:
        self._semaphore = asyncio.Semaphore(max_concurrent)

    async def execute(self, coro):
        if self._semaphore.locked():
            raise BulkheadFullError("bulkhead at capacity")
        async with self._semaphore:
            return await coro
```

### Typescript

```typescript
class Bulkhead {
  private active = 0;
  constructor(private readonly maxConcurrent: number) {}

  async execute<T>(fn: () => Promise<T>): Promise<T> {
    if (this.active >= this.maxConcurrent) {
      throw new Error("bulkhead full");
    }
    this.active++;
    try {
      return await fn();
    } finally {
      this.active--;
    }
  }
}
```

## When to Use

- When multiple downstream dependencies share a common resource pool.
- When one slow dependency must not starve others of capacity.
- When you need predictable degradation under partial failure.

## When NOT to Use

- When you have a single downstream dependency with no isolation benefit.
- When the overhead of managing separate pools outweighs the resilience gain.

## Anti-Patterns

- Setting compartment sizes too small, causing legitimate traffic to be rejected.
- Using a single shared bulkhead for all services, defeating the purpose.
- Not monitoring rejection rates to detect misconfigured limits.

## Related Patterns

- [resilience/circuit-breaker](circuit-breaker.md) -- trip when a compartment sees too many failures.
- [resilience/rate-limiter](rate-limiter.md) -- limit request rate rather than concurrency.
- [resilience/timeout](timeout.md) -- prevent slow calls from holding a compartment slot indefinitely.

## References

- Michael Nygard, "Release It!", Chapter 5 -- Bulkheads.
- **Rust**: `tower::limit::ConcurrencyLimit`
- **Go**: semaphore pattern with buffered channels
- **Python**: `asyncio.Semaphore`, `aiohttp` connector limits
- **Java/Kotlin**: Resilience4j Bulkhead
- **TypeScript**: `cockatiel` bulkhead policy
