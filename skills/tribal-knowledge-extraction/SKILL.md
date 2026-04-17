---
name: tribal-knowledge-extraction
description: Extract implicit tribal knowledge from codebases — configure patterns, common modification sites, failure patterns, cross-module dependencies, and undocumented conventions. Produces structured tribal knowledge for onboarding and compass context generation.
origin: ECC
---

# Tribal Knowledge Extraction

Atomic extraction skill for surfacing implicit, undocumented knowledge held by long-term contributors — the "why this exists", "how to change X", and "what breaks when Y". Produces structured output for onboarding documentation and compass context files.

## When to Activate

- Before generating compass context files (`skills/compass-context-gen/SKILL.md`)
- During onboarding documentation generation
- When writing runbooks for maintainers new to a module
- After a codebase audit when hidden coupling is suspected
- When doc-analyzer flags low doc coverage on high-churn modules

## Five Questions Framework

Apply these five questions to each module or component under analysis:

### Q1: How do I configure this component?

Scan for configuration entry points:

```
grep -r "env::\|std::env\|dotenv\|config\.\|Config::\|from_env\|from_config"
```

For each config point, record:
- Variable name and expected format
- Where it is consumed (file:line)
- What happens if it is absent or malformed (fallback vs panic)
- Whether it is documented in README or CLAUDE.md

### Q2: What is the most common modification to this module?

Identify high-churn change sites using git history:

```
git log --follow --format="%H" -- <module_path> | head -30 | xargs git show --stat | grep "^\s*[0-9]" | sort -rn | head -10
```

Classify each change site by type:
- `ADD_VARIANT` — new enum variant, command, agent, or rule
- `EXTEND_STRUCT` — new field on existing data type
- `FIX_LOGIC` — correction to business logic
- `DOC_UPDATE` — documentation-only changes
- `RENAME` — symbol or file renames

Record: file path → change type → frequency estimate (high/medium/low).

### Q3: What are the known failure patterns?

Delegate to `skills/failure-modes/SKILL.md` for structural failure extraction, then augment with tribal layer:

- Patterns that only appear under specific load or timing conditions
- Silent failures (no error returned, wrong output produced)
- Failures caused by environment drift (works on dev, fails in CI)
- Historical bugs that recurred after being "fixed" (regression traps)

Record each pattern: description, trigger, detection signal, last occurrence (git tag or date if known).

### Q4: What are the non-obvious cross-module dependencies?

Beyond import graphs, surface hidden coupling:

- Shared mutable state (globals, static variables, environment variables read in multiple places)
- Convention-based coupling (magic strings, naming patterns that must stay in sync)
- File-system contracts (paths, directory structures, file naming conventions)
- Ordering dependencies (module A must initialize before module B)
- Implicit type constraints not enforced by the type system

Record: module-A → couples-to → module-B, coupling type, risk if broken.

### Q5: What tribal knowledge should every contributor know?

Capture undocumented conventions and hard-won lessons:

- "Always run X before Y when changing Z"
- "This file has a 800-line limit — split it before adding more"
- "Hook H blocks writes in this directory — bypass requires `ecc bypass grant`"
- "Test T is flaky under high load — skip it in mutation runs"
- "Config key K was renamed from K2 — old name still works but is deprecated"

Elicit from: long comment blocks, TODO/FIXME notes, unusual guard clauses, and non-obvious test setup.

## Delegation

This skill delegates to:

- `skills/failure-modes/SKILL.md` — for Q3 structural failure extraction (blast radius, recovery procedures, error taxonomy)
- `skills/behaviour-extraction/SKILL.md` — for Q1/Q4 runtime behaviour and side-effect mapping that reveals hidden coupling

Apply failure-modes depth **shallow** by default; escalate to **deep** only for modules flagged as high-priority by doc-analyzer.

## Output Format

```
# Tribal Knowledge: <module-name>

## Configuration (Q1)
| Variable | Format | Fallback | Documented |
|----------|--------|----------|------------|
| ECC_METRICS_DISABLED | "1" or unset | disabled=false | CLAUDE.md |

## Common Modifications (Q2)
| File | Change Type | Frequency |
|------|-------------|-----------|
| src/dispatch.rs | ADD_VARIANT | high |

## Failure Patterns (Q3)
| Pattern | Trigger | Detection | Last Seen |
|---------|---------|-----------|-----------|
| Silent drop on queue full | queue capacity exceeded | metrics drop | v1.2.0 |

## Cross-Module Dependencies (Q4)
| Module A | Couples To | Type | Risk |
|----------|-----------|------|------|
| ecc-infra | ecc-domain | convention (enum names) | high — breaks dispatch |

## Tribal Knowledge (Q5)
- Run `cargo xtask mutants` before merging changes to ecc-domain
- Hook `pre:write-edit:worktree-guard` blocks all writes on main — call EnterWorktree first
```

## Edge Cases

### Zero-Marker Fallback

When a module has no git history, no env var usage, and no cross-module imports, output:

```
No embedded tribal knowledge detected for <module-name>.
Recommend: pair with a long-term contributor or review ADRs in docs/adr/.
```

Do not fabricate patterns. Prefer explicit "not found" over speculative output.

### New or Scaffolded Modules

For modules created within the last 5 commits, tribal knowledge is sparse by definition. Output a partial result with a note:

```
Module <name> is new (created <date>). Q2/Q3/Q5 data unavailable until churn history accumulates.
```

## Related

- Compass context generation: `skills/compass-context-gen/SKILL.md`
- Failure mode extraction: `skills/failure-modes/SKILL.md`
- Behaviour extraction: `skills/behaviour-extraction/SKILL.md`
- Documentation analyzer: `agents/doc-analyzer.md`
