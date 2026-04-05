---
name: secrets-management
category: security
tags: [security, secrets, vault, environment, owasp]
languages: [all]
difficulty: intermediate
---

## Intent

Store, distribute, and rotate secrets (API keys, database credentials, signing keys) outside of source code, ensuring they are accessible only to authorized services at runtime.

## Problem

Hardcoded secrets in source code end up in version control, CI logs, error reports, and container images. A single leaked credential can compromise entire systems. Manual secret distribution via shared documents or chat is unauditable and unreliable.

## Solution

Use a secrets manager (HashiCorp Vault, AWS Secrets Manager, cloud-native KMS) or environment variables as the runtime source of truth. Never commit secrets to version control. Validate that required secrets are present at startup. Rotate secrets on a schedule and immediately after suspected exposure.

## Language Implementations

### Environment Variables (Baseline)

```bash
# .env.example (committed — documents required vars, no values)
DATABASE_URL=
STRIPE_SECRET_KEY=
JWT_SIGNING_KEY=

# .env (gitignored — local development only)
DATABASE_URL=postgres://user:pass@localhost:5432/mydb
STRIPE_SECRET_KEY=sk_test_abc123
JWT_SIGNING_KEY=your-256-bit-secret
```

### Startup Validation (Pseudocode)

```
fn validate_secrets():
    required = ["DATABASE_URL", "STRIPE_SECRET_KEY", "JWT_SIGNING_KEY"]
    missing = [key for key in required if env.get(key) is None]
    if missing:
        panic(f"Missing required secrets: {missing}")
    # Never log secret values
    log.info(f"All {len(required)} required secrets present")
```

### Git Pre-Commit Hook (Detection)

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/gitleaks/gitleaks
    rev: v8.18.0
    hooks:
      - id: gitleaks

# Patterns to detect
# API keys:   [A-Za-z0-9]{32,}
# AWS keys:   AKIA[0-9A-Z]{16}
# Private keys: -----BEGIN (RSA|EC|OPENSSH) PRIVATE KEY-----
```

### Secret Rotation Lifecycle

```yaml
rotation_policy:
  api_keys:
    max_age: 90d
    overlap_period: 24h  # both old and new keys valid during transition
    steps:
      1: generate_new_key
      2: deploy_new_key_to_consumers
      3: verify_consumers_using_new_key
      4: revoke_old_key

  database_credentials:
    max_age: 30d
    strategy: dual_account  # rotate between two DB accounts
```

### .gitignore Essentials

```
# Secrets — never commit
.env
.env.local
.env.*.local
*.pem
*.key
credentials.json
service-account.json
```

## When to Use

- Every project, without exception. There is no valid use case for hardcoded secrets.
- CI/CD pipelines that need deployment credentials.
- Multi-environment deployments (dev, staging, production) with different credentials per environment.

## When NOT to Use

- There is no scenario where secrets management should be skipped.

## Anti-Patterns

- Committing `.env` files to version control — even "test" secrets can be reused in production.
- Logging secrets in error messages, debug output, or HTTP responses.
- Using a single secret across all environments (dev, staging, production).
- Sharing secrets via Slack, email, or shared documents.
- Storing secrets in Docker images or build artifacts.

## Related Patterns

- [security:authn-authz](authn-authz.md) — secrets are the foundation of authentication systems.
- [security:input-validation](input-validation.md) — validate environment variables at startup.
- [observability:structured-logging](../observability/structured-logging.md) — ensure log redaction of sensitive values.

## References

- OWASP A07:2021 Security Misconfiguration: https://owasp.org/Top10/A07_2021-Security_Misconfiguration/
- OWASP Secrets Management Cheat Sheet: https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html
- HashiCorp Vault: https://www.vaultproject.io/
- Gitleaks: https://github.com/gitleaks/gitleaks
