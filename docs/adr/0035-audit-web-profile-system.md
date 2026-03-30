# ADR 0035: Audit-Web Profile System

## Status

Accepted (2026-03-30)

## Context

`/audit-web` scans 8 hardcoded dimensions with no project-specific customization, no persistence between runs, and no self-improvement. Users must manually specify `--focus` each time, custom dimensions are lost, and report quality is unverifiable.

## Decision

Add a YAML-based profile system:

1. **Profile artifact** at `docs/audits/audit-web-profile.yaml` — committed to git, human-editable
2. **YAML format** with `serde-saphyr` — matches ECC's skill/agent frontmatter convention
3. **Schema versioning** (`version: 1`) for future migration safety
4. **Guided setup** (Phase 0) scans codebase characteristics and generates a suggested profile interactively
5. **Self-improvement** (Phase 5) analyzes findings for coverage gaps and suggests profile adjustments
6. **Deterministic report validation** via `ecc audit-web validate-report` — checks sections, scores, citations
7. **Query template sanitization** — allowlist: alphanumeric, spaces, hyphens, underscores, dots, slashes, `{placeholder}` tokens

Profile lives in `docs/audits/` (not `.ecc/` or `.claude/`) because it is project-scoped, version-controlled, and visible in PRs.

## Consequences

- Custom dimensions persist across audit runs — no repeated `--focus` configuration
- Profile improvement suggestions build institutional knowledge over time
- Report validation catches structural errors before commit
- Breaking profile format changes are detected via version field
- Standard 8 dimensions remain default — custom dimensions add to, not replace
