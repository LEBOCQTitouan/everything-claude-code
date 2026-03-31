# ADR 0038: Mandatory Keyless Sigstore Cosign Verification

## Status

Accepted (2026-03-31)

## Context

`ecc update` downloads pre-built binaries from GitHub Releases and replaces the running binary. This is a high-risk supply chain surface — a compromised release or man-in-the-middle attack could inject malicious code into every developer's machine.

Three verification strategies were evaluated:

- **Checksum-only (SHA-256)**: Verifies integrity but not provenance. If an attacker compromises the GitHub Release, they can update both the tarball and its checksum. No protection against supply chain attacks.
- **GPG signing**: Requires managing a long-lived signing key, distributing the public key, and handling key rotation. Key compromise is catastrophic and silent.
- **Keyless Sigstore cosign**: Ephemeral keys tied to GitHub Actions OIDC identity. No long-lived secrets to manage or rotate. Verification checks that the artifact was built by a specific GitHub Actions workflow in a specific repository. Transparency log (Rekor) provides public auditability.

The GitHub Actions release workflow already has OIDC token access. Cosign is available as a GitHub Action (`sigstore/cosign-installer`) and as a standalone binary for client-side verification.

## Decision

Mandate keyless Sigstore cosign verification for all `ecc update` downloads. SHA-256 checksums are retained as a secondary integrity check. The `--skip-verify` escape hatch is intentionally omitted.

Verification checks:
1. SHA-256 checksum of the downloaded tarball matches the published checksum file
2. Cosign bundle signature is valid against the Sigstore transparency log
3. OIDC identity matches the expected GitHub Actions workflow and repository

## Consequences

- Every release artifact has cryptographic provenance tied to the CI workflow — not a person or long-lived key
- No secret key management burden — ephemeral keys are generated and discarded per signing event
- Compromised GitHub Release metadata alone is insufficient for attack — the transparency log is immutable
- Users cannot bypass verification — this is intentional to prevent supply chain compromise via social engineering
- Requires `cosign` binary on the client machine for signature verification, or a Rust-native Sigstore client
- If Sigstore infrastructure is unavailable, `ecc update` fails rather than proceeding unverified
- Key rotation is a non-issue — there are no long-lived keys to rotate
