---
name: functor-applicative
category: functional
tags: [functional, functor, applicative, type-class]
languages: [rust, kotlin, typescript]
difficulty: advanced
---

## Intent

Lift ordinary functions over wrapped values (Option, Result, List) without unwrapping, enabling composition of effectful computations.

## Problem

When values are wrapped in containers (Option, Result, Future), applying a plain function requires manual unwrapping, error checking, and re-wrapping at every step.

## Solution

Functor provides `map` — apply a function to the value inside a container. Applicative provides `ap` — apply a wrapped function to a wrapped value. Together they enable point-free composition of multi-argument functions over wrapped values.

## Language Implementations

### Rust
```rust
// Functor: Option::map, Result::map (built-in)
let x: Option<i32> = Some(5);
let y = x.map(|n| n * 2); // Some(10)
// Applicative: combine via zip + map
let result = x.zip(Some(3)).map(|(a, b)| a + b); // Some(8)
```

### Kotlin
```kotlin
// Arrow provides Functor/Applicative via extension functions
val result = either {
    val a = getUser(id).bind()
    val b = getOrder(orderId).bind()
    combine(a, b)
}
```

### TypeScript
```typescript
import { pipe } from "fp-ts/function";
import * as O from "fp-ts/Option";
const result = pipe(O.some(5), O.map(n => n * 2)); // Some(10)
```

**Relevance:** Haskell (native type classes), Scala (Cats, native), Rust (map/and_then built-in, no formal type class), Kotlin (Arrow, library), TypeScript (fp-ts, library), Go (N/A), Python (N/A).

## When to Use

- Composing multiple fallible or optional computations.
- FP-heavy codebases using fp-ts, Arrow, or Cats.

## When NOT to Use

- Imperative codebases where try/catch or if-else is clearer.
- Languages without higher-kinded types (Go, Python).

## Anti-Patterns

- Forcing category theory abstractions in non-FP codebases.
- Nesting map calls instead of using flatMap/and_then.

## Related Patterns

- functional/monads
- functional/map-filter-reduce

## References

- fp-ts documentation. Arrow library (Kotlin). Typeclassopedia (Haskell wiki).
