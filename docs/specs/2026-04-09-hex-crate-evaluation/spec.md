# Spec: BL-138 Hex Crate Evaluation — REJECT, Add Domain Purity Test

## Problem Statement

The project uses convention-based hexagonal boundary enforcement (Cargo.toml deps, CLAUDE.md docs, single-file self-test). The web radar flagged the `hex` crate as a potential compile-time enforcement tool. Evaluation reveals: `hexser` is a boilerplate framework (not enforcement), no Rust crate exists for compile-time boundary checking, and Cargo's crate isolation already prevents the critical violations. The remaining gap — `std::fs`/`std::io` imports sneaking into domain files — is closable with a simple test.

## Research Summary

- `hexser` (v0.4.7, 3,137 downloads, 8 versions in 6 days then abandoned) provides Entity/Repository/Directive traits — boilerplate reduction, NOT boundary enforcement
- `boundary` crate (Go-focused static analysis CLI) — not applicable to Rust crate structure
- No Rust crate exists for compile-time hexagonal boundary validation
- Cargo workspace crate isolation already enforces dependency direction at compile time
- Current gap: `ecc-domain` has only 1 self-test (worktree.rs) checking for forbidden imports; other 100+ files unchecked

## Decisions Made

| # | Decision | Rationale | ADR Needed? |
|---|----------|-----------|-------------|
| 1 | REJECT hexser/hex crate | No enforcement value; boilerplate framework; abandoned | No |
| 2 | Add comprehensive domain purity test | Closes the gap in CI without new dependency | No |

## User Stories

### US-001: Domain purity test covering all files

**As a** developer, **I want** a test that scans all ecc-domain .rs files for forbidden I/O imports, **so that** architecture violations are caught in CI, not just by convention.

#### Acceptance Criteria

- AC-001.1: Test scans all .rs files in `crates/ecc-domain/src/` recursively
- AC-001.2: Test fails if any file contains `std::fs`, `std::io`, `std::process`, `std::net`, or `tokio`
- AC-001.3: Test passes on current codebase (no existing violations)
- AC-001.4: Test lives in `crates/ecc-domain/tests/` as integration test
