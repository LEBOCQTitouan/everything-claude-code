---
description: Test architecture audit — coverage analysis, test classification, fixture ratios, structural coupling, and E2E boundary coverage matrix.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Test Architecture Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.

Focused test architecture analysis of the codebase. Includes special E2E boundary coverage analysis for hexagonal architectures. Produces a dated report in `docs/audits/` with actionable findings.

Scope: $ARGUMENTS (or full codebase if none provided)

## Arguments

- `--scope=<path>` — limit to subdirectory (default: entire repo)

## 1. Analysis

### 1a. Standard Test Architecture Analysis

Invoke the `test-auditor` agent with full codebase access (allowedTools: [Read, Grep, Glob, Bash]).

The agent evaluates:
- **Test classification** — unit, integration, E2E test distribution
- **Structural coupling** — tests tightly coupled to implementation details
- **Fixture ratios** — test setup complexity vs assertion density
- **Coverage gaps** — untested modules, functions, and branches
- **Test isolation** — shared state, test ordering dependencies
- **Test naming** — clarity and consistency of test descriptions
- **Mock usage** — appropriate use of mocks vs real dependencies

### 1b. E2E Boundary Coverage (Special)

After the agent completes, perform additional boundary analysis directly:

1. **Scan port definitions**: Glob `crates/ecc-ports/src/` for trait definitions (port traits)
2. **Map adapters**: For each port trait, find its concrete adapter(s) in `crates/ecc-infra/`
3. **Check E2E coverage**: For each adapter, verify E2E test coverage exists in `crates/ecc-integration-tests/`
4. **Verify markers**: Check for `// e2e-boundary:` markers and validate they match actual port traits
5. **Report gaps**: Adapters without E2E tests are HIGH severity findings

This produces the "E2E Boundary Coverage Matrix" in the report.

## 2. Report

Write findings to `docs/audits/test-YYYY-MM-DD.md` using today's date.

Report structure:

```markdown
# Test Architecture Audit — YYYY-MM-DD

## Project Profile
- **Repository**: <repo name>
- **Scope**: <audited path or "full codebase">
- **Date**: YYYY-MM-DD
- **Agent**: test-auditor + direct boundary analysis

## Health Grade

| Grade | Criteria |
|-------|----------|
| **A** | 0 CRITICAL, 0 HIGH, ≤3 MEDIUM |
| **B** | 0 CRITICAL, ≤2 HIGH |
| **C** | 0 CRITICAL, >2 HIGH |
| **D** | 1+ CRITICAL or >5 HIGH |
| **F** | 3+ CRITICAL |

**Grade: X**

## Test Distribution

| Type | Count | Percentage |
|------|-------|------------|
| Unit | N | N% |
| Integration | N | N% |
| E2E | N | N% |

## E2E Boundary Coverage Matrix

| Port Trait | Adapter | E2E Tests | Marker | Status |
|-----------|---------|-----------|--------|--------|
| FileSystem | OsFileSystem | ✅/❌ | ✅/❌ | COVERED/MISSING |
| ShellExecutor | ProcessExecutor | ✅/❌ | ✅/❌ | COVERED/MISSING |
| ... | ... | ... | ... | ... |

## Findings

### [TEST-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: file:line-range
- **Principle**: The violated testing principle
- **Evidence**: Concrete data (coverage numbers, test counts, ratios)
- **Risk**: What breaks if unaddressed
- **Remediation**: Directional fix (what, not how)

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | N |
| HIGH     | N |
| MEDIUM   | N |
| LOW      | N |

## Top 5 Recommendations

1. ...
2. ...
3. ...
4. ...
5. ...

## Next Steps

To act on these findings, run `/spec` referencing this report.
```

## 3. Present

Display a console summary:
- Health grade
- Test distribution breakdown
- E2E boundary coverage (ports covered / total ports)
- Finding counts by severity
- Top 3 most critical findings
- Report file path

**STOP. DO NOT modify source code.**

Say: "To act on findings, run `/spec` referencing this report."

## When to Use

- Before releases to verify test coverage adequacy
- After adding new ports or adapters (hexagonal architecture)
- When test suite feels brittle or slow

## Related Agents

- `agents/test-auditor.md` — primary agent for test architecture analysis
