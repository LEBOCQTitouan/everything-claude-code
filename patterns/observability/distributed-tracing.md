---
name: distributed-tracing
category: observability
tags: [observability, distributed, tracing]
languages: [all]
difficulty: intermediate
---

## Intent

Propagate trace context across service boundaries for end-to-end request visibility.

## Problem

Debugging failures in distributed systems requires correlating events across multiple services, but logs from different services have no shared identifier.

## Solution

Instrument services with OpenTelemetry to create spans that propagate trace IDs via HTTP headers (traceparent) or gRPC metadata.

## Language Implementations

### All
OpenTelemetry provides unified APIs across Rust, Go, Python, TypeScript, and Java for distributed tracing.

## When to Use

- Production services requiring operational visibility.
- Distributed systems with multiple service boundaries.

## When NOT to Use

- Local development scripts with no operational requirements.
- Single-process CLI tools.

## Anti-Patterns

- Sampling too aggressively and missing critical events.
- Not including service name and version in telemetry.

## Related Patterns

- observability/structured-logging
- observability/correlation-id

## References

- OpenTelemetry documentation.
- Prometheus client libraries.
