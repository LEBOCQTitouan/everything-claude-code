---
name: failure-modes
description: Extract failure scenarios, detection signals, blast radius, and recovery procedures from source code for runbook and documentation generation.
origin: ECC
---

# Failure Mode Extraction

Atomic extraction skill for identifying how a system can fail — what breaks, how you detect it, how bad it gets, and how to fix it. Produces structured failure data for runbook generation and operational documentation.

## When to Activate

- Before generating operational runbooks
- When documenting error handling and recovery procedures
- During reliability audits
- When building incident response documentation

## Methodology

### 1. Error Taxonomy

Scan codebase for error definitions and handling:

**Custom error classes/types:**
```
grep -r "class.*Error\|class.*Exception\|type.*Error\|errors\.New\|fmt\.Errorf"
```

For each custom error:
- Name and hierarchy (extends which base?)
- Where it's thrown/returned
- Where it's caught/handled
- HTTP status code mapping (if applicable)
- User-facing message vs internal message

**Error code enums:**
```
grep -r "ErrorCode\|error_code\|ERR_\|E_"
```

### 2. Failure Scenario Mapping

For each module, identify failure scenarios:

| Category | Detection Pattern |
|----------|-------------------|
| **Input validation** | Early returns, throw on invalid input |
| **External service** | try/catch around HTTP calls, timeouts, retries |
| **Data integrity** | Null checks, type assertions, schema validation |
| **Resource exhaustion** | Memory limits, connection pools, rate limits |
| **Concurrency** | Deadlock potential, race conditions, lock timeouts |
| **Configuration** | Missing env vars, invalid config, startup checks |
| **File system** | Permission errors, disk full, missing files |
| **Authentication** | Token expiry, invalid credentials, permission denied |

For each scenario, record:

```
Scenario: External API timeout
  Module: src/api/client.ts
  Trigger: API response time > 30s
  Detection: TimeoutError caught in fetchWithRetry()
  Impact: Degraded — feature unavailable, other features unaffected
  Recovery: Automatic retry (3x with exponential backoff)
  Escalation: After 3 retries, returns cached data if available, else throws
```

### 3. Blast Radius Assessment

For each failure scenario, determine impact scope:

| Level | Definition |
|-------|-----------|
| **Isolated** | Single request/operation fails, no side effects |
| **Degraded** | Feature partially unavailable, workarounds exist |
| **Major** | Core functionality impacted, multiple users affected |
| **Critical** | System-wide outage, data integrity at risk |

Assess by tracing:
1. What calls the failing function? (upstream impact)
2. Does the failure propagate or get caught? (containment)
3. Is there a fallback/circuit breaker? (resilience)
4. Does it affect shared state? (data integrity)

### 4. Recovery Procedure Extraction

For each failure scenario, document recovery:

**Automatic recovery** (code-level):
- Retry logic (how many, what backoff?)
- Circuit breakers (when do they trip? when do they reset?)
- Fallback values (what degraded behaviour?)
- Self-healing (automatic restart, reconnection)

**Manual recovery** (operator-level):
- What to check first (logs, metrics, dashboards)
- What to restart/redeploy
- Data repair procedures (if data was corrupted)
- Rollback instructions

### 5. Dependency Failure Mapping

For each external dependency:
- What happens if it's down?
- What happens if it's slow (2x, 10x normal latency)?
- What happens if it returns unexpected data?
- Is there a health check endpoint?
- What's the SLA assumption?

## Output Format

```
# Failure Modes: project-name

## Error Taxonomy
| Error | Module | HTTP Status | Recoverable | Documented |
|-------|--------|-------------|-------------|------------|
| ValidationError | src/lib/validate.ts | 400 | Yes | Yes |
| DatabaseConnectionError | src/db/pool.ts | 503 | Auto-retry | No |
| AuthTokenExpired | src/auth/jwt.ts | 401 | Re-auth | Yes |

## Failure Scenarios (12 identified)
[structured scenarios as above]

## Dependency Failures
| Dependency | Down Impact | Slow Impact | Fallback |
|-----------|-------------|-------------|----------|
| PostgreSQL | Critical — no writes | Degraded — queue backlog | Read from replica |
| Redis | Degraded — no caching | Minor — slower responses | Skip cache, hit DB |
| Stripe API | Major — no payments | Degraded — slow checkout | Queue for retry |

## Unhandled Paths (3 found)
| Location | Risk | Recommendation |
|----------|------|----------------|
| src/api/webhook.ts:45 | Unhandled promise rejection | Add try/catch |
| src/lib/parse.ts:89 | No validation on external input | Add schema validation |
```

## Related

- Common failure patterns: `skills/failure-modes/references/common-patterns.md`
- Runbook generation: `skills/runbook-gen/SKILL.md`
- Behaviour extraction: `skills/behaviour-extraction/SKILL.md`
- Error handling auditor: `agents/error-handling-auditor.md`
