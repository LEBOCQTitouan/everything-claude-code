---
name: symbol-extraction
description: Extract public symbols, types, signatures, and visibility from source code — the atomic unit of API surface analysis.
origin: ECC
---

# Symbol Extraction

Atomic extraction skill for identifying and cataloguing all public symbols in a codebase. Produces structured data that downstream skills (api-reference-gen, doc-gap-analyser) and agents (doc-analyzer, doc-generator) consume.

## When to Activate

- As part of the doc-analyzer pipeline (Step 2: Module Inventory)
- Before generating API reference documentation
- When auditing documentation coverage
- When building a public API surface inventory

## Methodology

### 1. Identify Export Boundaries

Scan each source file for language-specific export signals:

| Language | Public Signal | Internal Signal |
|----------|--------------|-----------------|
| TypeScript/JS | `export`, `export default`, `module.exports` | No export keyword, `_` prefix |
| Python | Listed in `__all__`, no `_` prefix | `_` prefix, not in `__all__` |
| Go | Capitalized identifiers | Lowercase identifiers |
| Java | `public` access modifier | `private`, `protected`, package-private |
| Rust | `pub`, `pub(crate)` | No `pub` keyword |

### 2. Classify Each Symbol

For every exported symbol, record:

| Field | Description |
|-------|-------------|
| `name` | Symbol name as exported |
| `kind` | `function`, `class`, `type`, `interface`, `constant`, `enum`, `variable` |
| `file` | Source file path (relative to project root) |
| `line` | Line number of declaration |
| `signature` | Full type signature (params + return type) |
| `visibility` | `public`, `package`, `crate` (language-dependent) |
| `documented` | Boolean — has doc comment above declaration |
| `deprecated` | Boolean — marked with `@deprecated`, `#[deprecated]`, etc. |

### 3. Resolve Re-exports

Track barrel files (`index.ts`, `__init__.py`, `mod.rs`) that re-export from submodules:

1. Follow re-export chains to the original declaration
2. Record both the re-export path and the original path
3. Flag conflicting re-exports (same name from different sources)

### 4. Extract Type Signatures

For typed languages, capture full signatures:

- **Functions**: parameter names, types, return type, generic constraints
- **Classes**: constructor params, public methods, public properties
- **Interfaces/Types**: all fields with types, extends/implements
- **Enums**: variant names and values
- **Constants**: type and value (if literal)

For untyped languages (JS, Python without annotations), infer from:
- JSDoc `@param`/`@returns` tags
- Default parameter values
- Runtime type checks in function body

## Output Format

Structured per-module symbol inventory:

```
Module: src/lib/
  Symbols: 45 (38 documented, 7 undocumented)

  function mergeDirectory(src: string, dest: string, opts?: MergeOptions): MergeResult
    file: src/lib/merge.ts:23  |  documented: yes  |  deprecated: no

  interface MergeOptions { strategy: MergeStrategy; dryRun?: boolean }
    file: src/lib/merge.ts:8   |  documented: yes  |  deprecated: no

  type MergeStrategy = 'overwrite' | 'skip' | 'prompt'
    file: src/lib/merge.ts:5   |  documented: no   |  deprecated: no
```

## Edge Cases

- **Overloaded functions**: List each overload as a separate entry
- **Default exports**: Use `[default]` as the name, note the actual identifier
- **Dynamic exports** (`export * from`): Follow the chain, mark as `(re-export)`
- **Computed property names**: Skip — not reliably extractable without AST
- **Conditional exports** (package.json `exports` field): Document all conditions

## Related

- Language-specific patterns: `skills/symbol-extraction/references/language-patterns.md`
- Consumer agent: `agents/doc-analyzer.md`
- Downstream skill: `skills/api-reference-gen/SKILL.md`
- Analysis skill: `skills/doc-analysis/SKILL.md`
