---
name: flyweight
category: structural
tags: [structural, memory-optimization, sharing, caching]
languages: [rust, go, python, typescript]
difficulty: advanced
---

## Intent

Use sharing to support large numbers of fine-grained objects efficiently by separating intrinsic (shared) state from extrinsic (context-specific) state.

## Problem

Your application creates a huge number of similar objects (e.g., characters in a text editor, particles in a game, tree sprites in a forest). Each object consumes memory for data that is largely identical. You need to reduce memory usage without changing the client interface.

## Solution

Extract the shared intrinsic state into flyweight objects stored in a pool/factory. Clients pass extrinsic state (position, context) at call time rather than storing it in the object. The factory returns existing flyweights when possible, dramatically reducing object count.

## Language Implementations

### Rust

N/A -- Rust's ownership model and zero-cost abstractions make the classical flyweight pattern largely unnecessary. Shared immutable data uses `Arc<T>` or `&'static T` naturally. String interning is handled by crates like `string-cache` or `lasso`. The borrow checker ensures shared references are safe without a manual pool. If you do need a pool, use a `HashMap<Key, Arc<Value>>`:

```rust
use std::collections::HashMap;
use std::sync::Arc;

struct GlyphPool {
    glyphs: HashMap<char, Arc<GlyphData>>,
}

impl GlyphPool {
    fn get(&mut self, ch: char) -> Arc<GlyphData> {
        self.glyphs.entry(ch)
            .or_insert_with(|| Arc::new(GlyphData::load(ch)))
            .clone()
    }
}

struct GlyphData { /* font metrics, bitmap */ }
impl GlyphData { fn load(_ch: char) -> Self { GlyphData {} } }
```

### Go

```go
type GlyphData struct{ /* font metrics, bitmap */ }

type GlyphPool struct {
    glyphs map[rune]*GlyphData
}

func NewGlyphPool() *GlyphPool {
    return &GlyphPool{glyphs: make(map[rune]*GlyphData)}
}

func (p *GlyphPool) Get(ch rune) *GlyphData {
    if g, ok := p.glyphs[ch]; ok { return g }
    g := &GlyphData{} // load from font
    p.glyphs[ch] = g
    return g
}
```

### Python

```python
class GlyphData:
    def __init__(self, char: str) -> None:
        self.char = char  # intrinsic shared state

class GlyphPool:
    def __init__(self) -> None:
        self._pool: dict[str, GlyphData] = {}

    def get(self, char: str) -> GlyphData:
        if char not in self._pool:
            self._pool[char] = GlyphData(char)
        return self._pool[char]
```

### Typescript

```typescript
class GlyphData {
  constructor(readonly char: string) {} // intrinsic shared state
}

class GlyphPool {
  private pool = new Map<string, GlyphData>();

  get(char: string): GlyphData {
    if (!this.pool.has(char)) {
      this.pool.set(char, new GlyphData(char));
    }
    return this.pool.get(char)!;
  }
}
```

## When to Use

- When an application uses a huge number of objects that share most of their state.
- When memory is a bottleneck and many objects have identical intrinsic data.
- When extrinsic state can be computed or passed in rather than stored per object.

## When NOT to Use

- When the number of objects is small — the pool overhead exceeds savings.
- When objects have little shared state — most state is extrinsic, so sharing is minimal.
- In Rust: when `Arc`, `&'static`, or string interning already handles sharing idiomatically.

## Anti-Patterns

- Storing extrinsic state in the flyweight, defeating the memory savings.
- Making flyweights mutable — shared objects must be immutable to be safe.
- Using flyweight prematurely without profiling to confirm memory is the actual bottleneck.

## Related Patterns

- [structural/composite](composite.md) — leaf nodes in a composite tree are often flyweights.
- [creational/factory-method](../creational/factory-method.md) — flyweight factory manages the pool of shared objects.
- [structural/proxy](proxy.md) — proxy controls access; flyweight controls memory usage.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 4.
- Refactoring.Guru — Flyweight: https://refactoring.guru/design-patterns/flyweight
- Rust string interning: https://github.com/Lark-Base/lasso
