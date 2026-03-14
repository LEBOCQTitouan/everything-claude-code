---
name: example-extraction
description: Extract usage examples from test files, doc blocks, and example directories — produces ready-to-embed code snippets for documentation.
origin: ECC
---

# Example Extraction

Atomic extraction skill for finding and cleaning usage examples from existing code. Tests are the best source of real, working examples — this skill transforms test code into documentation-ready snippets.

## When to Activate

- During doc-generator's Step 5 (Extract Usage Examples)
- When generating API reference documentation
- When enriching doc comments with `@example` blocks
- When building a project's example gallery

## Methodology

### 1. Source Prioritisation

Search for examples in this order (highest quality first):

1. **Dedicated example directories**: `examples/`, `example/`, `docs/examples/`
2. **Test files**: `*.test.*`, `*.spec.*`, `*_test.*`, `test_*.*`
3. **Doc comments**: Existing `@example`, `# Examples`, `/// # Examples` blocks
4. **README code blocks**: Fenced code blocks in `README.md`
5. **Integration tests**: `tests/integration/`, `e2e/`

### 2. Test-to-Example Transformation

For each test that exercises a public function:

1. **Extract the test body** (inside `it()`, `test()`, `def test_*`, `func Test*`)
2. **Remove test scaffolding**:
   - Strip assertion calls (`assert.*`, `expect().*`, `t.Error*`)
   - Strip mock setup (`jest.mock`, `unittest.mock`, `gomock`)
   - Strip cleanup/teardown code
3. **Keep the meaningful parts**:
   - Variable declarations showing input data
   - The function call under test
   - Result variable assignment
4. **Add context**:
   - Brief comment explaining what the example demonstrates
   - Import statement for the function being called
5. **Validate**: Ensure the cleaned snippet would compile/run independently

### 3. Example Classification

Tag each extracted example:

| Tag | Meaning |
|-----|---------|
| `basic` | Simplest usage, happy path |
| `error-handling` | Shows error/exception handling |
| `advanced` | Complex usage, edge cases |
| `integration` | Multi-module interaction |
| `config` | Configuration/setup example |

### 4. Deduplication

When multiple tests exercise the same function:

1. Keep at most 3 examples per function (basic, error-handling, advanced)
2. Prefer shorter examples over longer ones
3. Prefer examples that show different aspects over redundant ones
4. If all examples are similar, keep only the most readable one

### 5. Output Formatting

Format examples for embedding in documentation:

**For doc comments** (inline):
```
@example
// Basic usage
const result = mergeDirectory('src', 'dest');

@example
// With options
const result = mergeDirectory('src', 'dest', { strategy: 'skip' });
```

**For markdown files** (standalone):
````markdown
### Examples

#### Basic Usage

```typescript
import { mergeDirectory } from './lib/merge';

const result = mergeDirectory('src', 'dest');
console.log(`Merged ${result.filesCopied} files`);
```

#### Error Handling

```typescript
try {
  mergeDirectory('nonexistent', 'dest');
} catch (error) {
  if (error.code === 'ENOENT') {
    console.error('Source directory not found');
  }
}
```
````

## Quality Checks

Before including an example:

- [ ] Does it compile/parse without errors?
- [ ] Does it demonstrate the function's purpose (not test internals)?
- [ ] Is it self-contained (no external state required)?
- [ ] Is it under 15 lines (excluding imports)?
- [ ] Does it use realistic, meaningful variable names?

## Related

- Symbol extraction: `skills/symbol-extraction/SKILL.md`
- API reference generation: `skills/api-reference-gen/SKILL.md`
- Doc generator agent: `agents/doc-generator.md`
- Doc analysis skill: `skills/doc-analysis/SKILL.md`
