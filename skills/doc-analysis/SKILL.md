---
name: doc-analysis
description: Methodology for analyzing codebases to identify documentation needs — public API surface, domain concepts, module boundaries, and call graphs.
origin: ECC
---

# Documentation Analysis

Systematic methodology for analyzing a codebase to understand what needs documenting, what exists, and what's missing. Language-agnostic with language-specific heuristics.

## When to Activate

- Running `/doc-suite` or `/doc-analyze`
- Onboarding documentation for a new project
- Auditing documentation completeness
- Before generating doc comments or summaries

## Core Concepts

### Public API Surface

The set of symbols explicitly exported for external consumption. Everything else is internal implementation.

| Language | Public Signal | Internal Signal |
|----------|--------------|-----------------|
| TypeScript/JS | `export`, `export default`, `module.exports` | No export keyword, `_` prefix |
| Python | Listed in `__all__`, no `_` prefix | `_` prefix, not in `__all__` |
| Go | Capitalized identifiers | Lowercase identifiers |
| Java | `public` access modifier | `private`, `protected`, package-private |
| Rust | `pub`, `pub(crate)` | No `pub` keyword |

### Module Boundary Detection

A module is a directory or file that forms a cohesive unit:

1. **Directory with index/barrel file** — `index.ts`, `__init__.py`, `mod.rs`
2. **Standalone file with exports** — a single file exporting multiple symbols
3. **Package boundary** — `package.json`, `go.mod`, `Cargo.toml` at directory root

### Domain Concept Detection

Extract domain terms from code:

1. **Type/class names** — `OrderService`, `UserRepository` → "Order", "User"
2. **Enum values** — `Status.PENDING`, `Role.ADMIN` → "Status", "Role"
3. **Function name patterns** — `calculateDiscount()` → "Discount"
4. **Constants** — `MAX_RETRY_COUNT` → "Retry"
5. **Frequency threshold** — include only terms appearing in 3+ files

### Call Graph Tracing

Lightweight import-based call graph (not full AST):

1. Grep for import/require statements
2. Map importer → imported module
3. Limit depth to 2 levels from entry points
4. Flag `(partial trace)` for dynamic imports or re-exports

## Analysis Steps

### Step 1: Project Profile

- Detect language and framework
- Count source files per directory
- Identify entry points
- Determine if small (<50 files) or large (50+) codebase

### Step 2: Module Inventory

For each module directory:
- Count source files
- List exported symbols (functions, classes, types, constants)
- List internal-only symbols
- Note dependencies (imports from other modules)

### Step 3: Documentation Inventory

For each exported symbol:
- Check for doc comment (JSDoc, docstring, godoc, rustdoc)
- If present: extract param names, return type description, description text
- If absent: mark as undocumented

### Step 4: Domain Concept Extraction

- Scan all type names, class names, interface names
- Extract noun components (split camelCase/PascalCase/snake_case)
- Group by frequency
- Filter: keep terms appearing in 3+ files
- Categorize: domain terms vs infrastructure terms

### Step 5: Dependency Mapping

- Build module-to-module import graph
- Annotate each edge with doc status (both ends documented? one? neither?)
- Detect circular dependencies
- Identify hub modules (high fan-in)

## Output Format

The analysis produces structured data that downstream agents consume. Output goes to `docs/` — either as single files (small codebase) or partitioned folders (large codebase).

### Size Thresholds

| Topic | Single file if | Folder if |
|-------|---------------|-----------|
| API Surface | <100 exports total | 100+ exports |
| Module Summaries | <10 modules | 10+ modules |
| Glossary | <50 terms | 50+ terms |
| Quality Report | <10 modules | 10+ modules |
| Coverage Report | <10 modules | 10+ modules |

## Language-Specific Heuristics

### TypeScript/JavaScript

```
Export patterns: export function, export class, export const, export type, export interface, export default, module.exports
Doc format: /** JSDoc */ or /** TSDoc */
Import patterns: import { X } from, import X from, require()
```

### Python

```
Export patterns: __all__ list, top-level def/class without _ prefix
Doc format: """docstring""" (first line of function/class body)
Import patterns: import X, from X import Y
```

### Go

```
Export patterns: Capitalized function/type/var names
Doc format: // Comment directly above declaration
Import patterns: import "pkg", import ( "pkg1" "pkg2" )
```

### Rust

```
Export patterns: pub fn, pub struct, pub enum, pub trait, pub mod
Doc format: /// doc comment, //! module doc
Import patterns: use crate::, use super::, use external_crate::
```

## Related

- Agent: `agents/doc-analyzer.md`
- Command: `commands/doc-analyze.md`
- Orchestrator: `agents/doc-orchestrator.md`
