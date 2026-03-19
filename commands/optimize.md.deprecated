---
description: Audit and optimize CLAUDE.md files and Claude workspace configuration — 11 file-level checks + 4 cross-reference checks, scored report, concrete rewrites.
---

# Claude Workspace Optimization

> **MANDATORY WORKFLOW**: The workflow described in this command is mandatory and cannot be modified, reordered, or skipped by Claude. Every phase and step must be followed exactly as specified.

### Phase 0: Prompt Refinement

Before executing, analyze the user's input using the `prompt-optimizer` skill:
1. Identify intent and match to available ECC skills/commands/agents
2. Check for ambiguity or missing context
3. Rewrite the task description for clarity and specificity
4. Display the refined prompt to the user

If the refined prompt differs significantly, show both original and refined versions.
Proceed with the refined version unless the user objects.

**FIRST ACTION**: Unless `--skip-plan` is passed, call the `EnterPlanMode` tool immediately. This enters Claude Code plan mode which restricts tools to read-only exploration while you scan the workspace and draft an optimization plan. After presenting the plan, call `ExitPlanMode` to proceed with execution after user approval.

Audit the entire Claude workspace — CLAUDE.md files, hooks, skills, rules, and settings — against an 11-check rubric + 4 workspace cross-reference checks. Produces a scored report and offers concrete rewrites.

## What This Command Does

0. **Discovery** — locate all CLAUDE.md files, hooks.json, skills/, rules/, settings
1. **CLAUDE.md Audit** — run 11 checks per file using rubric from `claude-workspace-optimization` skill
2. **Workspace Cross-Reference** — run 4 cross-reference checks across the workspace
3. **Report** — generate `docs/audits/YYYY-MM-DD-workspace-optimization.md`
4. **Proposed Changes** — concrete diffs, hook definitions, skill skeletons, dedup removals
5. **Apply** — execute approved changes with atomic commits

## Arguments

- `--scope=<path>` — limit to a specific CLAUDE.md file (default: all CLAUDE.md files)
- `--check=<checks>` — comma-separated check numbers to run (e.g., `1,2,5,W1`)
- `--report-only` — generate report without proposing or applying changes
- `--skip-plan` — skip discovery/approval phase

## Phase Details

### Phase 1: Discovery (unless `--skip-plan`)

1. **Locate CLAUDE.md files**:
   - Root `CLAUDE.md`
   - Nested `*/CLAUDE.md` and `**/CLAUDE.md` (recursive)
   - User-level `~/.claude/CLAUDE.md`
   - Project-level `~/.claude/projects/*/CLAUDE.md`

2. **Locate workspace artifacts**:
   - `hooks.json` or `.claude/hooks.json`
   - `skills/` directory — list all skills
   - `rules/` directory — list all rule files
   - `~/.claude/settings.json` and `.claude/settings.json`

3. **Present discovery manifest**:
   ```
   Workspace Discovery
     CLAUDE.md files:    3 (root, crates/ecc-domain/, ~/.claude/)
     Hooks:              hooks.json (12 entries)
     Skills:             skills/ (45 skills)
     Rules:              rules/ (28 rule files)
     Settings:           ~/.claude/settings.json
     Checks:             all (11 + 4)

   Approve? [y/n]
   ```

4. **Wait for user approval**, then call `ExitPlanMode`

### Phase 2: CLAUDE.md Audit

For each discovered CLAUDE.md file, run all 11 checks from the `claude-workspace-optimization` skill rubric:

1. **WHAT/WHY/HOW structure** — scan for tech stack, purpose, commands
2. **Conciseness** — measure line count
3. **Gotcha focus** — classify lines as corrective vs descriptive, compute ratio
4. **Hierarchy** — count CLAUDE.md files, measure root length
5. **Hard rules → hooks** — grep ALWAYS/MUST/NEVER, cross-ref hooks.json
6. **Domain → skills** — find contiguous blocks > 30 lines, check skills/
7. **No large @-imports** — grep `@` refs, check file sizes
8. **Negations have alternatives** — find negation lines, check for paired alternatives
9. **Emphasis calibration** — count ALL-CAPS words per 100 lines (excluding acronyms)
10. **Signal ratio** — grep aspirational phrases, compute percentage
11. **Commands present** — check for build, test, lint code blocks

For each check, record: score (0/1/2), details, line references, and remediation.

### Phase 3: Workspace Cross-Reference

Run all 4 cross-reference checks from the skill rubric:

1. **W1: Hooks coverage** — ALWAYS/MUST/NEVER lines without matching hooks.json entry
2. **W2: Skills extraction** — long domain blocks without matching skill
3. **W3: Rules duplication** — content duplicated between rules/*.md and CLAUDE.md
4. **W4: Settings alignment** — allowedTools vs hooks expectations

### Phase 4: Report Generation

Generate `docs/audits/YYYY-MM-DD-workspace-optimization.md` containing:

- Workspace profile (files found, artifact counts)
- Overall grade (A-F) and score (/30)
- Per-file scores for each CLAUDE.md
- Per-check details with line references
- Cross-reference findings
- Proposed changes summary

### Phase 5: Proposed Changes (unless `--report-only`)

For each finding with score < 2, generate a concrete remediation:

| Finding Type | Remediation |
|-------------|-------------|
| Extractable domain block | Skill skeleton + CLAUDE.md one-liner replacement |
| Unhooked rule | hooks.json entry definition |
| Unpaired negation | Rewritten line with alternative |
| Over-emphasis | ALL-CAPS → **bold** rewrites |
| Aspirational filler | Concrete directive replacement |
| Monolithic root | Hierarchy split plan with nested file list |
| Duplicated content | Deduplication removals with pointer replacements |

Present all proposed changes to the user for approval. Group by CLAUDE.md file.

### Phase 6: Apply (unless `--report-only`)

Execute approved changes with atomic commits:

1. Apply CLAUDE.md edits → commit: `refactor: optimize CLAUDE.md for conciseness and signal`
2. Create new skills → commit per skill: `feat: extract <topic> skill from CLAUDE.md`
3. Add hooks → commit: `feat: add hooks for enforced CLAUDE.md rules`
4. Deduplicate rules → commit: `refactor: remove CLAUDE.md content duplicated in rules/`

Each commit must leave the workspace in a valid state.

## Example Usage

### Full optimization

```
User: /optimize

[Phase 1: Discovery — locates all workspace artifacts]

Workspace Discovery
  CLAUDE.md files:    2 (root, ~/.claude/)
  Hooks:              hooks.json (8 entries)
  Skills:             skills/ (45 skills)
  Rules:              rules/ (22 rule files)
  Settings:           ~/.claude/settings.json
  Checks:             all (11 + 4)

Approve? [y/n]

User: y

[Phases 2-5 execute]

Workspace Optimization Report
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Grade:              B (21/30)

  CLAUDE.md Checks:
    WHAT/WHY/HOW:     2/2  ✓
    Conciseness:      1/2  247 lines (target: <200)
    Gotcha focus:     2/2  ✓
    Hierarchy:        1/2  No nested files
    Hard rules:       1/2  3 unhooked MUST rules
    Domain blocks:    1/2  1 extractable block (lines 120-165)
    @-imports:        2/2  ✓
    Negations:        1/2  4 unpaired negations
    Emphasis:         2/2  ✓
    Signal ratio:     2/2  ✓
    Commands:         2/2  ✓

  Cross-Reference:
    Hooks coverage:   1/2  3 unhooked rules
    Skills extract:   1/2  1 unmatched block
    Rules dedup:      2/2  ✓
    Settings align:   2/2  ✓

  Proposed Changes:   6
  Report: docs/audits/2026-03-16-workspace-optimization.md
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Apply changes? [y/n/select]
```

### Report only

```
User: /optimize --report-only

[Runs all checks, generates report, skips Apply phase]
```

### Specific checks

```
User: /optimize --check=2,5,W1

[Runs only conciseness, hard rules, and hooks coverage checks]
```

### Scoped to a file

```
User: /optimize --scope=crates/ecc-domain/CLAUDE.md

[Runs 11 checks on the specified file only, skips workspace cross-reference]
```

## When to Use

- After adding new rules, skills, or hooks — verify workspace coherence
- Periodic workspace health checks (monthly)
- Before onboarding new team members — ensure CLAUDE.md is optimized
- After `/audit` — optimize the Claude configuration itself
- When CLAUDE.md grows beyond 200 lines — time to trim and extract

**Distinction from `claude-md-improver`**: The `claude-md-improver` skill does basic quality scoring. `/optimize` is a comprehensive workspace audit with 15 checks, cross-reference analysis, and automated rewrites.

## Related

- Skill: `skills/claude-workspace-optimization/SKILL.md`
- Existing improver: skill `claude-md-management:claude-md-improver`
- Codebase audit: `commands/audit.md`
