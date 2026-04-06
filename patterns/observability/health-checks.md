---
name: health-checks
category: observability
tags: [observability, health, checks]
languages: [all]
difficulty: intermediate
---

## Intent

Expose HTTP endpoints that report service readiness and liveness for orchestrators and load balancers.

## Problem

Container orchestrators (Kubernetes) and load balancers need to know if a service can accept traffic (readiness) and if it is alive (liveness).

## Solution

Implement /healthz (liveness) and /readyz (readiness) endpoints. Liveness: process is running. Readiness: dependencies (DB, cache) are connected.

## Language Implementations

### All
OpenTelemetry provides unified APIs across Rust, Go, Python, TypeScript, and Java for health checks.

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
