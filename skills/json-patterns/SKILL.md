---
name: json-patterns
description: JSON best practices including JSON Schema design, validation tools, package.json conventions, and API response formatting patterns.
origin: ECC
---

# JSON Patterns

Best practices for working with JSON in configuration, APIs, and data exchange.

## When to Activate

- Designing JSON APIs or response formats
- Writing JSON Schema for validation
- Configuring JSON-based tools (tsconfig, eslint, package.json)
- Debugging JSON parsing issues

## JSON Schema

### Basic Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://example.com/config.schema.json",
  "title": "Application Config",
  "type": "object",
  "required": ["database", "server"],
  "properties": {
    "database": {
      "type": "object",
      "required": ["host", "port"],
      "properties": {
        "host": { "type": "string" },
        "port": { "type": "integer", "minimum": 1, "maximum": 65535 },
        "name": { "type": "string", "default": "app" }
      }
    },
    "server": {
      "type": "object",
      "properties": {
        "port": { "type": "integer", "default": 3000 },
        "cors": {
          "type": "array",
          "items": { "type": "string", "format": "uri" }
        }
      }
    }
  }
}
```

### Reusable Definitions

```json
{
  "$defs": {
    "address": {
      "type": "object",
      "properties": {
        "street": { "type": "string" },
        "city": { "type": "string" },
        "zip": { "type": "string", "pattern": "^[0-9]{5}$" }
      }
    }
  },
  "properties": {
    "billing": { "$ref": "#/$defs/address" },
    "shipping": { "$ref": "#/$defs/address" }
  }
}
```

## Validation Tools

```bash
# jq — validate and format
jq empty file.json           # Syntax check
jq '.' file.json             # Pretty print
jq '.data[] | .name' file.json  # Query

# ajv — JSON Schema validation
npx ajv validate -s schema.json -d data.json

# check-jsonschema — Python-based
check-jsonschema --schemafile schema.json data.json
```

## package.json Conventions

```json
{
  "name": "@scope/package-name",
  "version": "1.0.0",
  "description": "Clear, one-line description",
  "type": "module",
  "main": "./dist/index.js",
  "types": "./dist/index.d.ts",
  "exports": {
    ".": {
      "types": "./dist/index.d.ts",
      "import": "./dist/index.js"
    }
  },
  "scripts": {
    "build": "tsc",
    "test": "vitest",
    "lint": "eslint .",
    "format": "prettier --write ."
  },
  "engines": {
    "node": ">=20"
  }
}
```

## API Response Conventions

### Success Response

```json
{
  "data": {
    "id": "user_123",
    "email": "user@example.com"
  }
}
```

### Error Response

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid email format",
    "details": [
      { "field": "email", "message": "Must be a valid email address" }
    ]
  }
}
```

### Paginated Response

```json
{
  "data": [],
  "meta": {
    "page": 1,
    "per_page": 20,
    "total": 142,
    "total_pages": 8
  }
}
```

## Quick Reference

| Tool | Purpose |
|------|---------|
| `jq` | JSON processor and validator |
| `ajv` | JSON Schema validator |
| `prettier` | JSON formatting |
| `jsonlint` | JSON syntax validation |
| `json-schema-to-ts` | Generate TypeScript from JSON Schema |
| `quicktype` | Generate types from JSON examples |
