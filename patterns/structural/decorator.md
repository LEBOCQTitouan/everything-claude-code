---
name: decorator
category: structural
tags: [structural, wrapper, middleware, composition]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Attach additional responsibilities to an object dynamically. Decorators provide a flexible alternative to subclassing for extending functionality.

## Problem

You need to add behavior to individual objects (logging, caching, retry, authorization) without affecting other objects of the same class. Subclassing each combination creates a class explosion. You need composable, stackable behavior additions.

## Solution

Wrap the original object in a decorator that implements the same interface. The decorator delegates to the wrapped object and adds behavior before or after the delegation. Multiple decorators can be stacked. In Rust, implement the same trait and hold a boxed inner instance.

## Language Implementations

### Rust

Trait-based decorator with stacking:

```rust
trait Notifier: Send + Sync {
    fn send(&self, msg: &str);
}

struct EmailNotifier;
impl Notifier for EmailNotifier {
    fn send(&self, msg: &str) { /* send email */ }
}

struct LoggingNotifier<N: Notifier> { inner: N }
impl<N: Notifier> Notifier for LoggingNotifier<N> {
    fn send(&self, msg: &str) {
        println!("Sending: {msg}");
        self.inner.send(msg);
    }
}
```

### Go

```go
type Notifier interface {
    Send(msg string)
}

type EmailNotifier struct{}
func (e *EmailNotifier) Send(msg string) { /* send email */ }

type LoggingNotifier struct{ Inner Notifier }
func (l *LoggingNotifier) Send(msg string) {
    fmt.Printf("Sending: %s\n", msg)
    l.Inner.Send(msg)
}
```

### Python

```python
from abc import ABC, abstractmethod

class Notifier(ABC):
    @abstractmethod
    def send(self, msg: str) -> None: ...

class EmailNotifier(Notifier):
    def send(self, msg: str) -> None: ...

class LoggingNotifier(Notifier):
    def __init__(self, inner: Notifier) -> None:
        self._inner = inner

    def send(self, msg: str) -> None:
        print(f"Sending: {msg}")
        self._inner.send(msg)
```

### Typescript

```typescript
interface Notifier {
  send(msg: string): void;
}

class EmailNotifier implements Notifier {
  send(msg: string): void { /* send email */ }
}

class LoggingNotifier implements Notifier {
  constructor(private inner: Notifier) {}
  send(msg: string): void {
    console.log(`Sending: ${msg}`);
    this.inner.send(msg);
  }
}
```

## When to Use

- When you need to add cross-cutting concerns (logging, caching, auth, retry) to objects.
- When behavior must be composable and stackable at runtime.
- When subclassing would cause a combinatorial explosion of classes.

## When NOT to Use

- When only one decoration is ever needed — direct composition is simpler.
- When the interface has many methods and decorating all of them is tedious — consider aspect-oriented approaches.

## Anti-Patterns

- Decorators that depend on specific ordering without documenting it — makes stacking fragile.
- Heavy decorators that violate the single responsibility principle.
- Using decorators when a simple function wrapper or middleware chain would suffice.

## Related Patterns

- [structural/adapter](adapter.md) — changes the interface; decorator preserves the interface.
- [structural/composite](composite.md) — composite aggregates children; decorator wraps a single component.
- [structural/proxy](proxy.md) — controls access; decorator adds behavior. Structurally similar, different intent.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 4.
- Refactoring.Guru — Decorator: https://refactoring.guru/design-patterns/decorator
