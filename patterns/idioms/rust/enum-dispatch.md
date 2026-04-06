---
name: enum-dispatch
category: idioms
tags: [idiom, rust]
languages: [rust]
difficulty: intermediate
---

## Intent

Replace dynamic dispatch (trait objects) with a flat enum whose variants hold concrete types, using `match` to dispatch method calls with zero heap allocation and full exhaustiveness checking.

## Problem

Trait objects (`Box<dyn Trait>`) incur heap allocation and vtable indirection. When the set of implementations is closed and known at compile time, dynamic dispatch adds unnecessary overhead and prevents the compiler from inlining or optimizing across variants.

## Solution

Define an enum with one variant per concrete type. Implement the shared interface as a method on the enum that matches on `self` and delegates to the inner type. The compiler can inline each arm, and adding a new variant triggers exhaustive-match errors at every call site.

## Language Implementations

### Rust

```rust
enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
    Triangle(Triangle),
}

struct Circle { radius: f64 }
struct Rectangle { width: f64, height: f64 }
struct Triangle { base: f64, height: f64 }

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle(c) => std::f64::consts::PI * c.radius * c.radius,
            Shape::Rectangle(r) => r.width * r.height,
            Shape::Triangle(t) => 0.5 * t.base * t.height,
        }
    }
}

// No heap allocation, no vtable, exhaustive matching
let shapes = vec![
    Shape::Circle(Circle { radius: 3.0 }),
    Shape::Rectangle(Rectangle { width: 4.0, height: 5.0 }),
];
let total: f64 = shapes.iter().map(|s| s.area()).sum();
```

**ECC codebase usage:** `ecc-domain` uses enum dispatch extensively for command types and validation targets. The `ValidateTarget` enum dispatches to concrete validators without trait objects, ensuring compile-time exhaustiveness when new targets are added.

## When to Use

- When the set of variants is closed and known at compile time.
- When performance matters and heap allocation or vtable indirection is undesirable.
- When you want exhaustive match guarantees on every dispatch point.

## When NOT to Use

- When the set of implementations is open-ended or defined by downstream crates (use trait objects).
- When variants have vastly different sizes, causing enum bloat.
- When you need plugin-style extensibility.

## Anti-Patterns

- Adding a catch-all `_ =>` arm that silently ignores new variants.
- Duplicating logic across match arms instead of extracting shared behavior.
- Using enum dispatch when there is only one variant (just use the concrete type).

## Related Patterns

- [typestate](typestate.md) -- compile-time state encoding; enum dispatch is its runtime counterpart.
- [strategy](../../behavioral/strategy.md) -- trait-object-based variant when extensibility is needed.
- [visitor](../../behavioral/visitor.md) -- the OOP pattern that enum dispatch replaces in Rust.

## References

- enum_dispatch crate: https://docs.rs/enum_dispatch
- Rust Performance Book -- Enum dispatch: https://nnethercote.github.io/perf-book/type-sizes.html
