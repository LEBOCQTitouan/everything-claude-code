---
name: clean-craft
description: Professional software craftsmanship principles — Programmer's Oath, design smells, Transformation Priority Premise, function discipline, Boy Scout Rule, and rework metrics.
origin: ECC
---

# Clean Craft

Professional software craftsmanship principles from Robert C. Martin's body of work. Covers the ethical and technical foundations of writing clean, professional code.

## When to Activate

- Code review (function discipline, naming, CQS)
- Planning and refactoring (design smells detection)
- TDD GREEN phase (Transformation Priority Premise)
- Session wrap-up (rework ratio, oath reflection)
- Architecture audits (structural design smell detection)

## The Programmer's Oath

Nine promises that define professional software craftsmanship:

1. **I will not produce harmful code.** — No code that is defective in behavior or structure.
2. **I will not make a mess.** — Degrading productivity is unprofessional.
3. **I will produce a proof.** — A test suite that covers as close to 100% as possible.
4. **I will make small, frequent releases.** — Never hold up downstream progress.
5. **I will fearlessly improve.** — Improve code at every opportunity; never make it worse.
6. **I will keep productivity high.** — Nothing that decreases throughput.
7. **I will continuously ensure easy substitution.** — Portable components, replaceable parts.
8. **I will produce estimates honestly.** — No false promises; communicate uncertainty.
9. **I will never stop learning and improving my craft.** — Continuous skill development.

### Oath Evaluation

When reviewing work against the oath, annotate findings with one-line "oath notes":

```
Oath 1 (no harmful code): CLEAN — no defective behavior or structure
Oath 3 (proof): WARNING — new endpoint /api/foo has no test coverage
Oath 5 (fearless improvement): CLEAN — Boy Scout improvements in 2 files
```

## Four Design Smells

Structural indicators that a codebase is degrading:

### Rigidity

The system is hard to change because every change forces cascade changes in other modules.

**Compound signals**:
- Dead code accumulation + rising cyclomatic complexity
- Single change touches 5+ files across 3+ modules
- Build times increasing without proportional feature growth

**Detection**: Count files changed per commit (rolling average). Track complexity trend via branching keyword count.

### Fragility

Changes in one area cause unexpected breakages in unrelated areas.

**Compound signals**:
- Low test coverage + high number of dependents (fan-in > 10)
- Bug fix commits that reference "also fixes" or "regression"
- Files with both high churn and high bug-fix commit ratio

**Detection**: Correlate test coverage per file with dependent count. Track fix-commit ratio per file.

### Immobility

Useful parts cannot be extracted for reuse because they are tangled with other parts.

**Compound signals**:
- High co-change coupling + no shared abstraction (interface/trait)
- Modules that import each other's internals (not just public API)
- Copy-paste patterns across modules (duplicated logic)

**Detection**: Analyze co-change coupling. Check for bidirectional imports. Scan for duplicated code blocks.

### Viscosity

Doing things the right way is harder than doing things the wrong way.

**Compound signals**:
- `console.log` / `println!` at system boundaries (bypassing logging infrastructure)
- Growing TODO/FIXME count over time
- Test-skipping patterns increasing (`skip`, `ignore`, `fixme`)
- Long build/test cycle discouraging frequent runs

**Detection**: Count debug logging at boundaries. Track TODO/FIXME trend over git history. Measure test skip rate.

## Transformation Priority Premise (TPP)

When making a failing test pass in the GREEN phase of TDD, prefer the simplest transformation that works. Transformations are ordered from simplest to most complex:

| Priority | Transformation | Example |
|----------|---------------|---------|
| 1 | `{} → nil` | Return nothing |
| 2 | `nil → constant` | Return a fixed value |
| 3 | `constant → variable` | Replace constant with a parameter |
| 4 | `add computation` | Simple arithmetic or string operation |
| 5 | `unconditional → selection` | Add `if` / `match` |
| 6 | `scalar → collection` | Single value → array/vec |
| 7 | `statement → tail recursion` | Loop via recursion |
| 8 | `selection → iteration` | `if` → `for` / `loop` |
| 9 | `value → mutated value` | Transform existing data (last resort) |

**Rule**: Always try the transformation with the lowest priority number first. If it doesn't make the test pass, move to the next. Jumping to complex transformations (e.g., adding iteration when a constant would suffice) leads to over-engineered solutions.

**Anti-pattern**: Going straight to iteration or recursion when a simple conditional would work.

## Function Discipline

### Length Thresholds

| Lines | Level | Action |
|-------|-------|--------|
| 1-20 | GOOD | Ideal range |
| 21-40 | WARNING | Consider extracting |
| 41+ | CAUTION | Must extract or justify |

### Abstraction Level (Stepdown Rule)

Functions should read like a newspaper — high-level summary at the top, details below. Each function should operate at a single level of abstraction.

**Violation signal**: A function that mixes named function calls (high level) with array indexing, bitwise operations, or raw string manipulation (low level).

### Argument Count

| Count | Classification | Guidance |
|-------|---------------|----------|
| 0 | Niladic | Ideal |
| 1 | Monadic | Good — transformation or event |
| 2 | Dyadic | Acceptable — natural pairs (x, y) |
| 3 | Triadic | Suspicious — consider parameter object |
| 4+ | Polyadic | Refactor — wrap in struct/object |

### Command-Query Separation (CQS)

Functions should either **do something** (command) or **answer something** (query), never both.

**Violation signal**: Function named `get*`, `find*`, `is*`, `has*`, `check*` that also mutates state. Or function named `set*`, `update*`, `create*`, `delete*` that returns a meaningful value beyond success/failure.

## Boy Scout Rule

> Always leave the code better than you found it.

During the REFACTOR phase (or any code edit), scan 3-5 nearby files for one small improvement:

**Candidates**:
- Remove a TODO/FIXME comment by doing the TODO
- Extract a magic number into a named constant
- Rename a vague identifier (`data` → `invoiceItems`)
- Remove dead code (unused import, unreachable branch)
- Add a missing type annotation

**Constraint**: One small improvement per session. Commit separately with `chore(scout): <description>`.

## "Go Well" Metric — Rework Ratio

Track the health of a development session by measuring the ratio of forward progress to rework:

```
rework_ratio = (fix_commits + chore_commits) / total_commits
```

| Ratio | Interpretation |
|-------|---------------|
| 0.00 - 0.20 | Healthy — mostly forward progress |
| 0.21 - 0.40 | Normal — some rework expected |
| 0.41 - 0.60 | Elevated — investigate friction sources |
| 0.61+ | Concerning — process or architecture problem |

**Trend tracking**: Compare rework ratio across sessions. Rising trend suggests growing technical debt.

## Related

- Agent: `agents/robert.md`
- Command: `commands/uncle-bob-audit.md`
- Complementary: `skills/tdd-workflow/SKILL.md`, `skills/coding-standards/SKILL.md`
