---
name: ascii-doc-diagrams
description: Convention for supplementing Rust doc-comments with ASCII diagrams and design pattern references. Defines diagram types, eligibility criteria, and annotation format.
origin: ECC
---

# ASCII Doc-Comment Diagrams

Convention for adding ASCII diagrams and pattern references to doc-comments. Applies to any Rust codebase.

## When to Activate

- Code review of new or changed functions/structs
- Audit of code quality (`/audit-code`)
- User says "add diagrams" or "document patterns"

## Diagram Types

Use fenced `text` code blocks inside `///` doc-comments. 80-column max.

1. **State transition diagram** — for enums with 3+ variants used as state machines. Show states as `[State]` and transitions as `-->` arrows.

2. **Flow/decision diagram** — for public functions with 3+ decision branches. Show conditions as `[condition?]` with `--Y-->` / `--N-->` paths.

3. **Composition/box diagram** — for public structs composing 3+ domain types. Show containment with `+--+` boxes and `|` borders.

Domain type = type defined in current crate or project domain crates. Excludes std, third-party, and primitives.

## Pattern Annotations

Add a `# Pattern` section to doc-comments on eligible items:

```text
/// # Pattern
///
/// Repository [DDD] / Port [Hexagonal Architecture]
```

Sources: GoF, DDD, Hexagonal Architecture, Rust Idiom. Multiple sources allowed.

## Eligibility

A diagram or pattern annotation is required when ANY of these hold:

- Public function with 3+ decision branches
- Enum with 3+ variants used as a state machine
- Public struct composing 3+ domain types
- Item referenced in ARCHITECTURE.md or onboarding docs
- Item with 5+ callers across the codebase

Items not matching any criterion: no annotation required. Macro-generated output excluded — convention applies to source-level items only.

## Style Rules

- Fenced `text` blocks (triple backtick + text language hint)
- 80-column max width
- Box-drawing: `+--+`, `|`, `-->` arrows only
- Diagrams under 20 lines to avoid file bloat
- Empty sections use "None identified" rather than being omitted
