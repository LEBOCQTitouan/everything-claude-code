---
id: BL-136
title: "Add cargo-vet for SLSA Level 2 supply chain compliance"
scope: MEDIUM
target: "/spec-dev"
status: implemented
created: "2026-04-09"
source: "docs/audits/web-radar-2026-04-09.md"
ring: assess
tags: [security, supply-chain, slsa]
---

## Context

cargo-vet (Mozilla) requires audits of third-party crates against safe-to-run/safe-to-deploy criteria. ECC already uses cosign for artifact signing — cargo-vet is the remaining gap for SLSA Level 2.

## Prompt

Evaluate and integrate cargo-vet into the ECC build pipeline. Set up `supply-chain/` directory with audit criteria. Import community audit sets (e.g., Mozilla's). Define audit policy for new dependencies. Add CI check that fails on unaudited crates. Assess effort vs benefit for a 30-dependency workspace.

## Acceptance Criteria

- [ ] cargo-vet initialized with audit criteria
- [ ] Community audit sets imported
- [ ] CI job validates all deps are audited
- [ ] New dep workflow documented
