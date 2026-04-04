---
name: clean-architecture
category: architecture
tags: [architecture, clean-code, dependency-rule, layers]
languages: [all]
difficulty: advanced
---

## Intent

Organise code into concentric layers where dependencies point exclusively inward. Business rules at the centre are isolated from UI, databases, and frameworks at the outer rings.

## Problem

Applications accumulate coupling between business logic and delivery mechanisms. A change in the web framework forces changes in domain code. Database schema changes ripple into use cases. Testing business rules requires booting the full stack.

## Solution

Arrange the system into four concentric rings: Entities (innermost), Use Cases, Interface Adapters, and Frameworks/Drivers (outermost). The Dependency Rule states: source code dependencies can only point inward. Nothing in an inner ring knows anything about an outer ring.

```
┌──────────────────────────────────────────────┐
│  Frameworks & Drivers (UI, DB, Web, Devices) │
│  ┌──────────────────────────────────────┐    │
│  │  Interface Adapters (Controllers,    │    │
│  │  Presenters, Gateways)               │    │
│  │  ┌────────────────────────────┐      │    │
│  │  │  Use Cases (Application    │      │    │
│  │  │  Business Rules)           │      │    │
│  │  │  ┌──────────────────┐      │      │    │
│  │  │  │  Entities        │      │      │    │
│  │  │  │  (Enterprise     │      │      │    │
│  │  │  │  Business Rules) │      │      │    │
│  │  │  └──────────────────┘      │      │    │
│  │  └────────────────────────────┘      │    │
│  └──────────────────────────────────────┘    │
└──────────────────────────────────────────────┘
```

## Language Implementations

### all

Practical layer mapping:

| Clean Architecture Ring | ECC / Rust Mapping |
|------------------------|-------------------|
| Entities | `ecc-domain`: aggregates, value objects, domain events |
| Use Cases | `ecc-app`: use case orchestrators, command/query handlers |
| Interface Adapters | `ecc-infra`: repository implementations, serialisers |
| Frameworks & Drivers | `ecc-cli`: argument parsing, binary entry point; external crates |

Enforcing the dependency rule in a Cargo workspace: list only inward-pointing crates in `[dependencies]`. CI catches violations via `cargo check`.

## When to Use

- When the business domain is non-trivial and must evolve independently of delivery.
- When multiple delivery mechanisms (REST, CLI, background workers) share the same use cases.
- When the team values long-term maintainability over initial velocity.

## When NOT to Use

- For simple scripts or prototypes where the cost of layering outweighs the benefit.
- When business logic is minimal and all code is essentially I/O.

## Anti-Patterns

- Importing database models directly into use cases — violates the dependency rule.
- Putting presentation logic (HTML generation, JSON serialisation) in entities.
- Allowing use cases to depend on HTTP request/response types.

## Related Patterns

- [hexagonal](hexagonal.md) — equivalent concept with different vocabulary (ports vs. rings).
- [cqrs](cqrs.md) — complements clean architecture by separating the read and write models.

## References

- Robert C. Martin — Clean Architecture: A Craftsman's Guide to Software Structure and Design.
- The Clean Architecture blog post: https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html
