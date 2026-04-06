---
name: structured-logging
category: observability
tags: [observability, logging, structured, json]
languages: [rust, go, python, typescript, java]
difficulty: intermediate
---

## Intent

Emit log entries as structured key-value pairs instead of free-form text strings.

## Problem

Free-form log messages are difficult to parse, search, and aggregate. Extracting fields requires fragile regex patterns that break as message formats change.

## Solution

Use a structured logging library that emits JSON or key-value formatted entries with consistent field names (level, timestamp, message, correlation_id, service).

## Language Implementations

### Rust
```rust
use tracing::{info, instrument};
#[instrument(skip(db))]
fn process(id: u64, db: &Db) {
    info!(user_id = id, "processing request");
}
```

### Go
```go
slog.Info("processing request", "user_id", id, "service", "api")
```

### Python
```python
import structlog
log = structlog.get_logger()
log.info("processing_request", user_id=id)
```

### TypeScript
```typescript
import pino from "pino";
const log = pino();
log.info({ userId: id }, "processing request");
```

### Java
```java
// Using Logback with JSON encoder
log.info("processing request", kv("userId", id));
```

## When to Use

- Every service producing logs for aggregation.
- Microservices needing cross-service log correlation.

## When NOT to Use

- Quick debugging scripts where printf suffices.
- Embedded systems with severe memory constraints.

## Anti-Patterns

- Logging sensitive data (PII, credentials) in structured fields.
- Inconsistent field naming across services.

## Related Patterns

- observability/correlation-id
- observability/distributed-tracing

## References

- OpenTelemetry Logging specification.
- tracing crate (Rust). slog/zerolog (Go). structlog (Python). pino (TypeScript).
