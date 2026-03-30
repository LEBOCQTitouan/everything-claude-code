---
name: comms-redactor
description: Secret and sensitive data redaction rules for the comms pipeline. Fail-safe scanning — blocks output on detection of CRITICAL patterns.
origin: ECC
---

# Comms Redactor — Content Safety

## Scanning Rules

### CRITICAL — BLOCK output

| Pattern | Regex | Example |
|---------|-------|---------|
| OpenAI API key | `sk-[a-zA-Z0-9]{20,}` | sk-abc123... |
| GitHub PAT | `ghp_[a-zA-Z0-9]{36}` | ghp_xxxx... |
| GitHub OAuth | `gho_[a-zA-Z0-9]{36}` | gho_xxxx... |
| AWS Access Key | `AKIA[A-Z0-9]{16}` | AKIAIOSFODNN... |
| Generic secret | `(?i)(password|secret|token)\s*[:=]\s*\S+` | password=hunter2 |
| Connection string | `(?i)(postgres|mysql|redis)://[^\s]+@` | postgres://user:pass@host |
| Bearer token | `Bearer\s+[a-zA-Z0-9._-]{20,}` | Bearer eyJhbGci... |

### HIGH — Redact with [REDACTED]

| Pattern | Action |
|---------|--------|
| Internal URLs (corporate domains) | Replace with [REDACTED-URL] |
| Email addresses (internal) | Replace with [REDACTED-EMAIL] |
| Private IPs (RFC 1918) | Replace with [REDACTED-IP] |
| Absolute file paths with usernames | Replace path with [REDACTED-PATH] |

## Enforcement

1. Redaction runs as the LAST step before writing any file
2. CRITICAL patterns → HALT output, report finding to user
3. Scanner errors → HALT output (fail-safe, never pass through)
4. External tools: use `trufflehog` or `gitleaks` if on PATH; fall back to regex patterns above
