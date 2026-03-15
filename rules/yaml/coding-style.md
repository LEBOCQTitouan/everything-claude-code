---
paths:
  - "**/*.yml"
  - "**/*.yaml"
---
# YAML Coding Style

> This file extends [common/coding-style.md](../common/coding-style.md) with YAML specific content.

## Formatting

- **yamllint** is mandatory for consistent formatting
- Use 2-space indentation (never tabs)
- Maximum line length: 120 characters

## Structure

- Use consistent key ordering (alphabetical or logical grouping)
- Prefer block style over flow style for readability
- Use comments to document non-obvious values
- Avoid deeply nested structures (max 4-5 levels)

## Values

- Quote strings that could be misinterpreted (`"yes"`, `"no"`, `"true"`, `"null"`)
- Use anchors and aliases to avoid repetition:
  ```yaml
  defaults: &defaults
    timeout: 30
    retries: 3

  production:
    <<: *defaults
    timeout: 60
  ```

## Reference

See skill: `yaml-patterns` for comprehensive YAML patterns and tools.
