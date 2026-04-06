---
name: log-aggregation
category: observability
tags: [observability, log, aggregation]
languages: [all]
difficulty: intermediate
---

## Intent

Centralize logs from multiple services into a single searchable store for debugging and analysis.

## Problem

Each service writes logs locally. Debugging a distributed request requires SSHing into multiple servers and grepping files.

## Solution

Ship logs to a centralized store (ELK, Loki, CloudWatch) via agents (Fluentd, Vector) or direct API. Structure logs for efficient indexing.

## Language Implementations

### All Languages
OpenTelemetry provides unified APIs across Rust, Go, Python, TypeScript, and Java for log aggregation.

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
