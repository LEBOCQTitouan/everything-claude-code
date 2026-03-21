---
description: Code quality audit — SOLID principles, clean code, naming quality, function size, complexity, and craftsmanship assessment.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Code Quality Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.

Multi-agent code quality analysis of the codebase. Produces a dated report in `docs/audits/` with actionable findings.

Scope: $ARGUMENTS (or full codebase if none provided)

## Arguments

- `--scope=<path>` — limit to subdirectory (default: entire repo)

## 1. Analysis

Launch **two agents in parallel**:

### 1a. `code-reviewer` Agent (allowedTools: [Read, Grep, Glob, Bash])
Evaluates:
- **Code readability** — naming clarity, comment quality, self-documenting code
- **Function size** — functions exceeding 50 lines
- **File size** — files exceeding 800 lines
- **Nesting depth** — nesting exceeding 4 levels
- **Duplication** — copy-paste code and near-duplicates
- **Dead code** — unused functions, variables, imports
- **Security** — basic security patterns (input validation, injection prevention)
- **Maintainability** — complexity, coupling, cohesion at function/class level

### 1b. `uncle-bob` Agent (allowedTools: [Read, Grep, Glob])
Evaluates (diagnosis only — never produces code):
- **SRP** (Single Responsibility) — each module/class has one reason to change
- **OCP** (Open/Closed) — open for extension, closed for modification
- **LSP** (Liskov Substitution) — subtypes substitutable for base types
- **ISP** (Interface Segregation) — no client forced to depend on unused methods
- **DIP** (Dependency Inversion) — depend on abstractions, not concretions
- **Clean Architecture** — dependency rule compliance
- **Meaningful naming** — names reveal intent, avoid disinformation
- **Small functions** — functions do one thing, one level of abstraction

## 2. Report

Merge findings from both agents. Write to `docs/audits/code-YYYY-MM-DD.md` using today's date.

Report structure:

```markdown
# Code Quality Audit — YYYY-MM-DD

## Project Profile
- **Repository**: <repo name>
- **Scope**: <audited path or "full codebase">
- **Date**: YYYY-MM-DD
- **Agents**: code-reviewer, uncle-bob

## Health Grade

| Grade | Criteria |
|-------|----------|
| **A** | 0 CRITICAL, 0 HIGH, ≤3 MEDIUM |
| **B** | 0 CRITICAL, ≤2 HIGH |
| **C** | 0 CRITICAL, >2 HIGH |
| **D** | 1+ CRITICAL or >5 HIGH |
| **F** | 3+ CRITICAL |

**Grade: X**

## SOLID Compliance

| Principle | Status | Key Violations |
|-----------|--------|---------------|
| SRP | ✅/⚠️/❌ | ... |
| OCP | ✅/⚠️/❌ | ... |
| LSP | ✅/⚠️/❌ | ... |
| ISP | ✅/⚠️/❌ | ... |
| DIP | ✅/⚠️/❌ | ... |

## Code Metrics

| Metric | Value | Threshold | Status |
|--------|-------|-----------|--------|
| Functions > 50 lines | N | 0 | ✅/❌ |
| Files > 800 lines | N | 0 | ✅/❌ |
| Max nesting depth | N | 4 | ✅/❌ |
| Dead code instances | N | 0 | ✅/❌ |
| Duplication instances | N | 0 | ✅/❌ |

## Findings

### [CODE-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: file:line-range
- **Principle**: The violated code quality principle
- **Evidence**: Concrete data (line counts, complexity scores, examples)
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

To act on these findings, run `/plan` referencing this report.
```

## 3. Present

Display a console summary:
- Health grade
- SOLID compliance (one-line per principle)
- Code metrics summary
- Finding counts by severity
- Top 3 most impactful findings
- Report file path

**STOP. DO NOT modify source code.**

Say: "To act on findings, run `/plan` referencing this report."

## When to Use

- After significant feature implementation
- Before code review or PR submission
- Periodic code quality health checks

## Related Agents

- `agents/code-reviewer.md` — code quality and security reviewer
- `agents/uncle-bob.md` — Clean Architecture and SOLID consultant
