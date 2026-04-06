---
name: metrics-collection
category: observability
tags: [observability, metrics, collection]
languages: [all]
difficulty: intermediate
---

## Intent

Collect quantitative measurements (counters, gauges, histograms) about system behavior for dashboards and alerting.

## Problem

Logs alone cannot answer questions like 'what is the p99 latency?' or 'how many requests per second?'

## Solution

Instrument code with a metrics library exposing Prometheus-compatible endpoints. Use counters for totals, gauges for current values, histograms for distributions.

## Language Implementations

### All
OpenTelemetry provides unified APIs across Rust, Go, Python, TypeScript, and Java for metrics collection.

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
