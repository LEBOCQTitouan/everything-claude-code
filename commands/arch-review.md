---
description: Architecture quality audit — layering, coupling, dependency direction, circular deps, DDD/hexagonal compliance. Invokes the arch-reviewer agent.
---

# Architecture Review

Comprehensive architecture audit of a codebase. Unlike `/code-review` which reviews code changes for bugs and style, `/arch-review` reviews the entire project structure for architectural violations.

## What This Command Does

1. **Detect project profile** — language, framework, file count, organization pattern
2. **Map directory structure** — identify layers, boundaries, and organization style
3. **Analyze dependency graph** — import direction, circular deps, coupling metrics, file sizes
4. **Run sub-agent reviews** — delegate to architect, architect-module, and uncle-bob agents
5. **Consolidate findings** — deduplicate, tag sources, classify severity
6. **Generate report** — structured output with A-F score and prioritized recommendations

## When to Use

- Onboarding to an unfamiliar codebase
- Before major releases
- Periodic health checks (monthly/quarterly)
- After significant refactoring
- When maintainability or performance degrades
- Planning a migration or rewrite

## Arguments

- `[path]` — target directory to audit (default: current project root)
- `--quick` — structural analysis only, skip sub-agent delegation (faster, less thorough)
- `--focus=<dimension>` — narrow scope to a specific dimension (e.g., `--focus=coupling`, `--focus=layers`, `--focus=domain`)

## Review Categories

**CRITICAL:**
- Domain layer imports infrastructure/framework types
- Adapter types (ORM, HTTP, SDK) leak into domain signatures
- Import cycles between architectural layers
- No port interfaces — domain depends on concrete implementations

**HIGH:**
- No clear layer separation in directory structure
- Anemic domain model (entities with only getters/setters)
- God module (>20 files depend on it)
- Cross-context direct imports
- Application layer imports adapter types

**MEDIUM:**
- Files exceeding 800 lines
- High fan-out (module imports >15 others)
- Import cycles within a layer
- Mixed concerns in a single directory
- Primitive obsession in domain signatures

**LOW:**
- Organized by type instead of by feature
- Missing domain events for state changes
- Minor cohesion issues within modules

## Approval Criteria

| Score | Verdict | Criteria |
|-------|---------|----------|
| A | HEALTHY | 0 CRITICAL, 0 HIGH, <=3 MEDIUM |
| B | GOOD | 0 CRITICAL, <=2 HIGH, any MEDIUM |
| C | NEEDS ATTENTION | 0 CRITICAL, >2 HIGH |
| D | NEEDS REFACTORING | 1+ CRITICAL or >5 HIGH |
| F | CRITICAL | 3+ CRITICAL issues |

## Example Usage

```
User: /arch-review

Agent (arch-reviewer):
# Architecture Review Report

## Project Profile
- Language: TypeScript
- Framework: Next.js 14
- Root: /Users/dev/my-saas
- Total source files: 247
- Organization: mixed (by-feature for pages, by-type for services)

## Architecture Score: C — NEEDS ATTENTION

## Findings

### CRITICAL
- [Structural] src/domain/order.ts:L15 — Imports PrismaClient from @prisma/client

### HIGH
- [Strategic] src/domain/ — Anemic domain: Order entity has no business methods
- [Structural] src/lib/utils.ts — God module: imported by 34 files (fan-in: 34)
- [Clean Code] src/services/payment.ts — DIP violation: depends on StripeSDK directly

### MEDIUM
- [Module] src/services/order-service.ts — 912 lines, exceeds 800-line limit
- [Structural] src/hooks/ ↔ src/services/ — Circular import detected

### LOW
- [Module] src/components/ — Organized by type (ui/, layout/, forms/)

## Dimension Summary
| Dimension | Status | Issues |
|-----------|--------|--------|
| Dependency Direction | FAIL | 1 |
| Layer Separation | PASS | 0 |
| Circular Dependencies | FAIL | 1 |
| Coupling | WARN | 1 |
| Cohesion | OK | 0 |
| Domain Model Quality | WARN | 1 |
| Bounded Contexts | N/A | 0 |
| Ports & Adapters | FAIL | 1 |
| File Organization | WARN | 1 |
| SOLID Compliance | WARN | 1 |

## Top Recommendations
1. Remove PrismaClient import from domain/order.ts — define a port interface instead
2. Add business methods to Order entity (calculateTotal, applyDiscount, validate)
3. Break src/lib/utils.ts into focused utility modules (<5 dependents each)

## Totals
| Severity | Count |
|----------|-------|
| CRITICAL | 1 |
| HIGH | 3 |
| MEDIUM | 2 |
| LOW | 1 |
```

## Difference from /code-review

| | `/code-review` | `/arch-review` |
|---|---|---|
| Scope | Uncommitted changes (git diff) | Entire codebase |
| Focus | Security, bugs, style, quality | Structure, layering, coupling, boundaries |
| Frequency | Every commit | Monthly or milestone-based |
| Output | Per-file issues | System-wide report with score |

## Integration with Other Commands

After reviewing:
- Use `/plan` to create a remediation plan for findings
- Use `/refactor-clean` to remove dead code or god modules found
- Use `/code-review` for code-level quality issues
- Use `/tdd` to add tests for untested areas identified

## Related

- Agent: `agents/arch-reviewer.md`
- Skill: `skills/architecture-review/SKILL.md`
- Consulted agents: `agents/architect.md`, `agents/architect-module.md`, `agents/uncle-bob.md`
