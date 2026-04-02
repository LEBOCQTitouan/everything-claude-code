# ADR 0040: Cosign Signing as Custom Post-Build Job

## Status
Accepted

## Context
cargo-dist does not natively support Sigstore/cosign signing. It supports GitHub Artifact Attestations and Windows code signing via SSL.com, but not the keyless cosign workflow the project previously used. The existing release pipeline had cosign signing with `continue-on-error: true` (non-blocking — releases ship even if signing fails).

## Decision
Preserve cosign signing as a custom job in the cargo-dist-generated workflow. The `cosign-sign` job runs after the `host` stage, downloads release archives, signs them with keyless cosign (Sigstore), and uploads `.sig` and `.bundle` files to the GitHub Release. The job is declared in `dist.toml` under `[plan.jobs.cosign-sign]` so it survives `cargo dist generate-ci` regeneration. `continue-on-error: true` is maintained — signing failure does not block releases.

## Consequences
- Cosign signing preserved with identical behavior to the previous pipeline
- Non-blocking policy means unsigned releases may still ship (conscious choice, documented)
- Custom job declaration in dist.toml protects against cargo-dist regeneration overwriting
- Future: if cargo-dist adds native Sigstore support, the custom job can be removed
