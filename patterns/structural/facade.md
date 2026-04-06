---
name: facade
category: structural
tags: [structural, simplification, api-design, encapsulation]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Provide a unified interface to a set of interfaces in a subsystem. Facade defines a higher-level interface that makes the subsystem easier to use.

## Problem

A subsystem has grown complex with many interacting classes. Client code must understand internal dependencies, initialization order, and coordination between components. You need a simple entry point that hides this complexity.

## Solution

Create a facade type that exposes a small number of high-level methods. The facade orchestrates calls to subsystem components internally. Clients interact with the facade only, remaining unaware of subsystem internals.

## Language Implementations

### Rust

Facade hiding multi-step video conversion:

```rust
struct VideoFile { path: String }
struct Codec;
impl Codec { fn decode(&self, _f: &VideoFile) -> Vec<u8> { vec![] } }
struct Compressor;
impl Compressor { fn compress(&self, data: &[u8]) -> Vec<u8> { vec![] } }

struct VideoConverter;
impl VideoConverter {
    pub fn convert(&self, path: &str) -> Vec<u8> {
        let file = VideoFile { path: path.to_string() };
        let raw = Codec.decode(&file);
        Compressor.compress(&raw)
    }
}
```

### Go

```go
type VideoConverter struct{}

func (v *VideoConverter) Convert(path string) ([]byte, error) {
    raw, err := decode(path)
    if err != nil { return nil, err }
    return compress(raw)
}

func decode(path string) ([]byte, error) { return nil, nil }
func compress(data []byte) ([]byte, error) { return nil, nil }
```

### Python

```python
class VideoConverter:
    def convert(self, path: str) -> bytes:
        raw = self._decode(path)
        return self._compress(raw)

    def _decode(self, path: str) -> bytes:
        return b""

    def _compress(self, data: bytes) -> bytes:
        return b""
```

### Typescript

```typescript
class VideoConverter {
  convert(path: string): Uint8Array {
    const raw = this.decode(path);
    return this.compress(raw);
  }

  private decode(path: string): Uint8Array { return new Uint8Array(); }
  private compress(data: Uint8Array): Uint8Array { return new Uint8Array(); }
}
```

## When to Use

- When a subsystem has many components and clients need a simple entry point.
- When you want to layer a system and define entry points for each layer.
- When decoupling clients from subsystem internals to reduce coupling.

## When NOT to Use

- When the subsystem is already simple — a facade adds an unnecessary layer.
- When clients genuinely need fine-grained control over subsystem components.

## Anti-Patterns

- Creating a "god facade" that exposes every subsystem method — defeats the purpose of simplification.
- Putting business logic in the facade — it should only delegate and coordinate.
- Making the facade the only way to access subsystem components, preventing legitimate advanced use.

## Related Patterns

- [structural/adapter](adapter.md) — wraps a single interface; facade wraps an entire subsystem.
- [creational/abstract-factory](../creational/abstract-factory.md) — can be used with facade to create subsystem objects.
- [structural/proxy](proxy.md) — controls access to a single object; facade simplifies access to many.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 4.
- Refactoring.Guru — Facade: https://refactoring.guru/design-patterns/facade
