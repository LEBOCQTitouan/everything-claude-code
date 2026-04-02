# Mutation Scores Dashboard

Last updated: 2026-03-31

## Per-Crate Scores

| Crate | Mutants | Killed | Survived | Timed Out | Score | Date |
|-------|---------|--------|----------|-----------|-------|------|
| ecc-domain | — | — | — | — | — | pending |
| ecc-app | — | — | — | — | — | pending |

## Module Breakdown: ecc-domain

| Module | Mutants | Killed | Survived | Score |
|--------|---------|--------|----------|-------|
| *Run baseline to populate* | — | — | — | — |

## Module Breakdown: ecc-app

| Module | Mutants | Killed | Survived | Score |
|--------|---------|--------|----------|-------|
| *Run baseline to populate* | — | — | — | — |

## Threshold Targets (Aspirational — TBD After Baseline)

| Crate | Target | Rationale |
|-------|--------|-----------|
| ecc-domain (validation) | 100% | Pure business rules — surviving mutants indicate real gaps |
| ecc-domain (other) | 85% | Domain logic with structural complexity |
| ecc-app | 85% | Orchestration with I/O-backed dependencies |

## History

| Date | Crate | Score | Delta | Notes |
|------|-------|-------|-------|-------|
| *pending* | — | — | — | Initial baseline not yet run |

## How to Update

1. Run `cargo mutants -p <crate>` (or `cargo xtask mutants`)
2. Update the Per-Crate Scores table with results
3. Update the Module Breakdown tables
4. Add a row to History
5. Commit: `docs: update mutation scores for <crate>`
