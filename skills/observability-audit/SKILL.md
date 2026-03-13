---
name: observability-audit
description: Logging, monitoring, and health check consistency audit — log level usage, structured logging, correlation IDs, metric coverage, and health endpoint depth.
origin: ECC
---

# Observability Audit

Methodology for assessing observability quality — whether the codebase produces the signals needed to operate, debug, and monitor the system in production.

## When to Activate

- Codebase audit (via `/audit --domain=observability`)
- Investigating production debugging gaps
- Before launching a new service
- After incidents caused by insufficient logging/monitoring
- Evaluating operational readiness

## Methodology

### 1. Log Level Consistency

Scan all log calls and classify by level to detect misuse.

**Detection**:
- Grep for log calls: `console.log/warn/error`, `log.Info/Warn/Error`, `logger.info/warning/error`, `logging.info/warning/error`
- Classify each call by level used

**Anti-patterns**:
- `ERROR` for non-actionable information (e.g., logging request parameters at ERROR level) — HIGH
- `INFO` for actual errors or failures — HIGH
- `DEBUG` statements left in production code paths — MEDIUM
- No `WARN` usage (missing early warning signals) — LOW
- Inconsistent level choices for similar operations across modules — MEDIUM

### 2. Structured vs Unstructured Logging

**Detection**:
- Structured: Log calls with key-value pairs, JSON objects, or structured logging libraries (winston, zap, structlog, slog)
- Unstructured: String concatenation, template literals, f-strings without structured fields

**Report**:
- Ratio of structured to unstructured log calls
- `< 50% structured` — HIGH: difficult to query and aggregate logs
- `50-80% structured` — MEDIUM: inconsistent, migrate remaining
- `> 80% structured` — OK

### 3. Correlation ID Propagation

**Detection**:
- Search for request/trace/correlation ID patterns: `requestId`, `traceId`, `correlationId`, `x-request-id`
- Check if ID is:
  - Generated at entry points (HTTP handlers, message consumers) — CRITICAL if missing
  - Passed through the call chain (function parameters, context objects) — HIGH if missing
  - Included in log calls — MEDIUM if missing
  - Propagated to outbound calls (HTTP clients, message publishers) — HIGH if missing

### 4. Metric Coverage

**Detection**:
- Identify service boundary functions: HTTP handlers, RPC endpoints, queue consumers, cron jobs
- Check for instrumentation: timing/duration metrics, counter increments, histogram observations
- Libraries to look for: prometheus, datadog, statsd, opentelemetry, micrometer

**Report**:
- `uninstrumented_boundaries / total_boundaries` ratio
- `> 50% uninstrumented` — HIGH
- `20-50% uninstrumented` — MEDIUM
- `< 20% uninstrumented` — OK

### 5. Health Check Depth

**Detection**:
- Find health/readiness/liveness endpoints: `/health`, `/healthz`, `/ready`, `/readiness`, `/liveness`, `/ping`
- Classify each endpoint:
  - **Static**: Returns 200 unconditionally — LOW value
  - **Self-check**: Verifies local state (memory, disk) — MEDIUM value
  - **Dependency-checking**: Verifies downstream dependencies (DB, cache, external APIs) — HIGH value (preferred)

**Anti-patterns**:
- No health endpoint at all — CRITICAL
- Health endpoint that never fails (static success) — HIGH
- Health endpoint that checks dependencies but has no timeout — MEDIUM
- Readiness and liveness returning the same result — LOW

## Finding Format

```
### [OBS-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: file:line-range
- **Principle**: The violated principle
- **Evidence**: Concrete data (log call counts, ratios, missing IDs)
- **Risk**: What breaks if unaddressed
- **Remediation**: Directional fix (what, not how)
```

## Related

- Agent: `agents/observability-auditor.md`
- Command: `commands/audit.md`
- Complementary: `skills/backend-patterns/SKILL.md`, `skills/deployment-patterns/SKILL.md`
