---
name: strategy
category: behavioral
tags: [behavioral, algorithm, polymorphism, composition]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Define a family of algorithms, encapsulate each one, and make them interchangeable at runtime without changing the clients that use them.

## Problem

You have multiple algorithms for a task (e.g., sorting, pricing, compression) and clients embed algorithm selection via conditionals. Adding a new variant requires modifying existing code, violating the Open-Closed Principle.

## Solution

Extract each algorithm behind a common interface (trait, interface, protocol). Clients depend on the abstraction and receive the concrete strategy via injection. New algorithms are added without touching existing code.

## Language Implementations

### Rust

```rust
trait Compressor: Send + Sync {
    fn compress(&self, data: &[u8]) -> Vec<u8>;
}

struct Gzip;
impl Compressor for Gzip {
    fn compress(&self, data: &[u8]) -> Vec<u8> { /* gzip bytes */ vec![] }
}

fn upload(data: &[u8], compressor: &dyn Compressor) {
    let compressed = compressor.compress(data);
    // send compressed bytes
}
```

### Go

```go
type Compressor interface {
    Compress(data []byte) ([]byte, error)
}

type GzipCompressor struct{}

func (g GzipCompressor) Compress(data []byte) ([]byte, error) {
    // gzip implementation
    return data, nil
}

func Upload(data []byte, c Compressor) error {
    compressed, err := c.Compress(data)
    _ = compressed
    return err
}
```

### Python

```python
from typing import Protocol

class Compressor(Protocol):
    def compress(self, data: bytes) -> bytes: ...

class GzipCompressor:
    def compress(self, data: bytes) -> bytes:
        import gzip
        return gzip.compress(data)

def upload(data: bytes, compressor: Compressor) -> None:
    compressed = compressor.compress(data)
```

### Typescript

```typescript
interface Compressor {
  compress(data: Uint8Array): Uint8Array;
}

class GzipCompressor implements Compressor {
  compress(data: Uint8Array): Uint8Array { return data; /* gzip */ }
}

function upload(data: Uint8Array, compressor: Compressor): void {
  const compressed = compressor.compress(data);
}
```

## When to Use

- When you have multiple algorithms for the same task and need to switch between them.
- When you want to eliminate conditional branches that select algorithm variants.
- When algorithm details should be hidden from the client.

## When NOT to Use

- When there is only one algorithm with no foreseeable variants -- a direct call is simpler.
- When the algorithm selection never changes at runtime -- a compile-time generic may suffice.

## Anti-Patterns

- Creating a strategy interface with only one implementation and no plans for more.
- Passing context data through the strategy that couples it to a specific caller.
- Using strategy when a simple function pointer or closure would suffice.

## Related Patterns

- [behavioral/state](state.md) -- similar structure but strategies are stateless; state objects carry transition logic.
- [behavioral/template-method](template-method.md) -- uses inheritance to vary steps; strategy uses composition.
- [behavioral/command](command.md) -- encapsulates a request; strategy encapsulates an algorithm.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 5.
- Refactoring.Guru -- Strategy: https://refactoring.guru/design-patterns/strategy
