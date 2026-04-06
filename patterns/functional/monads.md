---
name: monads
category: functional
tags: [functional, monad, composition, effects]
languages: [rust, python, typescript]
difficulty: advanced
---

## Intent

Chain computations that produce wrapped values (Option, Result, Future) in a composable way, handling the "wrapping" automatically so business logic stays clean.

## Problem

When functions return wrapped types (nullable, error-bearing, async), composing them requires manual unwrapping at every step. Nested if-checks for null/error create deeply indented code. You need a way to sequence these computations while the container handles the plumbing.

## Solution

A monad provides two operations: `unit` (wrap a value) and `bind`/`flatMap` (apply a function that returns a wrapped value to an already-wrapped value). This enables chaining computations where each step may fail, produce nothing, or have effects, without manual unwrapping.

## Language Implementations

**Relevance**: Rust (native — Result/Option), Python (library — returns), Typescript (library), Haskell (native), Scala (native)

### Rust

```rust
fn parse_config(path: &str) -> Result<Config, Error> {
    let content = std::fs::read_to_string(path)?;  // ? is monadic bind
    let parsed: toml::Value = content.parse()?;
    let config = Config::from_toml(parsed)?;
    Ok(config)
}

fn find_user_email(db: &Db, id: u64) -> Option<String> {
    db.find_user(id)
        .and_then(|user| user.profile)
        .and_then(|profile| profile.email)
}
```


### Python

```python
from returns.result import Result, Success, Failure

def parse_config(path: str) -> Result[Config, str]:
    return (
        read_file(path)                    # Result[str, str]
        .bind(parse_toml)                  # Result[TomlValue, str]
        .bind(config_from_toml)            # Result[Config, str]
    )
```

### Typescript

```typescript
// Using fp-ts
import { pipe } from "fp-ts/function";
import * as TE from "fp-ts/TaskEither";

const parseConfig = (path: string) =>
  pipe(
    TE.tryCatch(() => readFile(path), String),
    TE.chain(content => TE.tryCatch(() => parseToml(content), String)),
    TE.chain(parsed => TE.fromEither(configFromToml(parsed)))
  );
```

## When to Use

- When chaining operations that may fail (Result/Either), be absent (Option/Maybe), or be async (Future/Task).
- When you want to eliminate nested null/error checks.
- When composition of effectful functions is a core concern.

## When NOT to Use

- When the language lacks monadic abstractions and the library overhead is not justified.
- When simple try/catch or null-coalescing is sufficient.
- When the team is unfamiliar with the concept and readability suffers.

## Anti-Patterns

- Using monads where simple if/else would be clearer — over-abstraction.
- Mixing monadic and imperative error handling in the same codebase.
- Deeply nested bind chains — extract named functions for clarity.

## Related Patterns

- [functional/functor-applicative](functor-applicative.md) — functors provide map; monads add flatMap.
- [functional/pattern-matching](pattern-matching.md) — unwrap monadic values at boundaries.
- [functional/adts](adts.md) — Option and Result are algebraic data types.

## References

- Wadler, P. — Monads for functional programming (1995).
- Rust Book — Error Handling: https://doc.rust-lang.org/book/ch09-00-error-handling.html
- fp-ts documentation: https://gcanti.github.io/fp-ts/
