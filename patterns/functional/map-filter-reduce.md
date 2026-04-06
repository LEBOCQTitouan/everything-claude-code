---
name: map-filter-reduce
category: functional
tags: [functional, collections, transformation, pipeline]
languages: [rust, python, typescript, haskell, scala]
difficulty: beginner
---

## Intent

Transform collections through composable operations — map (transform each element), filter (select elements), and reduce (aggregate to a single value) — without mutating the original data.

## Problem

Imperative loops mix iteration mechanics with business logic, making code harder to read, test, and parallelize. Mutable accumulators introduce bugs. You need a declarative way to express collection transformations.

## Solution

Chain pure functions: `map` applies a transformation to each element, `filter` selects elements matching a predicate, and `reduce` (fold) aggregates elements into a single result. Each operation returns a new collection, preserving immutability.

## Language Implementations

**Relevance**: Rust (native), Python (native), Typescript (native), Haskell (native), Scala (native)

### Rust

```rust
let orders: Vec<Order> = get_orders();
let total_revenue: f64 = orders.iter()
    .filter(|o| o.status == Status::Completed)
    .map(|o| o.amount)
    .sum();
```

### Haskell

```haskell
totalRevenue :: [Order] -> Double
totalRevenue = sum . map amount . filter ((== Completed) . status)
```

### Python

```python
total_revenue = sum(
    o.amount for o in orders if o.status == Status.COMPLETED
)
# Or explicitly:
# total = reduce(lambda acc, o: acc + o.amount, filter(lambda o: o.status == Status.COMPLETED, orders), 0)
```

### Typescript

```typescript
const totalRevenue = orders
  .filter(o => o.status === "completed")
  .map(o => o.amount)
  .reduce((sum, amount) => sum + amount, 0);
```

## When to Use

- When transforming collections with pure, composable operations.
- When readability matters more than micro-optimization.
- When operations can be parallelized (map and filter are embarrassingly parallel).

## When NOT to Use

- When performance requires single-pass processing (use iterators/lazy evaluation instead).
- When side effects must occur during iteration (use explicit loops).
- When the transformation logic is inherently stateful.

## Anti-Patterns

- Chaining map/filter/reduce when a single loop would be clearer and faster.
- Using reduce for complex logic that would be more readable as a loop.
- Creating intermediate collections unnecessarily — prefer lazy iterators.

## Related Patterns

- [functional/immutable-data](immutable-data.md) — map/filter/reduce preserve immutability.
- [functional/functor-applicative](functor-applicative.md) — map generalizes to functors.
- [functional/monads](monads.md) — flatMap extends map with context-dependent transformations.

## References

- Hutton, G. — Programming in Haskell, Chapter 7.
- MDN — Array.prototype.reduce(): https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/reduce
- Rust std::iter: https://doc.rust-lang.org/std/iter/
