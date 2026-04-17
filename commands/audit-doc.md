---
description: Documentation audit — coverage analysis, staleness detection, drift identification, and placement violation checks.
allowed-tool-set: audit-command
---

# Documentation Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Narrative**: See narrative-conventions skill.

Multi-agent documentation analysis with additional direct placement checks. Produces a dated report in `docs/audits/` with actionable findings.

Scope: $ARGUMENTS (or full codebase if none provided)

## Arguments

- `--scope=<path>` — limit to subdirectory (default: entire repo)

## 1. Analysis

### 1a. Agent Analysis (Parallel)

Launch **two agents in parallel**:

#### `doc-analyzer` Agent (allowedTools: [Read, Grep, Glob, Bash])
Evaluates:
- **Coverage** — percentage of public API surface with documentation
- **Module summaries** — presence and completeness of module-level docs
- **Domain concepts** — documented vs undocumented domain terms
- **Dependency documentation** — dependency purpose and usage documented

#### `doc-validator` Agent (allowedTools: [Read, Grep, Glob, Bash])
Evaluates:
- **Accuracy** — doc claims match actual code behavior
- **Staleness** — docs that reference removed or renamed entities
- **Code examples** — examples that compile/run correctly
- **Contradictions** — conflicting information across doc files
- **Duplicates** — same information repeated in multiple places

### 1b. Placement Violation Checks (Post-Agent)

After both agents complete, perform additional checks directly:

1. **Volatile content in `docs/`** — scan for content that changes with every release and belongs as code comments instead (e.g., implementation details, line-specific references)
2. **Missing architecture decisions** — check if structural decisions referenced in code have corresponding entries in `docs/ARCHITECTURE.md`
3. **Decisions without ADRs** — identify architectural decisions mentioned in commit messages or comments that lack an ADR in `docs/adr/`
4. **Oversized README** — check if `README.md` exceeds 100 lines of content (excluding blank lines and badges)
5. **Oversized CLAUDE.md** — check if `CLAUDE.md` exceeds 120 lines of content
6. **Mixed doc types** — identify files that mix tutorial-style content with reference-style content


## Adversarial Challenge


### Adversary Gate

If the aggregate finding count from the analysis phase is <3 AND all findings are MEDIUM severity or lower (threshold rationale: low-signal audits provide insufficient material for meaningful adversarial review — see BL-121 finding 4.5), skip the adversary challenge:

> "Adversary challenge skipped: N findings, all ≤MEDIUM severity."

Otherwise, proceed with the challenger launch below.

> After the analysis phase completes, launch an independent adversary to challenge the findings.

Launch a Task with the `audit-challenger` agent (allowedTools: [Read, Grep, Glob, Bash, WebSearch]):

- Pass the findings from the analysis phase as structured input (finding ID, severity, description, evidence)
- The agent independently re-interrogates the codebase and searches web for best practices
- Collect challenged findings: confirmed, refuted, or amended with per-finding rationale

### Quality Check

If the adversary output lacks structured per-finding verdicts (each with finding ID, verdict {confirmed|refuted|amended}, and rationale):
1. Retry once with a stricter prompt demanding the exact output format
2. If second attempt still lacks structure, surface a "Low-quality adversary output" warning alongside the raw content and proceed

### Disagreement Handling

When audit and adversary disagree on a finding:
- Display both the original finding and the challenger's assessment side by side
- Include the challenger's recommendation
- Prompt the user for final decision: accept audit / accept challenger / custom resolution

### Graceful Degradation

If the audit-challenger agent fails to spawn or returns an error:
- Emit: "Adversary challenge skipped: <reason>"
- Proceed with unchallenged findings

## 2. Report

Merge findings from agents and placement checks. Write to `docs/audits/doc-YYYY-MM-DD.md` using today's date.

Report structure:

```markdown
# Documentation Audit — YYYY-MM-DD

## Project Profile
- **Repository**: <repo name>
- **Scope**: <audited path or "full codebase">
- **Date**: YYYY-MM-DD
- **Agents**: doc-analyzer, doc-validator + direct placement checks

## Health Grade

| Grade | Criteria |
|-------|----------|
| **A** | 0 CRITICAL, 0 HIGH, ≤3 MEDIUM |
| **B** | 0 CRITICAL, ≤2 HIGH |
| **C** | 0 CRITICAL, >2 HIGH |
| **D** | 1+ CRITICAL or >5 HIGH |
| **F** | 3+ CRITICAL |

**Grade: X**

## Coverage Summary

| Module | Public API | Documented | Coverage |
|--------|-----------|------------|----------|
| ... | N | N | N% |

## Placement Violations

| Violation | Location | Recommendation |
|-----------|----------|---------------|
| Volatile content in docs/ | ... | Move to code comment |
| Missing ADR | ... | Create ADR |
| Oversized README | ... | Extract to sub-docs |
| Mixed doc types | ... | Split tutorial/reference |

## Findings

### [DOC-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Location**: file:line-range
- **Principle**: The violated documentation principle
- **Evidence**: Concrete data (coverage numbers, stale references, drift examples)
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
- Overall documentation coverage percentage
- Placement violation count
- Finding counts by severity
- Top 3 most critical findings
- Report file path

**STOP. DO NOT modify source code.**

Say: "To act on findings, run `/spec` referencing this report."

## When to Use

- Before releases to ensure docs are current
- After major refactoring that may have left docs stale
- Periodic documentation health checks

## Related Agents

- `agents/doc-analyzer.md` — codebase documentation analyzer
- `agents/doc-validator.md` — documentation accuracy validator
