---
paths:
  - "**/*.json"
  - "**/*.jsonc"
---
# JSON Security

> This file extends [common/security.md](../common/security.md) with JSON specific content.

## Secrets

- Never commit secrets in JSON config files
- Use `.gitignore` for local config overrides
- Use environment variable interpolation at runtime

## Parsing Safety

- Use safe JSON parsers — never execute JSON data as code (e.g., avoid `eval`-based parsing)
- Set size limits on JSON input to prevent DoS
- Validate JSON structure against a schema before processing

## package.json Security

- Pin dependency versions or use lockfiles
- Run `npm audit` / `yarn audit` regularly
- Never include private tokens in `package.json` scripts
