---
name: railway-oriented
category: error-handling
tags: [error-handling, composition, monadic, pipeline]
languages: [rust, go, typescript]
difficulty: advanced
---

## Intent

Compose a sequence of fallible operations into a pipeline where each step either continues on the "success track" or short-circuits to the "error track," eliminating nested error-checking boilerplate.

## Problem

Chaining multiple operations that can each fail produces deeply nested match/if-err blocks. You need a linear pipeline where failure at any step automatically bypasses remaining steps and propagates the error.

## Solution

Use monadic composition (bind/flatMap/and_then) or syntactic sugar (`?` in Rust, `?.` in TypeScript with neverthrow) to chain Result-returning functions. Each function receives the success value of the previous step; on failure, the chain short-circuits.

**Language matrix:**

| Language | Mechanism | Type |
|----------|-----------|------|
| Rust | `?` operator, `.and_then()`, `.map()` | Native |
| Go | Sequential `if err != nil` (no syntactic sugar) | Convention |
| TypeScript | `neverthrow` `.andThen()`, `fp-ts` `pipe` | Library |

> Python uses exceptions for control flow, which is inherently railway-oriented (try/except is the "switch"). Explicit Result chaining is possible with libraries like `returns`.

## Language Implementations

### Rust

The `?` operator provides built-in railway semantics:

```rust
use std::num::ParseIntError;

fn parse_and_double(input: &str) -> Result<i64, String> {
    let trimmed = input.trim();
    let parsed: i64 = trimmed.parse().map_err(|e: ParseIntError| e.to_string())?;
    let validated = if parsed >= 0 { Ok(parsed) } else { Err("negative".into()) }?;
    Ok(validated * 2)
}

// Each step short-circuits on Err — no nesting needed
```

Explicit chaining with combinators:

```rust
fn pipeline(input: &str) -> Result<i64, String> {
    input
        .trim()
        .parse::<i64>()
        .map_err(|e| e.to_string())
        .and_then(|n| if n >= 0 { Ok(n) } else { Err("negative".into()) })
        .map(|n| n * 2)
}
```

### Go

Go lacks monadic syntax — railway is expressed as sequential guards:

```go
func parseAndDouble(input string) (int64, error) {
    trimmed := strings.TrimSpace(input)

    parsed, err := strconv.ParseInt(trimmed, 10, 64)
    if err != nil {
        return 0, fmt.Errorf("parse: %w", err)
    }

    if parsed < 0 {
        return 0, fmt.Errorf("validation: negative value %d", parsed)
    }

    return parsed * 2, nil
}
```

### TypeScript

Using `neverthrow` for explicit railway:

```typescript
import { ok, err, Result } from "neverthrow";

const parse = (input: string): Result<number, string> => {
  const n = Number(input.trim());
  return isNaN(n) ? err("not a number") : ok(n);
};

const validate = (n: number): Result<number, string> =>
  n >= 0 ? ok(n) : err("negative");

const double = (n: number): number => n * 2;

const pipeline = (input: string): Result<number, string> =>
  parse(input).andThen(validate).map(double);
```

## When to Use

- When chaining 3+ fallible operations that each depend on the previous result.
- When nested error checks obscure the happy path.
- When you want compile-time guarantees that errors are handled.

## When NOT to Use

- For single fallible operations — a simple match/if-err suffices.
- When the language idiom is exceptions and the team expects try/catch flow.

## Anti-Patterns

- Chaining `.unwrap()` to avoid `?` — defeats the entire purpose.
- Mixing railway composition with thrown exceptions in the same pipeline.
- Overly long chains that are hard to debug — break into named functions.

## Related Patterns

- [result-either](result-either.md) — the foundation type that railway composition operates on.
- [error-wrapping](error-wrapping.md) — often used within `.map_err()` to add context at each step.
- [structured-errors](structured-errors.md) — railway steps can produce typed error variants.

## References

- Scott Wlaschin — Railway Oriented Programming: https://fsharpforfunandprofit.com/rop/
- Rust `?` operator: https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-question-mark-operator
- neverthrow: https://github.com/supermacro/neverthrow
