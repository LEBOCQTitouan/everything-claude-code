---
name: api-reference-gen
description: Generate API reference documentation from symbol, behaviour, and example extraction data — per-function docs with signatures, behaviour, and examples.
origin: ECC
---

# API Reference Generation

Generation skill for producing structured API reference documentation. Combines symbol extraction (signatures), behaviour extraction (side effects, errors), and example extraction (usage snippets) into comprehensive per-function/class/type documentation.

## When to Activate

- During doc-generator pipeline when API surface documentation is needed
- When generating per-module API reference files
- When enriching doc comments with complete documentation
- When building SDK or library documentation

## Methodology

### 1. Input Assembly

Gather data from extraction skills:

| Source | Data |
|--------|------|
| symbol-extraction | Names, signatures, types, visibility, deprecation status |
| behaviour-extraction | Side effects, error paths, protocols, concurrency notes |
| example-extraction | Usage examples tagged by complexity |

### 2. Per-Symbol Documentation

For each public symbol, generate documentation in this structure:

**Functions/Methods:**
```markdown
### `functionName(param1, param2, options?)`

One-sentence description of what the function does and why you'd use it.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| param1 | `string` | yes | What this parameter controls |
| param2 | `number` | yes | What this parameter controls |
| options | `Options` | no | Configuration object (see below) |

**Returns:** `ResultType` — description of what's returned

**Throws:**
- `ValidationError` — when param1 is empty
- `NotFoundError` — when the resource doesn't exist

**Side Effects:**
- Writes to filesystem at `dest` path
- Logs to console when `options.verbose` is true

**Example:**
\```typescript
const result = functionName('input', 42);
\```
```

**Classes:**
```markdown
### `ClassName`

One-sentence description.

**Constructor:** `new ClassName(config)`

| Param | Type | Description |
|-------|------|-------------|
| config | `Config` | Configuration object |

**Methods:**
- [`methodName()`](#classname-methodname) — brief description
- [`otherMethod()`](#classname-othermethod) — brief description

**Properties:**
| Name | Type | Access | Description |
|------|------|--------|-------------|
| status | `Status` | readonly | Current status |
```

**Types/Interfaces:**
```markdown
### `TypeName`

One-sentence description of what this type represents.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| field1 | `string` | yes | What this field means |
| field2 | `number` | no | Default: 0. What this controls |
```

### 3. Quality Rules

Apply these rules to every generated doc entry:

- **No tautologies**: "Gets the name" for `getName()` is useless. Describe *what* the name represents and when you'd need it.
- **Behaviour over implementation**: Document what it does, not how the code works internally.
- **Show constraints**: Required fields, valid ranges, format requirements.
- **Link related**: Cross-reference related functions, types, and concepts.
- **Deprecation notes**: If deprecated, state what to use instead and when removal is planned.

### 4. Output Structure

**Small codebase** (single file):
- `docs/API-SURFACE.md` — all symbols in one file, grouped by module

**Large codebase** (folder):
- `docs/api-surface/INDEX.md` — table of contents with module links
- `docs/api-surface/<module>.md` — per-module API reference

### 5. Cross-Linking

Every API reference entry links to:
- Source file and line number
- Module summary (if exists)
- Related types (parameter types, return types)
- Glossary terms (if term appears in glossary)

## Anti-Patterns

See `skills/api-reference-gen/references/bad-examples.md` for documentation anti-patterns to avoid.

## Related

- Anti-patterns reference: `skills/api-reference-gen/references/bad-examples.md`
- Symbol extraction: `skills/symbol-extraction/SKILL.md`
- Behaviour extraction: `skills/behaviour-extraction/SKILL.md`
- Example extraction: `skills/example-extraction/SKILL.md`
- Doc generator agent: `agents/doc-generator.md`
