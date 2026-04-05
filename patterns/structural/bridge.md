---
name: bridge
category: structural
tags: [structural, abstraction, decoupling, composition]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Decouple an abstraction from its implementation so that the two can vary independently.

## Problem

You have a concept with multiple dimensions of variation (e.g., shape + renderer, notification + channel). Inheritance-based approaches create a combinatorial explosion of subclasses. You need a way to compose these dimensions independently without coupling them at compile time.

## Solution

Split the class hierarchy into two separate hierarchies — one for the abstraction and one for the implementation. The abstraction holds a reference to an implementation object and delegates work to it. In Rust, use trait objects or generics to bridge the two hierarchies.

## Language Implementations

### Rust

Trait-based bridge with generic abstraction:

```rust
trait Renderer {
    fn render_circle(&self, x: f64, y: f64, radius: f64);
}

struct VectorRenderer;
impl Renderer for VectorRenderer {
    fn render_circle(&self, x: f64, y: f64, radius: f64) { /* SVG */ }
}

struct Circle<R: Renderer> {
    x: f64, y: f64, radius: f64, renderer: R,
}

impl<R: Renderer> Circle<R> {
    fn draw(&self) { self.renderer.render_circle(self.x, self.y, self.radius); }
}
```

### Go

```go
type Renderer interface {
    RenderCircle(x, y, radius float64)
}

type VectorRenderer struct{}
func (v *VectorRenderer) RenderCircle(x, y, radius float64) { /* SVG */ }

type Circle struct {
    X, Y, Radius float64
    Renderer     Renderer
}

func (c *Circle) Draw() { c.Renderer.RenderCircle(c.X, c.Y, c.Radius) }
```

### Python

```python
from abc import ABC, abstractmethod

class Renderer(ABC):
    @abstractmethod
    def render_circle(self, x: float, y: float, radius: float) -> None: ...

class VectorRenderer(Renderer):
    def render_circle(self, x: float, y: float, radius: float) -> None: ...

class Circle:
    def __init__(self, x: float, y: float, radius: float, renderer: Renderer):
        self.x, self.y, self.radius, self.renderer = x, y, radius, renderer

    def draw(self) -> None:
        self.renderer.render_circle(self.x, self.y, self.radius)
```

### Typescript

```typescript
interface Renderer {
  renderCircle(x: number, y: number, radius: number): void;
}

class VectorRenderer implements Renderer {
  renderCircle(x: number, y: number, radius: number): void { /* SVG */ }
}

class Circle {
  constructor(private x: number, private y: number,
              private radius: number, private renderer: Renderer) {}
  draw(): void { this.renderer.renderCircle(this.x, this.y, this.radius); }
}
```

## When to Use

- When an abstraction has multiple orthogonal dimensions of variation.
- When you want to switch implementations at runtime without affecting the abstraction.
- When both the abstraction and implementation hierarchies need to evolve independently.

## When NOT to Use

- When there is only one implementation dimension — simple dependency injection suffices.
- When the abstraction and implementation are tightly coupled by design and unlikely to change.

## Anti-Patterns

- Creating a bridge when there is only one implementor — adds complexity with no benefit.
- Leaking implementation details through the bridge interface, defeating the decoupling.
- Confusing bridge with adapter — bridge is designed upfront; adapter retrofits compatibility.

## Related Patterns

- [structural/adapter](adapter.md) — retrofits interface compatibility; bridge separates dimensions upfront.
- [creational/abstract-factory](../creational/abstract-factory.md) — can create platform-specific implementations for a bridge.
- [structural/decorator](decorator.md) — extends behavior on one side of the bridge.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 4.
- Refactoring.Guru — Bridge: https://refactoring.guru/design-patterns/bridge
