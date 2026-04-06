---
name: observer
category: behavioral
tags: [behavioral, event, pub-sub, decoupling]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Define a one-to-many dependency so that when one object changes state, all its dependents are notified and updated automatically.

## Problem

A component needs to notify an unknown number of other components about state changes. Hard-coding the notification targets creates tight coupling and makes adding new listeners require modifying the source.

## Solution

The subject maintains a list of observers and notifies them through a common interface. Observers register and deregister dynamically. The subject knows nothing about concrete observer types.

## Language Implementations

### Rust

```rust
type Listener = Box<dyn Fn(&str) + Send>;

struct EventBus {
    listeners: Vec<Listener>,
}

impl EventBus {
    fn new() -> Self { Self { listeners: vec![] } }
    fn subscribe(&mut self, f: Listener) { self.listeners.push(f); }
    fn emit(&self, event: &str) {
        for listener in &self.listeners { listener(event); }
    }
}
```

### Go

```go
type Observer interface {
    OnEvent(event string)
}

type EventBus struct {
    observers []Observer
}

func (eb *EventBus) Subscribe(o Observer) { eb.observers = append(eb.observers, o) }
func (eb *EventBus) Emit(event string) {
    for _, o := range eb.observers { o.OnEvent(event) }
}
```

### Python

```python
from typing import Callable

class EventBus:
    def __init__(self) -> None:
        self._listeners: list[Callable[[str], None]] = []

    def subscribe(self, fn: Callable[[str], None]) -> None:
        self._listeners.append(fn)

    def emit(self, event: str) -> None:
        for fn in self._listeners:
            fn(event)
```

### Typescript

```typescript
type Listener = (event: string) => void;

class EventBus {
  private listeners: Listener[] = [];

  subscribe(fn: Listener): void { this.listeners.push(fn); }
  emit(event: string): void {
    this.listeners.forEach((fn) => fn(event));
  }
}
```

## When to Use

- When a change to one object requires updating others, and you do not know how many objects need updating.
- When you need loose coupling between a publisher and its subscribers.
- When event-driven architecture fits the domain.

## When NOT to Use

- When there is only one subscriber -- a direct callback is simpler.
- When notification order matters and observers have complex interdependencies.
- When cascading updates risk infinite loops or performance problems.

## Anti-Patterns

- Forgetting to unsubscribe, causing memory leaks (especially in UI frameworks).
- Notifying observers synchronously in a performance-critical path without async support.
- Observers modifying the subject during notification, causing re-entrant updates.

## Related Patterns

- [behavioral/mediator](mediator.md) -- centralizes communication; observer is decentralized.
- [behavioral/command](command.md) -- can queue events for later processing.
- [behavioral/chain-of-responsibility](chain-of-responsibility.md) -- passes events along a chain rather than broadcasting.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 5.
- Refactoring.Guru -- Observer: https://refactoring.guru/design-patterns/observer
