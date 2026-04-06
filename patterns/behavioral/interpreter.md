---
name: interpreter
category: behavioral
tags: [behavioral, grammar, dsl, parsing]
languages: [rust, go, python, typescript]
difficulty: advanced
---

## Intent

Given a language, define a representation for its grammar along with an interpreter that uses the representation to interpret sentences in the language.

## Problem

You have a recurring problem that can be expressed as sentences in a simple language (e.g., filter expressions, validation rules, math formulas). Parsing and evaluating these sentences ad hoc leads to duplicated, brittle code.

## Solution

Define an abstract syntax tree (AST) where each node represents a grammar rule. Each node implements an `interpret` method. Complex expressions are composed from simpler ones, forming a tree that is evaluated recursively.

## Language Implementations

### Rust

```rust
enum Expr {
    Lit(f64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

fn interpret(expr: &Expr) -> f64 {
    match expr {
        Expr::Lit(n) => *n,
        Expr::Add(l, r) => interpret(l) + interpret(r),
        Expr::Mul(l, r) => interpret(l) * interpret(r),
    }
}
```

### Go

```go
type Expr interface{ Interpret() float64 }

type Lit struct{ Val float64 }
func (l Lit) Interpret() float64 { return l.Val }

type Add struct{ L, R Expr }
func (a Add) Interpret() float64 { return a.L.Interpret() + a.R.Interpret() }

type Mul struct{ L, R Expr }
func (m Mul) Interpret() float64 { return m.L.Interpret() * m.R.Interpret() }
```

### Python

```python
from abc import ABC, abstractmethod

class Expr(ABC):
    @abstractmethod
    def interpret(self) -> float: ...

class Lit(Expr):
    def __init__(self, val: float) -> None: self.val = val
    def interpret(self) -> float: return self.val

class Add(Expr):
    def __init__(self, l: Expr, r: Expr) -> None: self.l, self.r = l, r
    def interpret(self) -> float: return self.l.interpret() + self.r.interpret()
```

### Typescript

```typescript
interface Expr { interpret(): number; }

class Lit implements Expr {
  constructor(private val: number) {}
  interpret(): number { return this.val; }
}

class Add implements Expr {
  constructor(private l: Expr, private r: Expr) {}
  interpret(): number { return this.l.interpret() + this.r.interpret(); }
}
```

## When to Use

- When you have a simple, well-defined grammar that changes infrequently.
- When the grammar can be represented as a tree of small, composable rules.
- When the domain experts think in terms of a small DSL (e.g., filter expressions, scoring rules).

## When NOT to Use

- When the grammar is complex or evolving -- use a **parser combinator library** instead. In Rust, reach for `nom`, `pest`, or `lalrpop`. In Python, use `lark` or `pyparsing`. In Go, use `participle`. In TypeScript, use `chevrotain` or `nearley`. These libraries handle tokenization, precedence, error recovery, and ambiguity far better than a hand-rolled interpreter.
- When performance matters -- tree-walking interpreters are slow compared to bytecode or compiled approaches.
- When the language has more than a handful of grammar rules -- the class explosion becomes unmanageable.

## Anti-Patterns

- Building a hand-rolled parser for a non-trivial grammar instead of using an established parser combinator or generator.
- AST nodes that depend on global state or context outside the expression tree.
- Mixing parsing and interpretation in the same class, making both hard to test.

## Related Patterns

- [behavioral/visitor](visitor.md) -- visitor can traverse and operate on an interpreter AST.
- [behavioral/iterator](iterator.md) -- iterator can traverse AST nodes sequentially.
- [structural/composite](../structural/composite.md) -- the AST is a composite structure.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 5.
- Refactoring.Guru -- Interpreter: https://refactoring.guru/design-patterns/interpreter
- nom (Rust parser combinator): https://docs.rs/nom
- pest (Rust PEG parser): https://pest.rs
- lalrpop (Rust LR parser): https://lalrpop.github.io/lalrpop
