---
name: adts
category: functional
tags: [functional, algebraic-data-types, sum-type, product-type]
languages: [rust, python, typescript, haskell, scala]
difficulty: intermediate
---

## Intent

Model data precisely using sum types (one-of) and product types (all-of), making illegal states unrepresentable in the type system.

## Problem

Using strings, integers, or boolean flags to represent variants leads to invalid combinations that compile but crash at runtime. A status field that is a string can hold any value, not just the valid ones. You need types that constrain values to exactly the legal states.

## Solution

Use sum types (tagged unions, enums with data) to represent "one of N variants" and product types (structs, tuples) to represent "all of these fields together." The compiler enforces exhaustive handling of all variants.

## Language Implementations

**Relevance**: Rust (native), Python (library — dataclasses + Literal), Typescript (native — discriminated unions), Haskell (native), Scala (native — sealed traits)

### Rust

```rust
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}

fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
        Shape::Rectangle { width, height } => width * height,
        Shape::Triangle { base, height } => 0.5 * base * height,
    }
}
```

### Haskell

```haskell
data Shape
  = Circle Double
  | Rectangle Double Double
  | Triangle Double Double

area :: Shape -> Double
area (Circle r)      = pi * r * r
area (Rectangle w h) = w * h
area (Triangle b h)  = 0.5 * b * h
```

### Typescript

```typescript
type Shape =
  | { readonly kind: "circle"; readonly radius: number }
  | { readonly kind: "rectangle"; readonly width: number; readonly height: number }
  | { readonly kind: "triangle"; readonly base: number; readonly height: number };

function area(shape: Shape): number {
  switch (shape.kind) {
    case "circle": return Math.PI * shape.radius ** 2;
    case "rectangle": return shape.width * shape.height;
    case "triangle": return 0.5 * shape.base * shape.height;
  }
}
```

### Scala

```scala
sealed trait Shape
case class Circle(radius: Double) extends Shape
case class Rectangle(width: Double, height: Double) extends Shape
case class Triangle(base: Double, height: Double) extends Shape

def area(shape: Shape): Double = shape match {
  case Circle(r)      => math.Pi * r * r
  case Rectangle(w, h) => w * h
  case Triangle(b, h)  => 0.5 * b * h
}
```

## When to Use

- When a value can be exactly one of several variants.
- When illegal state combinations must be prevented at compile time.
- When exhaustive pattern matching is valuable for correctness.

## When NOT to Use

- When variants are open-ended and frequently extended (use interfaces/traits instead).
- When the language lacks sum type support and workarounds are too complex.
- When a simple enum without data suffices.

## Anti-Patterns

- Using strings or integers where a sum type would prevent invalid values.
- Non-exhaustive matching — ignoring variants with a catch-all default.
- Deeply nested ADTs that are hard to construct and destructure.

## Related Patterns

- [functional/pattern-matching](pattern-matching.md) — destructure ADTs at use sites.
- [functional/monads](monads.md) — Option and Result are ADTs with monadic operations.
- [functional/immutable-data](immutable-data.md) — ADTs are naturally immutable.

## References

- Types and Programming Languages, Benjamin C. Pierce, Chapter 11.
- Rust Book — Enums: https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html
- Haskell Wiki — Algebraic data types: https://wiki.haskell.org/Algebraic_data_type
