---
name: correlation-id
category: observability
tags: [observability, correlation, id]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Assign a unique identifier to each request and propagate it through all downstream service calls.

## Problem

When a request fails in a multi-service system, correlating logs from different services is impossible without a shared identifier.

## Solution

Generate a UUID at the ingress point (API gateway or first service). Pass it via HTTP header (X-Correlation-Id) and include in every log entry.

## Language Implementations

### All Languages
OpenTelemetry provides unified APIs across Rust, Go, Python, TypeScript, and Java for correlation id.

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
