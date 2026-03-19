---
name: doc-drift-detector
description: Detect documentation-code drift by comparing doc claims against actual code state — produces CI-ready drift reports with severity and fix suggestions.
origin: ECC
---

# Documentation Drift Detector

Quality skill for finding places where documentation has drifted from the code it describes. Goes beyond staleness (time-based) to check semantic accuracy — does the doc still describe what the code actually does?

## When to Activate

- During doc-validator pipeline (complement to accuracy check)
- As a CI gate to prevent drift accumulation
- After large refactors that may invalidate existing docs
- When staleness analysis flags potential issues

## Methodology

### 1. Drift Categories

| Category | What Drifted | Detection Method |
|----------|-------------|------------------|
| **Signature drift** | Params, return types, function name | Compare doc `@param`/`@returns` against actual signature |
| **Behaviour drift** | What the function does | Compare doc description against code logic |
| **Structural drift** | Module layout, file locations | Compare doc references to actual file paths |
| **Config drift** | Env vars, defaults, CLI flags | Compare documented config against config-extraction data |
| **Example drift** | Code examples that no longer work | Attempt to compile/type-check examples |
| **Count drift** | Stated numbers (test count, file count) | Compare stated numbers against actual counts |

### 2. Signature Drift Detection

For each documented public symbol:

1. Parse the doc comment for parameter names and types
2. Parse the actual function/method signature
3. Compare:
   - Parameter names match?
   - Parameter types match? (for typed languages)
   - Return type matches?
   - Parameter count matches?
   - Optional/required status matches?

Flag mismatches:
```
DRIFT: src/lib/merge.ts:mergeDirectory
  Doc says: @param manifest (ManifestEntry[])
  Code has: @param entries (ManifestEntry[])
  Type: signature drift (parameter renamed)
  Severity: HIGH
  Fix: Rename @param manifest to @param entries in doc comment
```

### 3. Structural Drift Detection

Scan all documentation files for internal references:

1. Extract all file path references (`src/lib/merge.ts`, `docs/API-SURFACE.md`)
2. Check that each referenced path exists
3. Extract module/directory references in architecture docs
4. Verify they match actual directory structure

```
DRIFT: docs/ARCHITECTURE.md:45
  Doc says: "src/commands/ — Slash command definitions"
  Reality: Directory does not exist (moved to commands/)
  Type: structural drift
  Severity: MEDIUM
  Fix: Update path to "commands/"
```

### 4. Config Drift Detection

Compare documented configuration against actual code:

1. Read documented env vars from README, CLAUDE.md, or docs/
2. Run config-extraction methodology against source code
3. Flag:
   - Documented vars not in code (removed?)
   - Code vars not in docs (undocumented?)
   - Default value mismatches
   - Required/optional status mismatches

### 5. Severity Classification

| Severity | Criteria | CI Action |
|----------|----------|-----------|
| **CRITICAL** | Instructions that would cause errors if followed (wrong commands, wrong paths) | Block merge |
| **HIGH** | Incorrect parameter docs, wrong types, missing required params | Block merge |
| **MEDIUM** | Outdated descriptions, stale counts, missing sections | Warn |
| **LOW** | Style drift, minor wording issues, cosmetic | Info only |

### 6. Output Format

CI-ready report format:

```
# Documentation Drift Report
Generated: 2026-03-14
Files scanned: 45 source, 12 doc

## Summary
Total drift issues: 8 (1 CRITICAL, 2 HIGH, 3 MEDIUM, 2 LOW)
Drift score: 73/100 (higher = less drift)

## Issues

### CRITICAL
- [ ] CLAUDE.md:15 — Test command `npm run test:all` does not exist. Actual: `npm test`

### HIGH
- [ ] src/lib/merge.ts:45 — @param `manifest` renamed to `entries` in code
- [ ] docs/API-SURFACE.md:120 — Function `resolveConflicts` removed from codebase

### MEDIUM
- [ ] README.md:30 — States "1401 tests" but actual count is 1456
- [ ] docs/ARCHITECTURE.md:45 — References src/commands/ (moved to commands/)
- [ ] CLAUDE.md:80 — Missing 2 new npm scripts from table

### LOW
- [ ] docs/domain/glossary.md:12 — "merge strategy" definition slightly outdated
- [ ] README.md:55 — Badge URL uses old repo name
```

## CI Integration

The drift report can be consumed by CI pipelines:

- Exit code 1 if any CRITICAL or HIGH issues
- Exit code 0 with warnings for MEDIUM/LOW
- Machine-readable JSON output with `--format=json`

## Related

- Doc validator agent: `agents/doc-validator.md`
- Doc gap analyser: `skills/doc-gap-analyser/SKILL.md`
- Config extraction: `skills/config-extraction/SKILL.md`
- Symbol extraction: `skills/symbol-extraction/SKILL.md`
