---
name: pattern-matching
category: functional
tags: [functional, pattern-matching, destructuring, exhaustive]
languages: [rust, python, typescript, haskell, scala]
difficulty: beginner
---

## Intent

Destructure data types and dispatch logic based on their shape, with compile-time exhaustiveness checking ensuring all cases are handled.

## Problem

Chains of if-else or switch statements are verbose, error-prone, and do not guarantee that all variants are handled. Adding a new variant silently falls through to a default case. You need a concise, safe way to branch on data shape.

## Solution

Use pattern matching to destructure values and bind their components to variables in one expression. The compiler checks that all patterns are covered, and warns or errors on missing cases.

## Language Implementations

**Relevance**: Rust (native), Python (native — 3.10+ match), Typescript (N/A — use discriminated unions), Haskell (native), Scala (native)

### Rust

```rust
enum Command {
    Quit,
    Echo(String),
    Move { x: i32, y: i32 },
}

fn execute(cmd: Command) -> String {
    match cmd {
        Command::Quit => "Goodbye".to_string(),
        Command::Echo(msg) => msg,
        Command::Move { x, y } => format!("Moving to ({x}, {y})"),
    }
}
```

### Haskell

```haskell
data Command = Quit | Echo String | Move Int Int

execute :: Command -> String
execute Quit       = "Goodbye"
execute (Echo msg) = msg
execute (Move x y) = "Moving to (" ++ show x ++ ", " ++ show y ++ ")"
```

### Python

```python
from dataclasses import dataclass

@dataclass(frozen=True)
class Quit: pass

@dataclass(frozen=True)
class Echo:
    message: str

@dataclass(frozen=True)
class Move:
    x: int
    y: int

def execute(cmd) -> str:
    match cmd:
        case Quit():
            return "Goodbye"
        case Echo(message=msg):
            return msg
        case Move(x=x, y=y):
            return f"Moving to ({x}, {y})"
```

### Scala

```scala
sealed trait Command
case object Quit extends Command
case class Echo(message: String) extends Command
case class Move(x: Int, y: Int) extends Command

def execute(cmd: Command): String = cmd match {
  case Quit        => "Goodbye"
  case Echo(msg)   => msg
  case Move(x, y)  => s"Moving to ($x, $y)"
}
```

## When to Use

- When branching on the variant of a sum type / enum.
- When destructuring nested data into named components.
- When exhaustiveness guarantees are important for correctness.

## When NOT to Use

- When the language does not support pattern matching natively (older Python, JavaScript).
- When a simple if/else on a boolean is sufficient.
- When matching against open-ended types that cannot be exhaustively enumerated.

## Anti-Patterns

- Using a wildcard catch-all (`_`) that silently swallows new variants.
- Deeply nested patterns that reduce readability — extract helper functions.
- Matching on strings when an enum/ADT would be more type-safe.

## Related Patterns

- [functional/adts](adts.md) — ADTs provide the types to match against.
- [functional/monads](monads.md) — pattern match on Result/Option at boundaries.
- [functional/immutable-data](immutable-data.md) — matched data is never mutated.

## References

- Rust Book — Pattern Syntax: https://doc.rust-lang.org/book/ch18-03-pattern-syntax.html
- Python PEP 634 — Structural Pattern Matching: https://peps.python.org/pep-0634/
- Scala — Pattern Matching: https://docs.scala-lang.org/tour/pattern-matching.html
