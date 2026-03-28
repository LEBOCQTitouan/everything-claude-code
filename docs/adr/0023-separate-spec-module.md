# 0023. Separate Spec Domain Module

Date: 2026-03-27

## Status

Accepted

## Context

The `ecc-domain` crate contains a `config/` module that handles validation of ECC configuration content: agents, commands, hooks, skills, and rules. This module was the natural home for validation logic when the domain only needed to understand ECC's own configuration artifacts.

`ecc validate spec` and `ecc validate design` introduce a new category of artifact: pipeline artifacts produced by the spec-driven development workflow (`/spec` → `/design` → `/implement`). These artifacts have a different structure, different validation rules, and a different lifecycle from ECC configuration files.

Two placement options were considered:

1. **Under `config/`**: Add spec/design parsing as a sub-module of `ecc-domain/src/config/validate.rs`, alongside existing configuration validators.
2. **Separate `spec/` module**: Create `ecc-domain/src/spec/` as a new top-level module, peer to `config/`.

## Decision

Create `ecc-domain/src/spec/` as a separate top-level module within `ecc-domain`, not under `config/`.

The `spec/` module owns:
- AC definition parsing and sequential numbering validation
- PC table parsing, column count validation, and ID sequencing
- AC-to-PC coverage mapping
- File-overlap dependency ordering

## Consequences

**Positive:**

- Clear bounded context separation: configuration validation and pipeline artifact validation are independent concerns with no shared types or logic
- The `spec/` module can evolve independently — new validation rules (e.g., PC dependency order, AC coverage) are added without touching `config/`
- Follows the Common Closure Principle (CCP): configuration changes (new agent frontmatter fields) do not affect spec parsing and vice versa
- Discoverability: engineers looking for spec validation logic have an unambiguous location (`src/spec/`)

**Negative:**

- Slightly more `mod` declarations in `ecc-domain/src/lib.rs`
- A new engineer may initially look in `config/` for all validation logic before discovering the `spec/` module
