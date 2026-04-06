---
description: Backlog conformance audit — verify implemented entries match original intent, detect shadow implementations, run scoped tests, and generate conformance report.
allowed-tools: [Task, Read, Grep, Glob, LS, Bash, Write, TodoWrite]
---

# Backlog Conformance Audit

> **MANDATORY**: Follow every phase exactly. Narrate per `skills/narrative-conventions/SKILL.md`.

Verify implemented backlog entries satisfy original intent. Detects conformance drift, partial implementations, and shadow implementations. Report in `docs/audits/`.

Scope: $ARGUMENTS (or all implemented entries)

## Arguments

- `--no-run-tests` — skip test execution (artifact-only, faster)
- `--scope=BL-NNN` — audit single entry

Default: `--run-tests` ON (`cargo test -p <crate>` for entries referencing crates).

## Terminology

| Term | Definition |
|------|-----------|
| Conformance | Degree implementation satisfies backlog entry intent/ACs |
| Shadow implementation | Entry marked `open` whose artifacts actually exist |
| Verdict | PASS (all artifacts+ACs) / PARTIAL (some missing) / FAIL (key missing) / MISSING (nothing found) |
| Confidence | HIGH (commit+spec) / MEDIUM (artifacts, no reference) / LOW (partial matches) |

## 1. Scan Backlog Entries

Read `docs/backlog/BL-*.md`. Parse frontmatter (id, title, status, scope, tags).
- `implemented`/`promoted` → conformance audit
- `open` → shadow detection
- `archived`/`superseded` → skip

## 2. Conformance Audit (Implemented)

For each: **2a.** Git cross-ref (`git log --grep="BL-NNN"`), extract files + spec artifacts. **2b.** Verify referenced artifacts exist (file paths, CLI commands, functions). **2c.** Run tests if `--run-tests` active. **2d.** Evaluate ACs (structured `- [ ]`/`- [x]` or inferred from description). **2e.** Assign verdict: PASS/PARTIAL/FAIL/MISSING.

## 3. Shadow Detection (Open)

For each open entry: search git log, spec artifacts, described artifacts. Assign confidence: HIGH/MEDIUM/LOW/NONE. Flag HIGH/MEDIUM/LOW as shadows.

## 4. Report

Write to `docs/audits/backlog-conformance-YYYY-MM-DD.md`.

Structure: Project Profile, Conformance Summary (verdict counts + rate), Per-Entry Conformance table, Shadow Implementations table, Remediation Suggestions, Findings ([CONF-NNN]), Summary, Next Steps.

## 5. Present

Console: conformance rate, verdict distribution, shadow count by confidence, top 3 priorities, report path.

**STOP. DO NOT modify source code.** Say: "To act on findings, run `/spec`."

## When to Use

- After sprints to verify completeness
- Before releases
- Periodically to detect drift
- To find shadow implementations

## Related Agents

Inline analysis, no dedicated agent. For full audit including backlog: `/audit-full`.
