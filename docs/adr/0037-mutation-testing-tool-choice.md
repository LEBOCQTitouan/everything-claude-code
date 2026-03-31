# ADR 0037: Mutation Testing Tool Choice and Crate Scoping

## Status

Accepted

## Context

Audit TEST-008 (flagged 2026-03-14, reconfirmed 2026-03-29) identified "No mutation testing configured" as a gap. ECC has 2148+ tests with 80%+ line coverage targets but no mechanism to measure test quality — whether tests actually detect behavioral changes.

Two tools were evaluated:
- **cargo-mutants** (sourcefrog): actively maintained, nextest integration, no code annotations, workspace support, reflink copy-on-write
- **mutest-rs** (llogiq): less mature, requires code annotations, no nextest integration

## Decision

1. **Use cargo-mutants** as the mutation testing tool for ECC.
2. **Scope mutation testing to ecc-domain and ecc-app only** — highest ROI crates (pure business logic + orchestration). Infrastructure/CLI adapter mutations are noisy and better covered by integration/E2E tests.

## Consequences

- cargo-mutants is installed as an external tool (`cargo install`), not a Cargo dependency
- `mutants.toml` at workspace root configures nextest integration, 120s per-mutant timeout, and crate scoping
- CI mutation job is non-blocking (`continue-on-error: true`) until scores stabilize
- No Rust `ecc mutants` CLI subcommand — mutation testing has no domain logic; xtask subcommand wraps the external binary
