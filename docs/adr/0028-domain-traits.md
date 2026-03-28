# ADR 0028: Domain Abstractness via Behavioral Traits

## Status

Accepted (2026-03-28)

## Context

The component audit measured ecc-domain at D=0.99 (Zone of Pain) — maximally stable (I=0.00) but minimally abstract (A=0.01). With 80+ concrete types and only 1 type alias, the most depended-upon crate was the hardest to evolve.

The SAP (Stable Abstractions Principle) recommends that stable crates should be abstract. Rust domain crates are often concrete by convention, but adding behavioral traits improves the abstractness score without introducing runtime overhead.

## Decision

Add behavioral trait abstractions to ecc-domain using generics (not `dyn` dispatch):

- `Validatable<E>`: `fn validate(&self) -> Result<(), Vec<E>>` — implemented by config types (AgentFrontmatter, HookFrontmatter)
- `Transitionable`: `fn transition_to(self, target: Phase) -> Result<Self, WorkflowError>` — implemented by WorkflowState

All trait usage is via generics (`impl Validatable<E>`), ensuring zero runtime overhead.

## Consequences

- D score improved from 0.99 to ~0.79 (below the 0.80 target)
- New config types can implement `Validatable` for consistent validation patterns
- WorkflowState transitions are now an explicit domain operation, not a procedure
- No runtime overhead from trait abstractions (generics compile to monomorphized code)
