---
description: Backlog conformance audit — verify implemented entries match original intent, detect shadow implementations, run scoped tests, and generate conformance report.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Backlog Conformance Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Narrative**: See narrative-conventions skill.

Verify that implemented backlog entries actually satisfy their original intent. Detects conformance drift, partial implementations, and shadow implementations (entries marked `open` but actually done). Produces a dated report in `docs/audits/`.

Scope: $ARGUMENTS (or all implemented entries if none provided)

## Arguments

- `--no-run-tests` — skip `cargo test` execution (artifact-only verification, faster)
- `--scope=BL-NNN` — audit a single entry instead of all

By default, `--run-tests` is ON: the audit runs `cargo test -p <crate>` for entries that reference specific crates.

## Terminology

| Term | Definition |
|------|-----------|
| **Conformance** | Degree to which an implementation satisfies the original backlog entry's intent and acceptance criteria |
| **Shadow implementation** | A backlog entry still marked `open` whose described artifacts actually exist in the codebase |
| **Verdict** | Per-entry assessment: PASS (fully implemented), PARTIAL (some artifacts missing), FAIL (key artifacts absent), MISSING (no implementation found) |
| **Confidence tier** | For shadow detection: HIGH (commit mentions BL-NNN + spec exists), MEDIUM (artifacts exist but no explicit reference), LOW (partial matches only) |

## 1. Scan Backlog Entries

Read all files in `docs/backlog/` matching `BL-*.md`. For each entry:

1. Parse YAML frontmatter to extract `id`, `title`, `status`, `scope`, `tags`
2. **Implemented entries** (status = `implemented` or `promoted`): queue for conformance audit
3. **Open entries**: queue for shadow detection
4. **Other statuses** (archived, superseded): skip silently

Report entry count: "Found N implemented, M open, K skipped entries."

## 2. Conformance Audit (Implemented Entries)

For each implemented entry, evaluate conformance:

### 2a. Git History Cross-Reference

Run `git log --all --oneline --grep="BL-NNN"` to find commits mentioning the entry ID.

If commits found:
- Extract file paths from commits: `git show --name-only <SHA>` for each commit
- Check for spec artifacts: `docs/specs/*/spec.md` containing BL-NNN reference
- Record as **evidence**

### 2b. Artifact Verification

Parse the backlog entry's markdown body for references to:
- File paths (anything matching `crates/`, `commands/`, `agents/`, `skills/`, `hooks/`, `rules/`, `docs/`)
- CLI commands (anything matching `ecc <subcommand>`)
- Function/type names (identifiers after "implement", "add", "create")

For each referenced artifact, verify it exists:
- File paths: `test -f <path>` or glob match
- CLI commands: check if the command/subcommand exists in the CLI enum or command files
- Functions: grep for the name in the referenced crate

### 2c. Test Verification (unless --no-run-tests)

If the entry references specific crates and `--run-tests` is active:
- Run `cargo test -p <crate> 2>&1` for each referenced crate
- Record test result (pass count, fail count)
- Include in verdict evidence

### 2d. Acceptance Criteria Evaluation

If the entry has structured ACs (lines starting with `- [ ]` or `- [x]`):
- For each AC, check if the described behavior/artifact exists
- Count satisfied vs unsatisfied ACs

If the entry has NO structured ACs (early entries BL-001 through BL-009):
- Infer from the entry description
- Note "inferred from description" in evidence

### 2e. Assign Verdict

| Verdict | Criteria |
|---------|----------|
| PASS | All referenced artifacts exist + all ACs satisfied (or inferred satisfied) |
| PARTIAL | Some artifacts exist but not all, or some ACs unsatisfied |
| FAIL | Key artifacts missing or majority of ACs unsatisfied |
| MISSING | No commits, no spec, no artifacts found |

## 3. Shadow Detection (Open Entries)

For each open entry:

### 3a. Evidence Gathering

1. Search git log: `git log --all --oneline --grep="BL-NNN"`
2. Search for spec artifacts: `ls docs/specs/*BL-NNN* 2>/dev/null` or grep spec files
3. Search for described artifacts in codebase (same heuristic as 2b)

### 3b. Assign Confidence

| Confidence | Criteria |
|------------|---------|
| HIGH | Commit mentions BL-NNN AND (spec exists OR all described artifacts present) |
| MEDIUM | Artifacts exist but no commit references BL-NNN explicitly |
| LOW | Partial artifact matches only (some files exist, others don't) |
| NONE | No evidence found — entry is genuinely open |

Only flag entries with confidence HIGH, MEDIUM, or LOW as shadow implementations.

## 4. Report

Write findings to `docs/audits/backlog-conformance-YYYY-MM-DD.md` using today's date.

Report structure:

```markdown
# Backlog Conformance Audit — YYYY-MM-DD

## Project Profile
- **Repository**: <repo name>
- **Date**: YYYY-MM-DD
- **Entries audited**: N implemented, M open (shadow scan)
- **Tests run**: yes/no (--run-tests flag)

## Conformance Summary

| Verdict | Count | Percentage |
|---------|-------|------------|
| PASS | N | X% |
| PARTIAL | N | X% |
| FAIL | N | X% |
| MISSING | N | X% |

**Conformance Rate**: X% (PASS / total implemented)

## Per-Entry Conformance

| ID | Title | Status | Verdict | Evidence | Gaps |
|----|-------|--------|---------|----------|------|
| BL-001 | ... | implemented | PASS | 3 commits, artifacts verified | — |
| BL-042 | ... | implemented | PARTIAL | CLI command exists, missing tests | No unit tests for background mode |

## Shadow Implementations

Entries marked `open` but with evidence of implementation:

| ID | Title | Confidence | Evidence | Recommendation |
|----|-------|------------|----------|----------------|
| BL-NNN | ... | HIGH | 5 commits, spec exists, CLI command works | Promote to `implemented` |
| BL-NNN | ... | MEDIUM | Artifacts exist, no BL reference in commits | Verify and promote |
| BL-NNN | ... | LOW | Partial matches | Review manually |

## Remediation Suggestions

For each PARTIAL or FAIL entry, provide a specific actionable suggestion:

- **BL-NNN (PARTIAL)**: <what's missing and how to fix it>
- **BL-NNN (FAIL)**: <what needs to be implemented or closed>

## Findings

### [CONF-NNN] Finding Title
- **Severity**: CRITICAL | HIGH | MEDIUM | LOW
- **Entry**: BL-NNN
- **Evidence**: Concrete data
- **Remediation**: Directional fix

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | N |
| HIGH     | N |
| MEDIUM   | N |
| LOW      | N |

## Next Steps

To act on these findings:
- Run `/spec` referencing specific BL-NNN entries for PARTIAL/FAIL verdicts
- Update backlog status for confirmed shadow implementations
- Close entries that are no longer relevant
```

## 5. Present

Display a console summary:
- Conformance rate (X% PASS)
- Verdict distribution (N PASS, N PARTIAL, N FAIL, N MISSING)
- Shadow implementation count (N HIGH, N MEDIUM, N LOW confidence)
- Top 3 remediation priorities
- Report file path

**STOP. DO NOT modify source code.**

Say: "To act on findings, run `/spec` referencing this report."

## When to Use

- After a development sprint to verify completeness
- Before releases to ensure all claimed features are actually implemented
- Periodically to detect conformance drift
- To find shadow implementations that need status updates

## Related Agents

This command does not delegate to a dedicated agent. It performs sequential analysis inline, following the simpler audit command pattern (similar to `/audit-convention`).

For full codebase audits including backlog conformance, use `/audit-full` which includes this as a domain.
