# Common Failure Mode Patterns

Reference material for the failure-modes extraction skill. Catalogues recurring failure patterns across application types.

## Web API Failures

### Request Handling

| Pattern | Cause | Detection | Impact |
|---------|-------|-----------|--------|
| Unvalidated input | Missing schema validation on body/params | 500 errors on malformed input | Isolated per request |
| Missing auth check | Endpoint lacks middleware | Unauthorized access | Major — security breach |
| N+1 queries | ORM loading related records in loop | Slow response, DB overload | Degraded — all users |
| Unbounded response | No pagination, returns all records | OOM kill, timeout | Major — process crash |
| Missing CORS | Browser blocks cross-origin request | 403 in browser console | Isolated — client-side |

### Database

| Pattern | Cause | Detection | Impact |
|---------|-------|-----------|--------|
| Connection pool exhaustion | Leaked connections, slow queries | "too many connections" error | Critical — all queries fail |
| Missing index | Query on unindexed column | Slow query log, high CPU | Degraded — slow reads |
| Deadlock | Concurrent updates same rows | Deadlock detected error | Isolated — retry succeeds |
| Migration failure | Schema change breaks running code | Startup crash, query errors | Critical — downtime |
| Data corruption | Partial write without transaction | Inconsistent data, constraint violations | Critical — data loss risk |

### External Service Integration

| Pattern | Cause | Detection | Impact |
|---------|-------|-----------|--------|
| Timeout without retry | No retry logic on transient failures | Sporadic errors | Isolated — single operation |
| Missing circuit breaker | Cascading failure from slow dependency | Thread pool exhaustion | Critical — cascading |
| Stale cache | Cache not invalidated on writes | Serving outdated data | Degraded — wrong data shown |
| Rate limit exceeded | Too many requests to external API | 429 responses | Degraded — feature unavailable |
| SSL certificate expiry | Cert not renewed | TLS handshake failure | Critical — all HTTPS fails |

## Background Job Failures

| Pattern | Cause | Detection | Impact |
|---------|-------|-----------|--------|
| Poison message | Malformed message crashes consumer | Consumer restart loop | Degraded — queue backup |
| Lost message | No acknowledgement, message dropped | Missing data, gap in processing | Major — data loss |
| Duplicate processing | At-least-once without idempotency | Duplicated side effects | Major — data integrity |
| Unbounded queue | Producer faster than consumer | Memory exhaustion | Critical — OOM kill |
| Stale lock | Worker crashes holding distributed lock | Other workers blocked | Degraded — processing halted |

## Authentication & Authorization

| Pattern | Cause | Detection | Impact |
|---------|-------|-----------|--------|
| Token not rotated | Long-lived tokens without refresh | Compromised credentials persist | Major — security |
| Insecure comparison | Timing attack on token comparison | No direct detection | Major — security breach |
| Missing RBAC check | Horizontal privilege escalation | Audit log, pentest findings | Critical — data leak |
| Session fixation | Session ID not regenerated after login | Security audit | Major — session hijack |

## Infrastructure

| Pattern | Cause | Detection | Impact |
|---------|-------|-----------|--------|
| Disk full | Logs not rotated, temp files not cleaned | Write errors, "no space left" | Critical — system halt |
| DNS resolution failure | DNS server down or misconfigured | ENOTFOUND errors | Critical — all network fails |
| Time skew | NTP not synced across hosts | Token validation failures, cert errors | Major — auth failures |
| Memory leak | Unbounded cache, event listener not removed | Gradual memory increase, eventual OOM | Major — process crash |

## Recovery Patterns

### Automatic Recovery

| Pattern | When to Use | Implementation |
|---------|-------------|---------------|
| **Retry with backoff** | Transient failures (network, rate limits) | Exponential backoff with jitter, max 3-5 retries |
| **Circuit breaker** | Downstream dependency failures | Open after N failures, half-open after timeout |
| **Fallback value** | Non-critical data unavailable | Return cached/default value, flag as degraded |
| **Dead letter queue** | Poison messages | Move failed messages to DLQ after N retries |
| **Graceful degradation** | Partial system failure | Disable non-essential features, serve core function |

### Manual Recovery

| Pattern | When to Use | Procedure |
|---------|-------------|-----------|
| **Rolling restart** | Memory leak, stale state | Restart instances one at a time behind load balancer |
| **Rollback deploy** | Bad release, data migration issue | Revert to previous version, verify data integrity |
| **Data repair** | Corruption from bug, partial write | Run repair script, verify with checksums |
| **DNS failover** | Region/provider outage | Switch DNS to backup region, verify propagation |
| **Cache flush** | Stale data, schema change | Clear cache, warm critical paths, monitor hit rate |
