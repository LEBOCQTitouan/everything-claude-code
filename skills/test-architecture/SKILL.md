---
name: test-architecture
description: Test quality assessment beyond coverage — test classification, structural coupling, fixture ratios, and missing test strategies.
origin: ECC
---

# Test Architecture

Test quality methodology that goes beyond coverage percentages. Evaluates how tests are structured, what they actually verify, and where the gaps are.

## When to Activate

- Codebase audit (via `/audit --domain=testing`)
- Test suite feels slow or brittle
- Coverage is high but bugs still escape
- Before adding a new test layer (e2e, contract, property-based)
- Evaluating test infrastructure health

## Methodology

### 1. Test Classification by Dependency Footprint

Classify tests by what they actually touch, not by directory name.

**Scan test files for**:
- **Pure unit**: No imports of fs, http, net, database drivers, external SDKs. No `setTimeout`/`setInterval`. Mock-free or only mocking internal interfaces.
- **Impure unit**: Uses filesystem, clock, or other OS primitives. May use test doubles for these.
- **Integration**: Imports database drivers, HTTP clients, message queue libraries, or makes real network calls. Uses test containers or test databases.
- **E2E**: Spawns the full application, uses browser automation (Playwright, Cypress), or hits live endpoints.

**Report**:
- Count per category
- Ratio of pure unit to total (healthy: > 60%)
- Flag test files that are hard to classify (mixed concerns)

### 2. Structural Coupling Score

Does the test tree mirror the source tree 1:1?

**Detection**:
- Map test files to source files by naming convention (`foo.test.ts` → `foo.ts`)
- Calculate mirror ratio: `matched_test_files / total_test_files`

**Thresholds**:
- `> 0.9` — HIGH coupling: tests are tightly bound to implementation structure. Refactoring source requires refactoring tests.
- `0.5-0.9` — OK: moderate coupling
- `< 0.5` — Tests organized by behavior/feature rather than structure (preferred for large codebases)

### 3. Fixture-to-Assertion Ratio

Tests with excessive setup relative to assertions are fragile and hard to maintain.

**Measurement per test file**:
- Lines of setup: `beforeEach`, `beforeAll`, factory calls, mock definitions, data construction
- Lines of assertion: `expect`, `assert`, `should`, `toBe`, etc.
- Ratio: `setup_lines / assertion_lines`

**Thresholds**:
- `> 10×` — CRITICAL: tests are mostly boilerplate
- `> 5×` — HIGH: heavy setup overhead
- `> 3×` — MEDIUM: consider shared fixtures or test data builders
- `≤ 3×` — OK

### 4. Missing Test Strategies

Identify valuable testing approaches not currently used.

**Check for presence of**:
- **Property-based tests**: Imports from fast-check, hypothesis, quickcheck, gopter
- **Contract tests**: Pact, Spring Cloud Contract, or consumer-driven test patterns
- **Mutation testing**: Stryker, mutmut, pitest configuration
- **Snapshot tests**: `.snap` files or snapshot assertions
- **Fuzzing**: Go fuzz, AFL, libFuzzer usage
- **Load/stress tests**: k6, artillery, locust, gatling

**Report**: List strategies present and absent. Recommend based on codebase type:
- API services → contract tests
- Data processing → property-based tests
- Security-sensitive → fuzzing
- User-facing → e2e + snapshot

### 5. Test-to-Source Coverage Mapping

Which source modules have zero test files?

**Detection**:
- List all source modules/directories
- For each, check if any test file references it (import or naming convention)
- Report untested modules, sorted by size (larger = higher priority)

**Cross-reference with hotspot data** (if available from evolutionary analysis):
- Hotspot files with zero tests = CRITICAL
- Hotspot files with only impure/integration tests = HIGH
- Non-hotspot files with zero tests = MEDIUM

## Finding Format

```
### [TEST-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: test-file:line or source-module
- **Principle**: The violated principle
- **Evidence**: Concrete data (ratios, counts, categories)
- **Risk**: What breaks if unaddressed
- **Remediation**: Directional fix (what, not how)
```

## Related

- Agent: `agents/test-auditor.md`
- Command: `commands/audit.md`
- Complementary: `skills/tdd-workflow/SKILL.md` (how to write tests), `skills/e2e-testing/SKILL.md`
