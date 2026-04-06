---
name: circuit-breaker
category: resilience
tags: [resilience, fault-tolerance, stability]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Prevent cascading failures by wrapping calls to external services in a stateful proxy that trips open after repeated failures, short-circuiting requests until the service recovers.

## Problem

When a downstream dependency becomes slow or unavailable, callers continue sending requests that queue up, exhaust resources (threads, connections, memory), and propagate failure upstream. Without automatic detection and fast-fail behaviour, a single unhealthy dependency can bring down the entire system.

## Solution

Implement a state machine with three states: Closed (requests pass through), Open (requests fail immediately), and Half-Open (a limited number of probe requests test recovery). Track failure counts and transition between states based on configurable thresholds and timeouts.

## Language Implementations

### Rust

```rust
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
enum State { Closed, Open, HalfOpen }

struct CircuitBreaker {
    failure_threshold: u32,
    reset_timeout: Duration,
    failures: AtomicU32,
    last_failure: AtomicU64,
}

impl CircuitBreaker {
    fn new(threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_threshold: threshold,
            reset_timeout: timeout,
            failures: AtomicU32::new(0),
            last_failure: AtomicU64::new(0),
        }
    }

    fn state(&self) -> State {
        let failures = self.failures.load(Ordering::Relaxed);
        if failures < self.failure_threshold { return State::Closed; }
        let elapsed = Instant::now().elapsed(); // simplified
        if elapsed > self.reset_timeout { State::HalfOpen } else { State::Open }
    }

    fn record_success(&self) { self.failures.store(0, Ordering::Relaxed); }
    fn record_failure(&self) { self.failures.fetch_add(1, Ordering::Relaxed); }
}
```

### Go

```go
type CircuitBreaker struct {
    threshold  int
    timeout    time.Duration
    failures   int
    lastFail   time.Time
    mu         sync.Mutex
}

func (cb *CircuitBreaker) Call(fn func() error) error {
    cb.mu.Lock()
    state := cb.state()
    cb.mu.Unlock()

    if state == Open {
        return ErrCircuitOpen
    }

    if err := fn(); err != nil {
        cb.mu.Lock()
        cb.failures++
        cb.lastFail = time.Now()
        cb.mu.Unlock()
        return err
    }

    cb.mu.Lock()
    cb.failures = 0
    cb.mu.Unlock()
    return nil
}
```

### Python

```python
import time
from enum import Enum, auto

class State(Enum):
    CLOSED = auto()
    OPEN = auto()
    HALF_OPEN = auto()

class CircuitBreaker:
    def __init__(self, threshold: int = 5, timeout: float = 30.0) -> None:
        self._threshold = threshold
        self._timeout = timeout
        self._failures = 0
        self._last_failure: float = 0.0

    @property
    def state(self) -> State:
        if self._failures < self._threshold:
            return State.CLOSED
        if time.monotonic() - self._last_failure > self._timeout:
            return State.HALF_OPEN
        return State.OPEN

    def call(self, fn):
        if self.state == State.OPEN:
            raise CircuitOpenError()
        try:
            result = fn()
            self._failures = 0
            return result
        except Exception as e:
            self._failures += 1
            self._last_failure = time.monotonic()
            raise
```

### Typescript

```typescript
enum State { Closed, Open, HalfOpen }

class CircuitBreaker {
  private failures = 0;
  private lastFailure = 0;

  constructor(
    private readonly threshold: number = 5,
    private readonly timeoutMs: number = 30_000,
  ) {}

  get state(): State {
    if (this.failures < this.threshold) return State.Closed;
    if (Date.now() - this.lastFailure > this.timeoutMs) return State.HalfOpen;
    return State.Open;
  }

  async call<T>(fn: () => Promise<T>): Promise<T> {
    if (this.state === State.Open) throw new Error("circuit open");
    try {
      const result = await fn();
      this.failures = 0;
      return result;
    } catch (e) {
      this.failures++;
      this.lastFailure = Date.now();
      throw e;
    }
  }
}
```

## When to Use

- When calling external services (HTTP APIs, databases, message brokers) that may become unavailable.
- When you need to fail fast rather than wait for timeouts on every request.
- When cascading failure across service boundaries is a real risk.

## When NOT to Use

- For in-process function calls that cannot fail due to external factors.
- When the downstream service has its own robust retry and backpressure mechanisms.
- For fire-and-forget operations where failure is acceptable.

## Anti-Patterns

- Setting the failure threshold too low, causing the circuit to trip on transient errors.
- Never transitioning to Half-Open, permanently blocking a recovered service.
- Using a single global circuit breaker for all downstream services instead of one per dependency.

## Related Patterns

- [resilience/retry-backoff](retry-backoff.md) -- retry before the circuit trips; stop retrying once it opens.
- [resilience/fallback](fallback.md) -- provide a degraded response when the circuit is open.
- [resilience/bulkhead](bulkhead.md) -- isolate resources per dependency to limit blast radius.
- [resilience/timeout](timeout.md) -- bound request duration; timeouts count as failures for the breaker.

## References

- Michael Nygard, "Release It!", Chapter 5 -- Stability Patterns.
- Martin Fowler, Circuit Breaker: https://martinfowler.com/bliki/CircuitBreaker.html
- **Rust**: `tower` (tower::retry, tower::limit), `failsafe`
- **Go**: `gobreaker` (sony/gobreaker), `go-resilience`
- **Python**: `pybreaker`, `circuitbreaker`
- **Java/Kotlin**: Resilience4j CircuitBreaker
- **TypeScript**: `opossum`, `cockatiel`
