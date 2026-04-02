# ADR 0043: Two-Tier Element Type System

## Status
Accepted

## Context
The element registry must work on any project (language-agnostic requirement from BL-064). However, the listed element types (Command, Agent, Skill, Hook, Rule, Crate, Port, Adapter, DomainEntity) are ECC and Rust/hexagonal-specific. A Python or TypeScript project has none of these.

## Decision
Two-tier element type system: universal base variants (Module, Interface, Config, Unknown) that work on any project, plus ECC-specific overlay variants (Command, Agent, Skill, Hook, Rule, Crate, Port, Adapter, DomainEntity). All variants live in a single `ElementType` enum. Type inference uses path prefixes for ECC types, crate roles for Rust types, and falls back to `Unknown` for unrecognized elements.

## Consequences
- Any project gets meaningful element classification via universal types
- ECC projects get rich, specific classification via the overlay
- New project types can be supported by adding variants without breaking existing code
- `Unknown` is the safe default — never fails, always classifiable
