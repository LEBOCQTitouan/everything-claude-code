---
id: BL-118
title: "Add SLSA provenance attestations to release pipeline"
scope: MEDIUM
target: "/spec-dev"
status: implemented
created: "2026-03-31"
source: "docs/audits/web-radar-2026-03-31.md"
ring: trial
tags: [security, ci, supply-chain]
---

## Context

Rust Foundation recommends cryptographic signing and SLSA provenance attestations for release artifacts. 2026 trends show escalating CI/CD supply chain attacks. The project already uses cosign signing — SLSA attestations add provenance metadata proving where and how the binary was built.

## Prompt

Add SLSA provenance attestation generation to the release pipeline (`.github/workflows/release.yml`). Use GitHub's `actions/attest-build-provenance` or equivalent. Also evaluate `cargo-auditable` for embedding dependency metadata in release binaries. This complements the existing cosign signing with build provenance.

## Acceptance Criteria

- [ ] SLSA provenance attestations generated for each release artifact
- [ ] Attestations uploaded alongside tarballs in GitHub Release
- [ ] cargo-auditable evaluated for dependency metadata embedding
- [ ] Existing cosign signing preserved
