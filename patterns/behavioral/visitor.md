---
name: visitor
category: behavioral
tags: [behavioral, double-dispatch, traversal, open-closed]
languages: [rust, go, python, typescript]
difficulty: advanced
---

## Intent

Represent an operation to be performed on elements of an object structure. Visitor lets you define a new operation without changing the classes of the elements on which it operates.

## Problem

You have a stable set of element types but frequently need to add new operations. Adding each operation as a method on every element type scatters unrelated logic and requires modifying all element classes.

## Solution

Define a visitor interface with a visit method per element type. Elements accept a visitor and call the appropriate visit method (double dispatch). New operations become new visitor implementations without modifying element classes.

## Language Implementations

### Rust

In idiomatic Rust, `enum` + `match` replaces the classic Visitor pattern. Because Rust enums are closed (all variants known at compile time), the compiler exhaustively checks every match arm, providing the same extensibility-for-operations benefit as Visitor without double dispatch:

```rust
enum Expr {
    Lit(f64),
    Add(Box<Expr>, Box<Expr>),
}

fn calculate(expr: &Expr) -> f64 {
    match expr {
        Expr::Lit(n) => *n,
        Expr::Add(l, r) => calculate(l) + calculate(r),
    }
}

fn pretty(expr: &Expr) -> String {
    match expr {
        Expr::Lit(n) => n.to_string(),
        Expr::Add(l, r) => format!("({} + {})", pretty(l), pretty(r)),
    }
}
```

Each new operation is a new function with a match -- no trait needed.

### Go

```go
type Visitor interface {
    VisitLit(n float64) float64
    VisitAdd(l, r float64) float64
}

type Calculator struct{}
func (Calculator) VisitLit(n float64) float64      { return n }
func (Calculator) VisitAdd(l, r float64) float64    { return l + r }
```

### Python

```python
from typing import Protocol

class Visitor(Protocol):
    def visit_lit(self, n: float) -> float: ...
    def visit_add(self, left: float, right: float) -> float: ...

class Calculator:
    def visit_lit(self, n: float) -> float: return n
    def visit_add(self, left: float, right: float) -> float: return left + right
```

### Typescript

```typescript
interface Visitor {
  visitLit(n: number): number;
  visitAdd(left: number, right: number): number;
}

class Calculator implements Visitor {
  visitLit(n: number): number { return n; }
  visitAdd(left: number, right: number): number { return left + right; }
}
```

## When to Use

- When you have a stable set of element types but frequently add new operations.
- When related operations on a structure should be grouped together rather than scattered across element classes.
- When double dispatch is needed to select behavior based on both element and operation type.

## When NOT to Use

- When element types change frequently -- each new type requires updating every visitor.
- When there are only one or two operations -- adding a method directly is simpler.
- In Rust, when an enum + match covers the use case without needing trait-based dispatch.

## Anti-Patterns

- Visitors that accumulate mutable state, making them hard to reason about.
- Using visitor when the element hierarchy is unstable, causing constant visitor updates.
- Over-engineering simple traversals that a loop or iterator handles cleanly.

## Related Patterns

- [behavioral/iterator](iterator.md) -- iterator provides sequential access; visitor performs operations during traversal.
- [behavioral/strategy](strategy.md) -- strategy varies one algorithm; visitor varies operations across a type hierarchy.
- [structural/composite](../structural/composite.md) -- visitor is commonly used to operate on composite structures.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 5.
- Refactoring.Guru -- Visitor: https://refactoring.guru/design-patterns/visitor
- Rust enum dispatch: https://doc.rust-lang.org/book/ch06-02-match.html
