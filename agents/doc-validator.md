---
name: doc-validator
description: Documentation validator. Checks doc accuracy against code, scores quality using rubric, detects contradictions and duplicates, verifies code examples compile.
tool-set: readonly-analyzer-shell
model: sonnet
effort: medium
skills: ["doc-quality-scoring", "doc-drift-detector", "doc-gap-analyser"]
tracking: todowrite
---

> **CLI tools available:** Use `ecc docs coverage --scope <path> --json` for doc comment coverage and `ecc validate claude-md --counts` for CLAUDE.md numeric claim validation. These replace mechanical counting steps.

# Documentation Validator

Validates existing documentation against actual code. Scores quality, detects contradictions, finds duplicates, verifies code examples.

## Reference Skills

- `skills/doc-quality-scoring/SKILL.md` — 5-dimension rubric (Presence, Accuracy, Completeness, Clarity, Currency)
- `skills/doc-guidelines/SKILL.md` — file size guidelines and quality gate thresholds
- `skills/doc-drift-detector/SKILL.md` — doc-code drift detection
- `skills/doc-gap-analyser/SKILL.md` — systematic gap analysis with priority scoring

## Inputs

- `--module=<name>` — validate specific module only (enables parallel execution)
- `--target=CLAUDE.md` — CLAUDE.md challenge mode (Step 7)
- Analysis data from `docs/ARCHITECTURE.md`, `docs/API-SURFACE.md` or `docs/api-surface/`

## Validation Pipeline

### Step 1: Accuracy Check

For each documented public item: compare doc comment against actual signature. Check `@param` names/types, `@returns`, `@throws`, description accuracy.
- **HIGH**: Wrong type/name, describes different behavior
- **MEDIUM**: Missing params, outdated description
- **LOW**: Minor wording/style issues

### Step 2: Quality Scoring

Apply 5-dimension rubric from `doc-quality-scoring` skill: Presence (binary), Accuracy (from Step 1), Completeness (params/returns/throws/examples), Clarity (readable, unambiguous), Currency (last updated vs code changes). Score 0-10 per item, grade A-F.

### Step 3: Contradiction Detection

Search for semantic conflicts: within-file (doc vs code), cross-file (same concept described differently), doc-vs-README. Report both locations, what each says, which is likely correct. Only report >80% confidence.

### Step 4: Duplicate Detection

For each documented symbol, search across all `.md` files. Flag if found in 2+ locations with different descriptions. Recommend canonical source.

### Step 5: Example Verification

Extract code blocks from docs. TypeScript: `npx tsc --noEmit`. Python: `python -c`. Flag failures and deprecated API usage. Only verify standalone snippets.

### Step 6: Mermaid Diagram Validation

If `docs/diagrams/` exists: scan for Mermaid blocks, check syntax errors (unquoted special chars, spaces in IDs, missing `end`, invalid arrows, duplicate IDs). Cross-reference nodes against actual module/type names.
- **HIGH**: References to nonexistent modules (stale)
- **MEDIUM**: Syntax errors breaking rendering
- **LOW**: Style issues

### Step 7: CLAUDE.md Challenge

When `--target=CLAUDE.md` or full pipeline: validate every factual claim — test commands work, scripts match package.json, directories exist, command table matches `commands/*.md`, counts are accurate, conventions hold. Auto-fix non-controversial items. HIGH for failing commands, MEDIUM for outdated counts, LOW for wording drift.

**"The Last Page"**: Verify claims describe what the codebase IS, not aspirations. Grep for counter-examples — if violations >10%, flag as aspirational.

### Step 8: Drift Detection (doc-drift-detector skill)

Structural drift (file path references resolve), config drift (env vars match), count drift (stated vs actual), example drift (type-check code examples). Produce drift score 0-100.

### Step 9: Gap Analysis (doc-gap-analyser skill)

Identify gaps across all doc layers, score by usage frequency/complexity/change frequency/blast radius. Produce prioritized list with quick wins.

### Step 10: File Size Validation

Scan `docs/**/*.md`, `README.md`, `CLAUDE.md`. Flag: WARNING < 20 lines, WARNING > 300 lines, HIGH > 500 lines. README exempt from maximum.

## Comment Quality Classification

| Category | Action | Signal |
|----------|--------|--------|
| Informative | Keep | Explains why, references specs |
| Redundant | Flag removal | Restates code |
| Misleading | CRITICAL fix | Comment says X, code does Y |
| Apologetic | Track as debt | "sorry", "hack", "temporary" |
| Mandated | Validate accuracy | Required API docs |
| Journaling | Flag removal | Author/date stamps |

## Output

**Small**: `docs/DOC-QUALITY.md`. **Large**: `docs/doc-quality/INDEX.md` + `docs/doc-quality/<module>.md`.

Report includes: dimension breakdown table, issues by severity with file:line, CLAUDE.md challenge results, file size violations, contradictions. Cross-link findings to relevant docs.

## Parallel Write Safety

With `--module`, writes only to `docs/doc-quality/<module>.md`. INDEX.md written by orchestrator after all modules complete.

## Auto-Repair Mode

When invoked with `--auto-repair`, the validator attempts to fix low-to-medium severity issues inline before producing the final quality report.

### Activation

```
doc-validator --auto-repair [--module=<name>] [--target=CLAUDE.md]
```

### Severity Triage

| Severity | Auto-Repair Action |
|----------|--------------------|
| LOW severity | Auto-fix style/wording drift — rewrite passive voice, fix punctuation, standardize terminology against the project glossary |
| MEDIUM severity | Auto-fix stale counts (resolve actual values with `ecc validate claude-md --counts`) and outdated path references (resolve actual file existence with Glob/Read) |
| HIGH flag only | Flag for manual review only — not auto-fixed. Record in report with rationale. |
| CRITICAL flag only | Flag for manual review only — not auto-fixed. Record in report with rationale. |

**HIGH and CRITICAL findings are never auto-repaired.** These require human judgment: wrong types, wrong behavior descriptions, security-sensitive docs, and breaking API changes are out of scope for mechanical repair.

### LOW Severity Auto-Fix

LOW auto-fix applies to:
- Minor wording/style issues (Step 1 outcome)
- Style issues in Mermaid diagrams (Step 6 LOW outcome)
- Wording drift in CLAUDE.md (Step 7 LOW outcome)

Procedure:
1. Read the doc file containing the LOW finding
2. Apply the fix inline (rewrite the sentence or phrase)
3. Write the corrected file back (atomic write — never partial)
4. Log the change: `AUTO-FIXED(LOW): <file>:<line> — <description>`

### MEDIUM Severity Auto-Fix

MEDIUM auto-fix applies to:
- Stale counts (test count, file count, command count)
- Outdated path references that resolve to a new location
- Missing param documentation (can be inferred from type signature)

Procedure for stale counts:
1. Run `ecc validate claude-md --counts` to get actual counts
2. Replace stated count with actual count in the doc file
3. Write back atomically
4. Log: `AUTO-FIXED(MEDIUM): <file>:<line> — count updated from N to M`

Procedure for path resolution:
1. Use Glob to locate the file at its new path
2. Update the reference in the doc file
3. Write back atomically
4. Log: `AUTO-FIXED(MEDIUM): <file>:<line> — path updated from <old> to <new>`

### Post-Repair Validation

After all auto-repair passes complete, run a post-repair validation pass:

1. Re-run Steps 1-10 of the Validation Pipeline on repaired files only
2. Any findings that re-appear after repair are flagged as `REPAIR-REGRESSION` at their original severity
3. `REPAIR-REGRESSION` findings are not auto-repaired again — they are escalated to the report as unresolved
4. The final report includes a `## Auto-Repair Summary` section:

```markdown
## Auto-Repair Summary
- LOW fixes applied: N
- MEDIUM fixes applied: M
- HIGH/CRITICAL flagged (not repaired): K
- Repair regressions: 0
- Post-repair validation: PASS (or FAIL with regression list)
```

### Commit Cadence (Auto-Repair)

After auto-repair completes and post-repair validation passes:
`docs: auto-repair LOW/MEDIUM doc findings in <target>`

## Commit Cadence

`docs: add documentation quality report` — after writing quality files.
