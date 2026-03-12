---
description: Comprehensive verification gate — build, tests, lint, code review, architecture review, coverage analysis, and dead code scan.
---

# Verification Command

Run comprehensive verification on current codebase state. This is the single quality gate that combines build checks, code review, and architecture review.

## Instructions

Execute verification in this exact order. Each phase is sequential — stop on critical failures.

### Phase 1: Build Check

- Run the build command for this project
- If it fails, report errors and **STOP** (use `/build-fix` to resolve)

### Phase 2: Type Check

- Run TypeScript/type checker (or language equivalent)
- Report all errors with file:line

### Phase 3: Lint Check

- Run linter (with `--fix` if `--fix` argument was passed)
- Report warnings and errors

### Phase 4: Test Suite + Coverage Analysis

- Run all tests with coverage enabled
- Report pass/fail count and coverage percentage
- List files **below 80% coverage**, sorted worst-first
- For each under-covered file, identify:
  - Untested functions or methods
  - Missing branch coverage (if/else, switch, error paths)

Coverage framework detection:

| Indicator | Coverage Command |
|-----------|-----------------|
| `jest.config.*` or `package.json` jest | `npx jest --coverage --coverageReporters=json-summary` |
| `vitest.config.*` | `npx vitest run --coverage` |
| `pytest.ini` / `pyproject.toml` pytest | `pytest --cov=src --cov-report=json` |
| `Cargo.toml` | `cargo llvm-cov --json` |
| `pom.xml` with JaCoCo | `mvn test jacoco:report` |
| `go.mod` | `go test -coverprofile=coverage.out ./...` |

### Phase 5: Dead Code Scan (report only)

Run dead code analysis based on project type:

| Tool | What It Finds | Command |
|------|--------------|---------|
| knip | Unused exports, files, dependencies | `npx knip` |
| depcheck | Unused npm dependencies | `npx depcheck` |
| ts-prune | Unused TypeScript exports | `npx ts-prune` |
| vulture | Unused Python code | `vulture src/` |
| deadcode | Unused Go code | `deadcode ./...` |
| cargo-udeps | Unused Rust dependencies | `cargo +nightly udeps` |

If no tool is available, use Grep to find exports with zero imports.

Categorize findings into safety tiers:

| Tier | Examples | Note |
|------|----------|------|
| **SAFE** | Unused utilities, test helpers, internal functions | Can be deleted with confidence |
| **CAUTION** | Components, API routes, middleware | Verify no dynamic imports or external consumers |
| **DANGER** | Config files, entry points, type definitions | Investigate before touching |

**Report only** — do not auto-delete. Recommend `/plan refactor` for cleanup.

### Phase 6: Code Review

Review all uncommitted changes (`git diff HEAD`). For each changed file, check for:

**Security Issues (CRITICAL):**
- Hardcoded credentials, API keys, tokens
- SQL injection vulnerabilities
- XSS vulnerabilities
- Missing input validation
- Insecure dependencies
- Path traversal risks

**Code Quality (HIGH):**
- Functions > 50 lines
- Files > 800 lines
- Nesting depth > 4 levels
- Missing error handling
- console.log statements
- TODO/FIXME comments
- Missing JSDoc for public APIs

**Best Practices (MEDIUM):**
- Mutation patterns (use immutable instead)
- Missing tests for new code
- Accessibility issues (a11y)

**Language-specific review auto-detection:**
- **Go project detected** → invoke the **go-reviewer** agent for idiomatic Go, concurrency safety, race detection, govulncheck
- **Python project detected** → invoke the **python-reviewer** agent for PEP 8, type hints, framework-specific checks (Django/FastAPI/Flask)

Block if CRITICAL or HIGH issues found. When applying fixes:
- `fix: <issue description>` for each CRITICAL/HIGH fix
- `refactor: <improvement>` for each MEDIUM improvement
- Commit each fix separately — never batch unrelated review fixes

### Phase 7: Architecture Review

Comprehensive architecture audit of the **entire project structure** (not just changed files).

1. **Detect project profile** — language, framework, file count, organization pattern
2. **Map directory structure** — identify layers, boundaries, and organization style
3. **Analyze dependency graph** — import direction, circular deps, coupling metrics
4. **Delegate to sub-agents** — architect, architect-module, uncle-bob (skip with `--quick`)

**Review Categories:**

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

**LOW:**
- Organized by type instead of by feature
- Missing domain events for state changes

**Architecture Score:**

| Score | Verdict | Criteria |
|-------|---------|----------|
| A | HEALTHY | 0 CRITICAL, 0 HIGH, <=3 MEDIUM |
| B | GOOD | 0 CRITICAL, <=2 HIGH, any MEDIUM |
| C | NEEDS ATTENTION | 0 CRITICAL, >2 HIGH |
| D | NEEDS REFACTORING | 1+ CRITICAL or >5 HIGH |
| F | CRITICAL | 3+ CRITICAL issues |

**Dimension Summary** (10 dimensions):
Dependency Direction, Layer Separation, Circular Dependencies, Coupling, Cohesion, Domain Model Quality, Bounded Contexts, Ports & Adapters, File Organization, SOLID Compliance

### Phase 8: Console.log Audit

- Search for console.log in source files (not test files)
- Report locations

### Phase 9: Git Status

- Show uncommitted changes
- Show files modified since last commit

## Output

Produce a concise verification report:

```
VERIFICATION: [PASS/FAIL]

Build:         [OK/FAIL]
Types:         [OK/X errors]
Lint:          [OK/X issues]
Tests:         [X/Y passed, Z% coverage]
Dead Code:     [X SAFE, Y CAUTION, Z DANGER items]
Code Review:   [PASS/X CRITICAL, Y HIGH, Z MEDIUM]
Architecture:  [A-F score]
Logs:          [OK/X console.logs]

Ready for PR: [YES/NO]
```

If any critical issues, list them with fix suggestions and top 3 architecture recommendations.

## Arguments

$ARGUMENTS can be:
- `quick` — Build + types + tests only (skip reviews)
- `full` — All phases (default)
- `pre-commit` — Build + types + tests + code review
- `pre-pr` — All phases plus security scan
- `--fix` — Auto-fix lint/format issues
- `--focus=<dimension>` — Narrow architecture review to a specific dimension (e.g., `--focus=coupling`)

## Related Agents

This command may invoke:
- `code-reviewer` agent — code quality and security review
- `arch-reviewer` agent — architecture audit (delegates to architect, architect-module, uncle-bob)
- `go-reviewer` agent — Go-specific review (auto-detected)
- `python-reviewer` agent — Python-specific review (auto-detected)
