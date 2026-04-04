---
paths:
  - "**/*.yml"
  - "**/*.yaml"
applies-to: { files: ["*.yml", "*.yaml"] }
---
# YAML Patterns

> This file extends [common/patterns.md](../common/patterns.md) with YAML specific content.

## Anchors and Aliases

```yaml
# Define reusable blocks
.defaults: &defaults
  restart: always
  logging:
    driver: json-file
    options:
      max-size: "10m"

services:
  web:
    <<: *defaults
    image: nginx
  api:
    <<: *defaults
    image: node
```

## Multi-Document Files

```yaml
# Separate documents with ---
---
apiVersion: v1
kind: ConfigMap
---
apiVersion: v1
kind: Service
```

## Schema Validation

- Use JSON Schema for YAML validation
- Define `$schema` comments for IDE support:
  ```yaml
  # yaml-language-server: $schema=https://json.schemastore.org/github-workflow
  ```

## Reference

See skill: `yaml-patterns` for comprehensive YAML tooling and CI/CD patterns.
