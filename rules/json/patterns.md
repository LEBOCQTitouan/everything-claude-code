---
paths:
  - "**/*.json"
  - "**/*.jsonc"
applies-to: { files: ["*.json"] }
---
# JSON Patterns

> This file extends [common/patterns.md](../common/patterns.md) with JSON specific content.

## JSON Schema

Define schemas for validation:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["name", "version"],
  "properties": {
    "name": { "type": "string", "minLength": 1 },
    "version": { "type": "string", "pattern": "^\\d+\\.\\d+\\.\\d+$" }
  },
  "additionalProperties": false
}
```

## Configuration Files

- Use JSON Schema `$schema` for IDE validation:
  ```json
  { "$schema": "https://json.schemastore.org/tsconfig" }
  ```
- Use JSONC (JSON with Comments) for human-edited configs
- Use `.json` for machine-generated/consumed data

## API Responses

Follow consistent envelope format:

```json
{
  "data": { "id": "123", "name": "Example" },
  "meta": { "page": 1, "total": 42 },
  "errors": null
}
```

## Reference

See skill: `json-patterns` for comprehensive JSON Schema and tooling patterns.
