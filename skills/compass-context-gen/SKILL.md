---
name: compass-context-gen
description: Generate compact compass context files (25-35 lines, <1000 tokens) for ECC components — crates, agents, commands, skills, hooks, and rules. Provides maintainers with a fast-access orientation file covering quick commands, key files, non-obvious patterns, and cross-references.
origin: ECC
---

# Compass Context Generation

Skill for producing and updating `docs/context/<component>.md` — lightweight orientation files that answer "where do I start?" for any ECC component. Compass files are idempotent: running generation a second time updates in place rather than duplicating content.

## When to Activate

- After implementing a new ECC component (crate, agent, command, skill, hook, rule, team)
- When a component's key files or patterns change significantly during the TDD loop
- When `/implement` Phase 7.5 dispatches `compass-context-writer`
- When running a documentation refresh across the codebase

## Component Types Covered

| Type | Example | Source Root |
|------|---------|-------------|
| Crate | `ecc-domain` | `crates/<name>/` |
| Agent | `doc-analyzer` | `agents/<name>.md` |
| Command | `/implement` | `commands/<name>.md` |
| Skill | `failure-modes` | `skills/<name>/SKILL.md` |
| Hook | `pre:edit-write:worktree-guard` | `hooks/<name>.md` |
| Rule | `coding-style` | `rules/common/<name>.md` |
| Team | `implement-team` | `teams/<name>.md` |

## Output Location

`docs/context/<component-name>.md`

Example: compass for `ecc-domain` → `docs/context/ecc-domain.md`

## Compass File Structure

Each compass file contains exactly four sections, in this order:

### Quick Commands

3-5 shell commands a maintainer runs most often for this component. Include the exact command string with no placeholders left unexplained.

```markdown
## Quick Commands
- `cargo test -p ecc-domain` — run domain unit tests
- `cargo clippy -p ecc-domain -- -D warnings` — lint domain only
- `cargo mutants -p ecc-domain` — mutation test domain logic
```

### Key Files

3-5 files that are the primary entry points or most frequently modified. One line per file: path + one-sentence description.

```markdown
## Key Files
- `crates/ecc-domain/src/lib.rs` — public API surface; all domain types re-exported here
- `crates/ecc-domain/src/dispatch.rs` — wave dispatch logic; modify when adding new PC types
- `crates/ecc-domain/src/workflow.rs` — state machine transitions; gated by phase-gate hook
```

### Non-Obvious Patterns

2-4 patterns that trip up new contributors. Each entry states the pattern and the consequence of violating it.

```markdown
## Non-Obvious Patterns
- ecc-domain has zero I/O imports — adding any `std::fs` or `std::net` call fails the pre-commit hook
- Enum variants in `PcType` must stay in sync with the dispatch table in `ecc-infra` — convention coupling, not enforced by types
- `SystemTime::now()` debt in `dispatch.rs:47` — tracked in TODO; do not add more `now()` calls without a wrapper
```

### Cross-References

2-4 links to related components, ADRs, or docs that a maintainer should read before modifying this component.

```markdown
## Cross-References
- `docs/adr/0055-bypass-grant.md` — why bypass is audited, not env-var
- `skills/wave-dispatch/SKILL.md` — dispatch algorithm this crate implements
- `agents/tdd-executor.md` — primary consumer of domain types
```

## Line Budget

A compass file MUST stay within **25-35 lines** of content (excluding the file header comment). Target is <1000 tokens. This constraint is intentional — compass files are for fast orientation, not comprehensive documentation.

If content exceeds 35 lines:
1. Trim Quick Commands to 3 (drop least-used)
2. Trim Key Files to 3 (drop secondary entry points)
3. Merge Non-Obvious Patterns entries that share a root cause
4. Reduce Cross-References to 2 (keep highest-signal links)

Never exceed 35 lines — refer maintainers to full docs instead of bloating the compass.

## Update-in-Place Behavior

The compass-context-gen skill is idempotent:

1. If `docs/context/<component>.md` does not exist → create it fresh
2. If it already exists → read it, diff the four sections against current source, update only changed sections, preserve unchanged sections verbatim
3. Never duplicate sections; never append a second copy of a section

A regeneration run MUST produce byte-identical output if the source component has not changed.

## Generation Procedure

### Step 1 — Identify component type and root

Determine component type from the file path or name passed by the caller. Locate root files using the Component Types table above.

### Step 2 — Gather source material

Read at most 5 source files from the component root. For crates, prioritize `lib.rs`, `main.rs`, and the highest-churn file from git. For agents/commands, read the single `.md` file. Apply `tribal-knowledge-extraction` Q1, Q2, Q4, Q5 to extract pattern material.

### Step 3 — Draft four sections

Draft each section independently:
- Quick Commands: derive from Makefile, CLAUDE.md, or crate `Cargo.toml` test/lint targets
- Key Files: pick files with highest import fan-in or most recent modification
- Non-Obvious Patterns: use Q4 (cross-module deps) and Q5 (tribal knowledge) output
- Cross-References: use ADR links from commit messages, related agent/skill references in source

### Step 4 — Apply line budget

Count lines. If over 35, apply trimming rules (see Line Budget section). If under 25, expand Non-Obvious Patterns with one additional entry or add one more Key File.

### Step 5 — Write or update

Write the file with the standard header:

```markdown
<!-- compass: <component-name> | updated: YYYY-MM-DD -->
```

Then the four sections in order: Quick Commands, Key Files, Non-Obvious Patterns, Cross-References.

## Related

- Tribal knowledge extraction: `skills/tribal-knowledge-extraction/SKILL.md`
- Compass context writer agent: `agents/compass-context-writer.md`
- Module summary updater: `agents/module-summary-updater.md`
- Documentation guidelines: `skills/doc-guidelines/SKILL.md`
