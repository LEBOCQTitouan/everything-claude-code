---
name: lenses
category: functional
tags: [functional, optics, lenses, nested-data]
languages: [kotlin, typescript]
difficulty: advanced
---

## Intent

Provide composable, immutable accessors for reading and updating deeply nested data structures without manual destructuring.

## Problem

Updating a field 3 levels deep in an immutable data structure requires reconstructing every intermediate layer. This produces verbose, error-prone code that obscures the intent.

## Solution

Use lens combinators that compose getter/setter pairs. Each lens focuses on one level of nesting. Composed lenses focus through multiple levels. The update operation returns a new structure with only the targeted field changed.

## Language Implementations

### Kotlin
```kotlin
// Using Arrow Optics
@optics data class Address(val city: String) { companion object }
@optics data class User(val address: Address) { companion object }
val updated = User.address.city.modify(user) { it.uppercase() }
```

### TypeScript
```typescript
import { pipe } from "fp-ts/function";
import * as L from "monocle-ts/Lens";
const cityLens = pipe(L.id<User>(), L.prop("address"), L.prop("city"));
const updated = cityLens.set("NYC")(user);
```

**Relevance by language:** Haskell (native), Scala (Monocle, native), Kotlin (Arrow Optics, library), TypeScript (monocle-ts, library), Rust (N/A — ownership + destructuring preferred), Go (N/A), Python (N/A).

## When to Use

- Deeply nested immutable data structures (3+ levels).
- Functional codebases already using fp-ts, Arrow, or Monocle.

## When NOT to Use

- Shallow data (1-2 levels) — spread/copy is simpler.
- Mutable-first languages (Go, Python) where in-place update is idiomatic.
- Rust — pattern matching and destructuring are preferred.

## Anti-Patterns

- Using lenses for flat structures (over-engineering).
- Mixing lenses with mutable state.

## Related Patterns

- functional/immutable-data
- functional/monads

## References

- monocle-ts (TypeScript). Arrow Optics (Kotlin). lens library (Haskell).
