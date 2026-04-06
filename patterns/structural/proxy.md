---
name: proxy
category: structural
tags: [structural, access-control, lazy-loading, caching]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Provide a surrogate or placeholder for another object to control access to it.

## Problem

You need to control access to an object — for lazy initialization, access control, logging, caching, or remote access. Modifying the original object violates the single responsibility principle. You need a stand-in that intercepts operations transparently.

## Solution

Create a proxy that implements the same interface as the real subject. The proxy holds a reference to the real subject and delegates calls to it, adding control logic before or after delegation. Clients interact with the proxy as if it were the real object.

## Language Implementations

### Rust

Lazy-loading proxy using `OnceCell`:

```rust
use std::cell::OnceCell;

trait Image {
    fn display(&self);
}

struct RealImage { data: Vec<u8> }
impl RealImage { fn load(path: &str) -> Self { RealImage { data: vec![] } } }
impl Image for RealImage { fn display(&self) { /* render */ } }

struct LazyImage { path: String, inner: OnceCell<RealImage> }
impl Image for LazyImage {
    fn display(&self) {
        self.inner.get_or_init(|| RealImage::load(&self.path)).display();
    }
}
```

### Go

```go
type Image interface {
    Display()
}

type RealImage struct{ data []byte }
func LoadImage(path string) *RealImage { return &RealImage{} }
func (r *RealImage) Display() { /* render */ }

type LazyImage struct {
    path  string
    inner *RealImage
}

func (l *LazyImage) Display() {
    if l.inner == nil { l.inner = LoadImage(l.path) }
    l.inner.Display()
}
```

### Python

```python
from abc import ABC, abstractmethod

class Image(ABC):
    @abstractmethod
    def display(self) -> None: ...

class RealImage(Image):
    def __init__(self, path: str) -> None:
        self.data = b""  # expensive load
    def display(self) -> None: ...

class LazyImage(Image):
    def __init__(self, path: str) -> None:
        self._path = path
        self._inner: RealImage | None = None

    def display(self) -> None:
        if self._inner is None:
            self._inner = RealImage(self._path)
        self._inner.display()
```

### Typescript

```typescript
interface Image {
  display(): void;
}

class RealImage implements Image {
  private data: Uint8Array;
  constructor(path: string) { this.data = new Uint8Array(); }
  display(): void { /* render */ }
}

class LazyImage implements Image {
  private inner?: RealImage;
  constructor(private path: string) {}
  display(): void {
    this.inner ??= new RealImage(this.path);
    this.inner.display();
  }
}
```

## When to Use

- When you need lazy initialization of expensive objects (virtual proxy).
- When you need access control or permission checks (protection proxy).
- When you need logging, metrics, or caching around object access (smart proxy).
- When the real object is remote and you need a local representative (remote proxy).

## When NOT to Use

- When the object is cheap to create — lazy loading adds complexity with no benefit.
- When access control belongs at a higher architectural layer (e.g., middleware, gateway).

## Anti-Patterns

- Proxies that diverge from the real subject's interface, breaking substitutability.
- Stacking multiple proxy layers without clear separation of concerns.
- Using proxies for business logic instead of pure access control or lifecycle management.

## Related Patterns

- [structural/decorator](decorator.md) — adds behavior; proxy controls access. Structurally similar, different intent.
- [structural/adapter](adapter.md) — provides a different interface; proxy preserves the same interface.
- [structural/facade](facade.md) — simplifies a subsystem; proxy controls access to a single object.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 4.
- Refactoring.Guru — Proxy: https://refactoring.guru/design-patterns/proxy
