---
name: iterator
category: behavioral
tags: [behavioral, traversal, collection, lazy]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Provide a way to access the elements of an aggregate object sequentially without exposing its underlying representation.

## Problem

Collections have different internal structures (arrays, trees, graphs), but clients need a uniform way to traverse them. Exposing internals for traversal breaks encapsulation and couples clients to a specific data structure.

## Solution

Define an iterator interface with methods to advance and retrieve the current element. Each collection provides its own iterator implementation. Clients use the iterator protocol without knowing the collection's structure.

## Language Implementations

### Rust

Rust's `Iterator` trait is the idiomatic approach -- implement `next()` on your type:

```rust
struct Countdown(u32);

impl Iterator for Countdown {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.0 == 0 { None } else { self.0 -= 1; Some(self.0 + 1) }
    }
}

// Usage: Countdown(3).map(|n| n * 2).collect::<Vec<_>>()
```

### Go

```go
type Iterator[T any] interface {
    Next() (T, bool)
}

type SliceIter[T any] struct{ items []T; idx int }

func (s *SliceIter[T]) Next() (T, bool) {
    if s.idx >= len(s.items) { var zero T; return zero, false }
    v := s.items[s.idx]; s.idx++; return v, true
}
```

### Python

```python
from typing import Iterator

class Countdown:
    def __init__(self, start: int) -> None:
        self._n = start

    def __iter__(self) -> Iterator[int]:
        return self

    def __next__(self) -> int:
        if self._n <= 0:
            raise StopIteration
        self._n -= 1
        return self._n + 1
```

### Typescript

```typescript
class Countdown implements Iterable<number> {
  constructor(private n: number) {}

  *[Symbol.iterator](): Iterator<number> {
    while (this.n > 0) {
      yield this.n--;
    }
  }
}

// Usage: [...new Countdown(3)] => [3, 2, 1]
```

## When to Use

- When you need to traverse a collection without exposing its internal structure.
- When you want to support multiple simultaneous traversals.
- When you need lazy evaluation over potentially large or infinite sequences.

## When NOT to Use

- When the language's built-in iteration (for-in, range, iterators) already suffices.
- When random access is needed rather than sequential traversal.

## Anti-Patterns

- Implementing a custom iterator when the standard library provides one.
- Iterators that mutate the underlying collection during traversal.
- Eager materialization of large sequences when lazy iteration would suffice.

## Related Patterns

- [behavioral/visitor](visitor.md) -- visitor traverses a structure and operates on each element; iterator just provides access.
- [behavioral/command](command.md) -- iterators can yield commands for deferred execution.
- [structural/composite](../structural/composite.md) -- iterators are often used to traverse composite structures.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 5.
- Refactoring.Guru -- Iterator: https://refactoring.guru/design-patterns/iterator
