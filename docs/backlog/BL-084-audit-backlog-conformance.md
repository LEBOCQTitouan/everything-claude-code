---
id: BL-084
title: "Backlog conformance audit — verify implementations match original backlog intent"
scope: MEDIUM
target: /spec dev
status: implemented
created: 2026-03-28
tier: null
tags: [audit, backlog, conformance, traceability]
---

# BL-084: Backlog Conformance Audit

## Problem

There are 39 implemented backlog entries and growing. No mechanism exists to verify that implementations actually satisfy the original backlog intent — acceptance criteria may have drifted, features may be partially implemented, or tests may not cover the original requirements.

## Proposed Solution

Create a new `/audit-backlog` command that:

1. **Scans all implemented/promoted backlog entries** in `docs/backlog/`
2. **Cross-references each entry** against git history (commits, specs, designs) to find the corresponding implementation
3. **Detects shadow implementations** — entries still marked `open` but actually implemented without formal promotion
4. **Verifies full conformance** for each entry:
   - Code exists (file paths, functions, CLI commands mentioned in the backlog entry)
   - Tests pass (run `cargo test` scoped to relevant crates/modules)
   - Original acceptance criteria / intent from the backlog entry are satisfied
5. **Generates a report** to `docs/audits/backlog-conformance-YYYY-MM-DD.md` with:
   - Per-entry verdict: PASS / PARTIAL / FAIL / MISSING
   - Summary statistics (conformance rate, gap count)
   - Remediation suggestions for PARTIAL/FAIL entries
   - List of shadow implementations needing status update

## Ready-to-Paste Prompt

```
/spec dev

Create `/audit-backlog` — a new audit command that verifies implemented backlog entries
actually satisfy their original intent.

**What it does:**
1. Read all backlog entries from docs/backlog/ with status implemented or promoted
2. For each entry, cross-reference git log and docs/specs/ to find the implementation
3. Also scan open entries to detect shadow implementations (implemented but not promoted)
4. For each implementation found:
   a. Verify code artifacts exist (grep for file paths, functions, CLI commands from the entry)
   b. Run relevant tests (cargo test scoped to affected crates)
   c. Evaluate whether the original acceptance criteria / description are satisfied
5. Generate a conformance report at docs/audits/backlog-conformance-YYYY-MM-DD.md

**Report format:**
- Per-entry table: ID | Title | Verdict (PASS/PARTIAL/FAIL/MISSING) | Evidence | Gaps
- Summary: conformance rate, total entries audited, gap count
- Shadow implementations: entries marked open but actually implemented
- Remediation: actionable suggestions for PARTIAL/FAIL entries

**Implementation notes:**
- Follow the /audit-* command pattern (read-only analysis + report generation)
- Use git log --all --oneline --grep="BL-NNN" to find commits per entry
- Use Explore agent for deep cross-referencing when commit messages don't mention BL IDs
- Scope: MEDIUM — single command file + audit agent, no new architecture
- Should work as both a standalone command and as a domain in /audit-full
```

## Dependencies

- BL-029 (persisted spec artifacts make cross-referencing easier)
- BL-066 (deterministic backlog management improves ID traceability)

## Acceptance Criteria

- [ ] `/audit-backlog` command exists and follows `/audit-*` conventions
- [ ] All implemented entries are audited with per-entry verdicts
- [ ] Shadow implementations (open but actually done) are detected
- [ ] Report persisted to `docs/audits/backlog-conformance-YYYY-MM-DD.md`
- [ ] Integrates as a domain in `/audit-full` orchestration
