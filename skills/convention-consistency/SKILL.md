---
name: convention-consistency
description: Naming convention analysis, pattern consistency detection, configuration access mapping, and primitive obsession identification.
origin: ECC
---

# Convention Consistency

Methodology for detecting inconsistencies in naming, patterns, and conventions across the codebase. Surfaces divergence from dominant patterns that makes code harder to read and maintain.

## When to Activate

- Codebase audit (via `/audit --domain=conventions`)
- Onboarding to unfamiliar codebase (understanding conventions)
- After multiple contributors have been working independently
- Before establishing or documenting coding standards
- When code reviews frequently flag style/convention issues

## Methodology

### 1. Naming Convention Analysis

Detect the dominant naming pattern per context and flag deviations.

**Contexts to analyze**:
- Types/classes/interfaces: `PascalCase` vs `camelCase` vs `snake_case`
- Functions/methods: `camelCase` vs `snake_case` vs `PascalCase`
- Variables/parameters: `camelCase` vs `snake_case`
- Constants: `SCREAMING_SNAKE` vs `camelCase` vs `PascalCase`
- File names: `kebab-case` vs `camelCase` vs `PascalCase` vs `snake_case`

**Detection**:
- For each context, count occurrences of each casing style
- Identify the dominant pattern (> 70% usage)
- Flag deviations from the dominant pattern

**Thresholds**:
- `> 10% deviation` from dominant pattern — HIGH
- `5-10% deviation` — MEDIUM
- `< 5% deviation` — LOW (acceptable noise)
- No clear dominant pattern (< 60% majority) — HIGH: no convention established

### 2. Naming Entropy

Mixed abbreviation styles within the same module indicate inconsistency.

**Detection**:
- Within each module/directory, scan for:
  - Mixed abbreviations: `msg` vs `message`, `req` vs `request`, `res` vs `response`, `err` vs `error`, `ctx` vs `context`, `cfg` vs `config`
  - Inconsistent pluralization: `user` vs `users` for same concept
  - Synonym usage: `get` vs `fetch` vs `retrieve`, `create` vs `make` vs `build`, `delete` vs `remove` vs `destroy`

**Report**:
- Per-module entropy score based on number of inconsistent pairs
- Flag specific inconsistent pairs with file locations

### 3. Pattern Consistency

For each cross-cutting concern, identify all distinct implementation patterns and flag divergence.

**Cross-cutting concerns to check**:
- **Error handling**: How many distinct patterns exist? (try/catch, Result types, error callbacks, .catch chains)
- **Logging**: Direct console vs logger instance vs injected logger
- **Configuration access**: Direct env var reads vs config object vs DI
- **Data serialization**: Manual mapping vs library (class-transformer, serde, encoding/json)
- **Validation**: Manual checks vs schema libraries vs decorator-based
- **HTTP client usage**: fetch vs axios vs got vs http module

**For each concern**:
- Count files using each pattern
- Identify the majority pattern
- Flag minority patterns as deviations

**Thresholds**:
- 3+ distinct patterns for same concern — HIGH
- 2 patterns with < 80/20 split — MEDIUM
- 2 patterns with > 80/20 split — LOW (migration in progress)

### 4. Configuration Access Map

Every point that reads configuration — is there a single source of truth?

**Detection**:
- `process.env.*`, `os.environ`, `os.Getenv`, `System.getenv`
- Config file reads: `config.*`, `settings.*`, `application.yml`
- Hardcoded defaults: Magic numbers, inline URLs, hardcoded timeouts

**Report**:
- Total unique config access points
- Number of files that directly read env vars
- Hardcoded values that should be configurable
- Whether a centralized config module exists

**Anti-patterns**:
- Same env var read in multiple files — HIGH
- Hardcoded values for environment-specific settings (URLs, ports, timeouts) — HIGH
- No centralized config module — MEDIUM
- Config values without defaults or validation — MEDIUM

### 5. Primitive Obsession

Using raw primitives where domain types would add safety and clarity.

**Detection**:
- Functions with multiple parameters of the same type: `createUser(string, string, string, number)` — which string is which?
- Repeated validation logic for the same concept in multiple files (email regex, phone format, UUID check)
- String/number literals representing domain concepts (status codes, role names, permission levels)

**Anti-patterns**:
- `function(id: string, name: string, email: string)` — HIGH: parameters easily swapped
- Same validation regex for "email" in 3+ files — HIGH: should be a value object
- String literals for state/status: `if (status === 'active')` without type safety — MEDIUM
- Magic numbers without named constants — MEDIUM

## Finding Format

```
### [CONV-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: file:line-range or module
- **Principle**: The violated principle
- **Evidence**: Concrete data (pattern counts, deviation percentages, specific examples)
- **Risk**: What breaks if unaddressed
- **Remediation**: Directional fix (what, not how)
```

## Related

- Agent: `agents/convention-auditor.md`
- Command: `commands/audit.md`
- Complementary: `skills/coding-standards/SKILL.md`, `skills/architecture-review/SKILL.md`
