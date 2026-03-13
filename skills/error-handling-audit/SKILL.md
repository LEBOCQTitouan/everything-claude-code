---
name: error-handling-audit
description: Error handling architecture audit — swallowed errors, error taxonomy, boundary translation, and partial failure handling.
origin: ECC
---

# Error Handling Audit

Methodology for assessing error handling architecture quality. Evaluates whether errors are properly caught, classified, translated at boundaries, and handled during partial failures.

## When to Activate

- Codebase audit (via `/audit --domain=errors`)
- After production incidents caused by silent failures
- When error messages are unhelpful or inconsistent
- Before adding error monitoring/alerting
- Evaluating resilience of multi-service operations

## Methodology

### 1. Swallowed Errors

Errors that are caught but not properly handled — the most dangerous pattern.

**Detection patterns**:
- Empty catch blocks: `catch (e) {}`, `except: pass`, `catch {}`, `if err != nil { }`
- Log-and-forget: `catch (e) { console.log(e) }` without re-throw or return
- Callback ignoring: `.catch(() => {})`, `.catch(noop)`, `_ = someFunction()`
- Ignored promise: async function call without `await` or `.catch`
- Error parameter unused: `catch (e)` where `e` is never referenced

**Severity**:
- In request handlers or business logic — CRITICAL
- In background jobs or event handlers — HIGH
- In cleanup/teardown code — MEDIUM
- In optional/best-effort operations (telemetry, analytics) — LOW

### 2. Error Taxonomy

Is there a structured error hierarchy or are all errors generic?

**Detection**:
- Search for custom error classes: `extends Error`, `extends Exception`, error types with domain-specific names
- Count usage of generic vs custom errors: `new Error("...")` vs `new NotFoundError("...")`
- Check for error codes or error enum definitions

**Assessment**:
- No custom error classes — HIGH: impossible to handle errors programmatically
- Custom errors but inconsistent usage (some modules use them, others don't) — MEDIUM
- Structured error hierarchy with domain-specific types — OK
- Error codes or machine-readable error identifiers — OK (preferred for APIs)

### 3. Error Boundary Completeness

At module boundary crossings, do infrastructure errors surface verbatim in domain code?

**Detection**:
- Find module/layer boundaries (domain ↔ infrastructure, service ↔ repository)
- Check if infrastructure-specific errors (SQL errors, HTTP errors, filesystem errors) propagate unchanged into domain/application code
- Look for error translation: `catch (SqlError) { throw new DomainError(...) }`

**Anti-patterns**:
- Database error messages reaching API responses — CRITICAL
- Infrastructure exception types in domain function signatures — HIGH
- Stack traces from internal libraries exposed to callers — HIGH
- Error wrapping without adding context: `throw new Error(originalError.message)` — MEDIUM

### 4. Partial Failure Handling

Multi-resource operations (multiple I/O calls in one function) without compensation logic.

**Detection**:
- Identify functions with multiple I/O operations (2+ database calls, API calls, or file operations in one function)
- Check for compensation/rollback patterns:
  - Database transactions wrapping multiple operations
  - Try/catch with undo logic in the catch block
  - Saga pattern or compensation events
  - Idempotency keys for retry safety

**Anti-patterns**:
- Multiple database writes without a transaction — CRITICAL
- Multiple external API calls without compensating actions on failure — HIGH
- File operations (create + write + move) without cleanup on partial failure — MEDIUM
- Fire-and-forget side effects in business operations — HIGH

## Finding Format

```
### [ERR-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: file:line-range
- **Principle**: The violated principle
- **Evidence**: Concrete data (pattern found, code snippet reference)
- **Risk**: What breaks if unaddressed
- **Remediation**: Directional fix (what, not how)
```

## Related

- Agent: `agents/error-handling-auditor.md`
- Command: `commands/audit.md`
- Complementary: `skills/backend-patterns/SKILL.md`, `skills/security-review/SKILL.md`
