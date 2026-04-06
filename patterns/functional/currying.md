---
name: currying
category: functional
tags: [functional, currying, partial-application, composition]
languages: [rust, python, typescript]
difficulty: intermediate
---

## Intent

Transform a function that takes multiple arguments into a sequence of functions each taking a single argument, enabling partial application and flexible function composition.

## Problem

Functions with many parameters are hard to reuse in different contexts. You want to fix some arguments now and supply the rest later. Without currying or partial application, you must create wrapper functions manually for every combination.

## Solution

Curry a multi-argument function into a chain of single-argument functions. Applying fewer arguments than the full arity returns a new function awaiting the remaining arguments. This enables creating specialized versions from general functions.

## Language Implementations

**Relevance**: Rust (library — closures), Python (library — functools.partial), Typescript (library — manual), Haskell (native — all functions curried), Scala (native — multiple parameter lists)



### Python

```python
from functools import partial

def add(x: int, y: int) -> int:
    return x + y

increment = partial(add, 1)

def format_log(level: str, message: str) -> str:
    return f"[{level}] {message}"

warn = partial(format_log, "WARN")
```

### Typescript

```typescript
const add = (x: number) => (y: number): number => x + y;
const increment = add(1);

const formatLog = (level: string) => (message: string): string =>
  `[${level}] ${message}`;

const warn = formatLog("WARN");
```

### Rust

```rust
fn add(x: i32) -> impl Fn(i32) -> i32 {
    move |y| x + y
}

fn format_log(level: &str) -> impl Fn(&str) -> String + '_ {
    move |message| format!("[{level}] {message}")
}

// let increment = add(1);
// let warn = format_log("WARN");
```

## When to Use

- When building specialized functions from general ones.
- When composing functions in a pipeline where each stage expects one argument.
- When configuring behavior at initialization and applying it repeatedly.

## When NOT to Use

- When the language does not support it idiomatically and the syntax is awkward.
- When function arguments are not naturally ordered from general to specific.
- When clarity suffers — named parameters may be more readable.

## Anti-Patterns

- Currying functions with unrelated parameters that do not benefit from partial application.
- Deep currying chains that obscure what the function does.
- Using currying where a simple closure or lambda is clearer.

## Related Patterns

- [functional/map-filter-reduce](map-filter-reduce.md) — curried functions compose naturally with map/filter.
- [functional/functor-applicative](functor-applicative.md) — applicative applies curried functions to wrapped values.
- [functional/lenses](lenses.md) — lens combinators use currying for composition.

## References

- Haskell Wiki — Currying: https://wiki.haskell.org/Currying
- Python functools.partial: https://docs.python.org/3/library/functools.html#functools.partial
- Scala — Multiple Parameter Lists: https://docs.scala-lang.org/tour/multiple-parameter-lists.html
