---
paths:
  - "**/*.json"
  - "**/*.jsonc"
applies-to: { files: ["*.json"] }
---
# JSON Coding Style

> This file extends [common/coding-style.md](../common/coding-style.md) with JSON specific content.

## Formatting

- Use 2-space indentation (consistent with most ecosystem defaults)
- Use **prettier** or **jq** for formatting
- Sort keys alphabetically in configuration files

## Structure

- Use descriptive key names in `camelCase` or `snake_case` (be consistent)
- Avoid deeply nested objects (max 4-5 levels)
- Use arrays for ordered collections, objects for named properties

## Validation

- Define JSON Schema for all configuration files
- Validate JSON in CI pipelines
- Use `jsonlint` or `jq` for syntax validation:
  ```bash
  jq empty file.json
  ```

## Reference

See skill: `json-patterns` for comprehensive JSON patterns and tooling.
