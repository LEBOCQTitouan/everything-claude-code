---
name: doc-quality-scoring
description: Rubric and methodology for scoring documentation quality across five dimensions — presence, accuracy, completeness, clarity, and currency.
origin: ECC
---

# Documentation Quality Scoring

A reproducible rubric for scoring documentation quality. Goes beyond presence checking to evaluate whether docs are accurate, complete, clear, and current.

## When to Activate

- Running `/doc-suite` or `/doc-validate`
- Reviewing documentation quality
- Setting documentation standards for a project
- Measuring documentation improvement over time

## Scoring Dimensions

### 1. Presence (0-10)

Does the documentation exist?

| Score | Criteria |
|-------|----------|
| 10 | All public items documented |
| 8 | >90% of public items documented |
| 6 | >70% of public items documented |
| 4 | >50% of public items documented |
| 2 | >25% of public items documented |
| 0 | <25% or no documentation |

### 2. Accuracy (0-10)

Does the documentation match the code?

| Score | Criteria |
|-------|----------|
| 10 | All param names, types, and return descriptions match code |
| 8 | Minor mismatches (e.g., param order different) |
| 6 | Some outdated descriptions but no wrong types |
| 4 | Multiple contradictions with code |
| 2 | Documentation describes different behavior than code |
| 0 | Documentation is actively misleading |

**Detection methods:**
- Compare `@param` names against actual function parameter names
- Compare `@returns` description against actual return type
- Check `@throws`/`@raises` against actual error paths
- Verify code examples produce described output

### 3. Completeness (0-10)

Does the documentation cover all aspects of the API?

| Score | Criteria |
|-------|----------|
| 10 | Description + all params + return + throws + examples |
| 8 | Description + all params + return |
| 6 | Description + most params |
| 4 | Description only (no param/return docs) |
| 2 | One-line description, no details |
| 0 | No meaningful content |

**Checklist per item:**
- [ ] Summary description (what it does)
- [ ] All parameters documented with types and descriptions
- [ ] Return value documented
- [ ] Error conditions / exceptions documented
- [ ] Side effects noted (if any)
- [ ] Usage example (for complex APIs)

### 4. Clarity (0-10)

Is the documentation readable and unambiguous?

| Score | Criteria |
|-------|----------|
| 10 | Clear, concise, uses domain terminology consistently |
| 8 | Readable, minor verbosity or jargon |
| 6 | Understandable but requires effort |
| 4 | Ambiguous or uses undefined terms |
| 2 | Confusing or contradictory within itself |
| 0 | Unintelligible or boilerplate only |

**Signals of poor clarity:**
- Auto-generated boilerplate (e.g., "Gets the value of X" for `getX()`)
- Circular definitions ("The user service services users")
- Undefined acronyms or domain terms
- Overly long descriptions (>5 sentences for a simple function)

### 5. Currency (0-10)

Is the documentation up to date?

| Score | Criteria |
|-------|----------|
| 10 | Doc modified in same commit as code changes |
| 8 | Doc updated within 1 week of code changes |
| 6 | Doc updated within 1 month of code changes |
| 4 | Doc updated within 3 months of code changes |
| 2 | Doc >3 months stale relative to code |
| 0 | Doc >1 year stale or code fundamentally changed since doc written |

**Detection methods:**
- `git log` for doc comment changes vs code changes in same file
- `git blame` to compare doc line timestamps vs code line timestamps
- File modification date relative to last code change

## Aggregate Scoring

### Per-Item Score

```
item_score = (presence + accuracy + completeness + clarity + currency) / 5
```

### Per-Module Score

```
module_score = sum(item_scores) / count(public_items)
```

### Overall Project Score

```
project_score = sum(module_scores * module_weight) / sum(module_weights)
```

Where `module_weight` = number of public items in module (larger modules matter more).

### Grade Mapping

| Grade | Score Range | Label |
|-------|------------|-------|
| A | 9.0 - 10.0 | Excellent |
| B | 7.0 - 8.9 | Good |
| C | 5.0 - 6.9 | Adequate |
| D | 3.0 - 4.9 | Poor |
| F | 0.0 - 2.9 | Failing |

## Contradiction Detection

### Type Mismatches

Compare doc-declared types against code types:
- `@param {string} id` but code has `id: number`
- `@returns {void}` but function returns a value
- Doc mentions param `options` but code has `config`

### Behavioral Contradictions

- Doc says "throws if invalid" but function returns null
- Doc says "async" but function is synchronous
- Doc mentions a default value that doesn't match code

### Cross-Reference Conflicts

- Same concept described differently in two locations
- Module README contradicts inline doc comments
- API docs describe different behavior than integration guide

## Duplicate Detection

Flag when the same symbol or concept is documented in 2+ places with conflicting descriptions:

1. Search for symbol name across all doc files
2. Extract description from each location
3. Compare descriptions for semantic equivalence
4. If descriptions conflict, flag with both locations and the differences

## Report Format

```markdown
## Documentation Quality: [Grade] ([Score]/10)

### Per-Module Scores

| Module | Presence | Accuracy | Completeness | Clarity | Currency | Overall |
|--------|----------|----------|--------------|---------|----------|---------|
| lib/   | 8        | 7        | 6            | 8       | 9        | 7.6 (B) |
| hooks/ | 4        | 6        | 3            | 5       | 7        | 5.0 (C) |

### Issues Found

| Severity | File | Issue |
|----------|------|-------|
| HIGH | src/lib/merge.ts:L45 | @param `manifest` removed from code but still in doc |
| MEDIUM | src/hooks/session.ts | Missing doc comment on 3 exported functions |
| LOW | README.md:L120 | Example uses deprecated API |

### Contradictions

| Location 1 | Location 2 | Conflict |
|------------|------------|----------|
| src/lib/utils.ts:L10 | docs/API.md:L45 | Different return type described |
```

## Related

- Agent: `agents/doc-validator.md`
- Command: `commands/doc-validate.md`
- Orchestrator: `agents/doc-orchestrator.md`
