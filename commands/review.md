---
description: Robert professional conscience check
---

# Uncle Bob Audit

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.
>
> **Narrative**: See narrative-conventions skill.
> Explain what the Programmer's Oath evaluation means: `robert` checks recent work against professional craftsmanship promises.

Standalone invocation of the `robert` meta-agent for professional craftsmanship evaluation.

## What This Command Does

1. **Invoke `robert` agent** — evaluate the current session/codebase against the Programmer's Oath
2. **Display compact summary** — oath evaluation, self-audit findings, rework ratio

## How It Works

1. Invoke the `robert` agent (allowedTools: [Read, Grep, Glob, Bash]) for diagnostic evaluation
2. The agent evaluates:
   - **Oath check**: relevant Programmer's Oath promises against recent work
   - **Self-audit**: ECC agent/command/skill files for SRP/DRY/consistency issues
   - **"Go well" metric**: rework ratio from recent git log
3. Capture robert's structured Markdown output from conversation
4. Write the output to `docs/audits/robert-notes.md` (create `docs/audits/` if needed, overwrite existing file)
5. Display the summary to the user

## When to Use

- After completing a feature or significant work session
- As a periodic craftsmanship health check
- When you want to evaluate the quality of the development process (not just the code)
- To audit ECC's own configuration for internal consistency

## Example Usage

```
User: /uncle-bob-audit

Robert — Craft Health Audit
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Oath Evaluation:
  Oath 1 (no harmful code): CLEAN
  Oath 3 (proof):           WARNING — 1 endpoint without tests
  Oath 5 (improvement):     CLEAN — 2 Boy Scout commits

Self-Audit:
  [SELF-001] DRY: "Commit Cadence" duplicated in 4 files

"Go Well" Metric:
  Commits: 14 | Forward: 10 | Rework: 3 | Ratio: 0.21 (Normal)

Report: docs/audits/robert-notes.md
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

## Related Agents

This command invokes:
- `robert` agent — professional conscience evaluation

## Sources Reference

If `docs/sources.md` exists:
1. Read `docs/sources.md` and find entries for the reviewed module (via module mapping table)
2. List matched sources as reference context for the reviewer
3. Update `last_checked` date on matched entries

If `docs/sources.md` does not exist, skip this step silently.
