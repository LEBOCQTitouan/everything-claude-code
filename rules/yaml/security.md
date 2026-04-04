---
paths:
  - "**/*.yml"
  - "**/*.yaml"
applies-to: { files: ["*.yml", "*.yaml"] }
---
# YAML Security

> This file extends [common/security.md](../common/security.md) with YAML specific content.

## Secrets

- Never commit secrets in YAML files
- Use environment variable references: `${ENV_VAR}` or templating
- Use `.gitignore` for local override files (e.g., `*.local.yml`)

## Safe Parsing

- Use safe YAML loaders (never `yaml.load()` without `Loader=SafeLoader` in Python)
- Disable arbitrary code execution in YAML parsers
- Validate YAML against schemas before processing

## CI/CD Security

- Pin action versions in GitHub Actions: `uses: actions/checkout@v4.1.0`
- Never expose secrets in workflow logs
- Use `environment` protection rules for production deployments
