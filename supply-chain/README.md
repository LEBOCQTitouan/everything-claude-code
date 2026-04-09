# Supply Chain Audit Policy

This directory contains [cargo-vet](https://mozilla.github.io/cargo-vet/) configuration for human-review verification of all third-party dependencies.

## How It Works

Every dependency must be certified as either `safe-to-deploy` (production deps) or `safe-to-run` (dev-only deps) before CI passes. Certifications come from:

1. **Imported audit sets** — Mozilla and Google publish audits for common crates
2. **Local audits** — Manual review for crates not covered by imports
3. **Exemptions** — Temporary pass for existing deps (to be worked down)

## Adding a New Dependency

1. **Add the dependency**: `cargo add <crate>`
2. **Check**: `cargo vet check` — see if it fails
3. **Certify**: If unvetted, review the crate and run `cargo vet certify <crate> <version>`
4. **Commit**: `git add supply-chain/ && git commit -m "chore: certify <crate> for cargo-vet"`

## Criteria

- **safe-to-deploy**: Production dependencies shipped in the binary
- **safe-to-run**: Dev dependencies used only during testing/building

## Files

- `config.toml` — Import sources, policy, and exemptions
- `audits.toml` — Local audit records
- `imports.lock` — Pinned state of imported audit sets (auto-generated, do not hand-edit)

## Imported Audit Sets

- **Mozilla**: Covers serde, regex, sha2, flate2, and many common crates
- **Google**: Covers tokio, tracing ecosystem, and chromium dependencies

## CI Integration

The `validate` job in `.github/workflows/ci.yml` runs `cargo vet check --locked` as a blocking check. No PR can land with unaudited dependencies.
