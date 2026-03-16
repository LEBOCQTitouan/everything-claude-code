---
name: claude-workspace-optimization
description: Audit rubric for CLAUDE.md files and Claude workspace health — 11 file-level checks + 4 cross-reference checks, scored 0-30 with A-F grading and concrete rewrite patterns.
origin: ECC
---

# Claude Workspace Optimization

A reproducible rubric for auditing CLAUDE.md quality and workspace coherence. Evaluates structure, conciseness, gotcha focus, hierarchy, hook coverage, skill extraction opportunities, emphasis calibration, and cross-workspace consistency.

## When to Activate

- Running `/optimize`
- Auditing CLAUDE.md quality beyond basic scoring
- Checking workspace coherence (hooks, skills, rules alignment)
- Onboarding to a project and evaluating its Claude configuration
- After adding new rules, hooks, or skills to verify consistency

## CLAUDE.md Audit Rubric (11 Checks)

Each check scores 0 (FAIL), 1 (PARTIAL), or 2 (PASS). Maximum: 22 points.

### Check 1: WHAT/WHY/HOW Structure

Does the file communicate what the project is, why it exists, and how to work with it?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | All 3 present: tech stack/purpose, project goals/context, build/test/lint commands |
| 1 (PARTIAL) | 2 of 3 present |
| 0 (FAIL) | 1 or fewer present |

**Detection**: Scan for technology mentions, purpose/overview section, and code-fenced commands.

### Check 2: Conciseness

Is the file short enough to fit in context without waste?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | < 200 lines |
| 1 (PARTIAL) | 200-400 lines |
| 0 (FAIL) | > 400 lines |

**Detection**: `wc -l` on the file.

### Check 3: Gotcha Focus

Does the file prioritize corrective guidance (gotchas, surprises, non-obvious behavior) over descriptive content (obvious facts derivable from code)?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | > 60% corrective content |
| 1 (PARTIAL) | 30-60% corrective content |
| 0 (FAIL) | < 30% corrective content |

**Detection**: Classify each non-header, non-blank line as corrective (warnings, exceptions, edge cases, "don't", "always", "note:", gotcha patterns) vs descriptive (file listings, obvious structure, boilerplate). Compute ratio.

**Corrective signals**: ALWAYS, NEVER, MUST, NOTE, WARNING, IMPORTANT, "don't", "avoid", "instead", "except", "gotcha", "caveat", "surprise", "unlike", "beware".

### Check 4: Hierarchy

Is guidance distributed across root and nested CLAUDE.md files appropriately?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | Root < 200 lines AND nested CLAUDE.md files exist for submodules |
| 1 (PARTIAL) | Root < 200 lines but no nested files, OR nested files exist but root > 200 |
| 0 (FAIL) | Single monolithic CLAUDE.md > 300 lines with no nesting |

**Detection**: Count all CLAUDE.md files in the project. Measure root file length.

### Check 5: Hard Rules to Hooks

Are enforceable rules (ALWAYS/MUST/NEVER) backed by hooks?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | All ALWAYS/MUST/NEVER rules have matching hooks.json entries |
| 1 (PARTIAL) | Some rules are hooked |
| 0 (FAIL) | > 5 enforceable rules with no matching hooks |

**Detection**: Grep CLAUDE.md for lines containing ALWAYS/MUST/NEVER. Cross-reference with hooks.json matchers. An "unhooked rule" is an enforceable directive with no automated enforcement.

### Check 6: Domain Knowledge to Skills

Are large domain-knowledge blocks extractable to skills?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | 0 extractable blocks (all domain knowledge is in skills/) |
| 1 (PARTIAL) | 1-2 contiguous blocks > 30 lines of domain-specific content |
| 0 (FAIL) | 3+ extractable blocks |

**Detection**: Identify contiguous non-header blocks > 30 lines covering a single domain topic. Check if a corresponding skill exists in skills/.

### Check 7: No Large @-imports

Are @-referenced files reasonably sized?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | 0 @-imports referencing files > 500 lines |
| 1 (PARTIAL) | 1-2 large @-imports |
| 0 (FAIL) | 3+ large @-imports |

**Detection**: Grep for `@` references in CLAUDE.md. Check referenced file sizes.

### Check 8: Negations Have Alternatives

Do "don't/never/avoid" directives tell the reader what to do instead?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | All negation lines are paired with an alternative in the same or next sentence |
| 1 (PARTIAL) | > 50% paired |
| 0 (FAIL) | < 50% paired |

**Detection**: Find lines containing "don't", "never", "avoid", "do not". Check if the same line or the next line contains "instead", "use", "prefer", "rather", or provides a concrete alternative.

### Check 9: Emphasis Calibration

Is ALL-CAPS emphasis used sparingly?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | < 3 ALL-CAPS words per 100 lines |
| 1 (PARTIAL) | 3-6 ALL-CAPS words per 100 lines |
| 0 (FAIL) | > 6 ALL-CAPS words per 100 lines |

**Detection**: Count words that are entirely uppercase and > 2 characters (excluding acronyms like API, SQL, HTML, CSS, URL, CLI, TDD, DDD, CI, CD, PR, MCP, ORM, REST, JSON, YAML, TOML, SDK, AWS, GCP). Divide by line count, multiply by 100.

**Common acronym allowlist**: API, SQL, HTML, CSS, URL, CLI, TDD, DDD, CI, CD, PR, MCP, ORM, REST, JSON, YAML, TOML, SDK, AWS, GCP, HTTP, HTTPS, DNS, TCP, UDP, SSH, TLS, SSL, JWT, OAuth, CORS, CSRF, XSS, SSRF, OWASP, DRY, SOLID, SRP, OCP, LSP, ISP, DIP, IDE, REPL, EOF, README, CLAUDE, TODO, WIP, UI, UX.

### Check 10: Signal Ratio

Is the file free of aspirational filler?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | < 5% aspirational lines |
| 1 (PARTIAL) | 5-15% aspirational lines |
| 0 (FAIL) | > 15% aspirational lines |

**Detection**: Count lines containing aspirational phrases. Divide by total non-blank, non-header lines.

**Aspirational phrase patterns**: "strive to", "aim to", "try to", "should ideally", "when possible", "where appropriate", "as needed", "consider", "feel free", "you may want", "it's recommended", "best practice is", "in general", "typically", "usually", "often", "sometimes", "might want to", "could consider".

### Check 11: Commands Present

Does the file include build, test, and lint commands?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | All 3 present in code blocks: build, test, lint |
| 1 (PARTIAL) | 2 of 3 present |
| 0 (FAIL) | 1 or fewer present |

**Detection**: Search for code-fenced blocks containing build/compile, test, and lint/check/clippy/eslint commands.

## Workspace Cross-Reference Checks (4 Checks)

Each check scores 0 (FAIL), 1 (PARTIAL), or 2 (PASS). Maximum: 8 points.

### W1: Hooks Coverage

Are enforceable CLAUDE.md rules backed by hooks?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | All ALWAYS/MUST/NEVER lines have a matching hook |
| 1 (PARTIAL) | > 50% coverage |
| 0 (FAIL) | < 50% coverage |

**Detection**: Extract ALWAYS/MUST/NEVER lines from all CLAUDE.md files. For each, search hooks.json for a matcher that could enforce it. Report unhooked rules.

### W2: Skills Extraction

Are domain knowledge blocks in CLAUDE.md already covered by skills?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | 0 extractable domain blocks without matching skills |
| 1 (PARTIAL) | 1-2 unmatched blocks |
| 0 (FAIL) | 3+ unmatched blocks |

**Detection**: Identify contiguous blocks > 20 lines in CLAUDE.md with domain-specific content. Check if skills/ contains a matching skill by topic.

### W3: Rules Duplication

Is content duplicated between rules/*.md and CLAUDE.md?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | No significant content overlap |
| 1 (PARTIAL) | 1-2 duplicated sections |
| 0 (FAIL) | 3+ duplicated sections |

**Detection**: For each rules/*.md file, check if similar content appears in any CLAUDE.md. Flag near-duplicate paragraphs or identical directives.

### W4: Settings Alignment

Do allowedTools and hook expectations align?

| Score | Criteria |
|-------|----------|
| 2 (PASS) | All hooks reference tools that are in allowedTools, no conflicts |
| 1 (PARTIAL) | Minor misalignment (1-2 tools) |
| 0 (FAIL) | Hooks expect tools not in allowedTools, or conflicts exist |

**Detection**: Read settings.json allowedTools. Compare with tools referenced in hooks.json. Flag mismatches.

## Scoring Aggregation

```
total_score = sum(11 CLAUDE.md checks) + sum(4 workspace checks)
max_score = 30
```

### Grade Mapping

| Grade | Score Range | Label |
|-------|------------|-------|
| A | 25 - 30 | Excellent |
| B | 19 - 24 | Good |
| C | 13 - 18 | Adequate |
| D | 7 - 12 | Poor |
| F | 0 - 6 | Failing |

## Rewrite Patterns

### Skill Extraction Pattern

When Check 6 or W2 finds extractable domain blocks:

1. Create `skills/<topic>/SKILL.md` with standard frontmatter
2. Move the domain content into the skill file
3. Replace the CLAUDE.md block with a one-line reference: "See `skills/<topic>/` for [topic] patterns"

### Hook Creation Pattern

When Check 5 or W1 finds unhooked rules:

1. Identify the rule's trigger (which tool, which event)
2. Create a hooks.json entry with appropriate matcher
3. Add the enforcement command or notification
4. Remove or soften the CLAUDE.md directive to reference the hook

### Negation Rewrite Pattern

When Check 8 finds unpaired negations:

```
BEFORE: "Don't use console.log for debugging"
AFTER:  "Don't use console.log for debugging — use the structured logger (src/lib/logger.ts) instead"
```

### Hierarchy Split Pattern

When Check 4 finds a monolithic root file:

1. Identify module-specific sections in root CLAUDE.md
2. Create nested `<module>/CLAUDE.md` files for each section
3. Keep only project-wide guidance in root (< 200 lines)
4. Add one-line pointers from root to nested files

### Emphasis Reduction Pattern

When Check 9 finds over-emphasis:

1. Replace ALL-CAPS with **bold** for emphasis
2. Reserve ALL-CAPS for acronyms only
3. Use `> **Note:**` callouts for critical warnings instead of inline CAPS

### Aspirational Removal Pattern

When Check 10 finds filler:

```
BEFORE: "You should strive to write clean, maintainable code when possible"
AFTER:  "Write functions < 50 lines. Extract helpers at 3+ usages."
```

Replace vague aspirations with concrete, measurable directives.

## Report Format

```markdown
## Workspace Optimization Report

**File**: `CLAUDE.md`
**Grade**: B (21/30)
**Date**: YYYY-MM-DD

### CLAUDE.md Checks

| # | Check | Score | Details |
|---|-------|-------|---------|
| 1 | WHAT/WHY/HOW | 2/2 | All sections present |
| 2 | Conciseness | 1/2 | 247 lines (target: <200) |
| ... | ... | ... | ... |

### Workspace Cross-Reference

| # | Check | Score | Details |
|---|-------|-------|---------|
| W1 | Hooks coverage | 1/2 | 3 unhooked MUST rules |
| ... | ... | ... | ... |

### Proposed Changes

1. **Extract testing guidelines to skill** (Check 6, lines 120-165)
2. **Add hook for "MUST run clippy"** (W1)
3. **Rewrite 4 unpaired negations** (Check 8)
```

## Related

- Command: `commands/optimize.md`
- Existing quality scorer: `skills/doc-quality-scoring/SKILL.md`
- Existing improver: skill `claude-md-management:claude-md-improver`
