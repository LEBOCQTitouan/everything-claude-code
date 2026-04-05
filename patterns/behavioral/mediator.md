---
name: mediator
category: behavioral
tags: [behavioral, decoupling, coordination, hub]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Define an object that encapsulates how a set of objects interact, promoting loose coupling by keeping objects from referring to each other explicitly.

## Problem

Multiple objects communicate directly with each other, creating a tangled web of dependencies. Adding or changing a participant requires modifying many others. The interaction logic is scattered across all participants.

## Solution

Introduce a mediator object that all participants communicate through. Each participant knows only the mediator, not the other participants. The mediator contains the interaction logic and coordinates communication.

## Language Implementations

### Rust

```rust
trait Mediator {
    fn notify(&self, sender: &str, event: &str);
}

struct ChatRoom;
impl Mediator for ChatRoom {
    fn notify(&self, sender: &str, event: &str) {
        println!("[{sender}] {event}");
        // route to other participants
    }
}

fn send(mediator: &dyn Mediator, user: &str, msg: &str) {
    mediator.notify(user, msg);
}
```

### Go

```go
type Mediator interface {
    Notify(sender, event string)
}

type ChatRoom struct{}

func (c *ChatRoom) Notify(sender, event string) {
    fmt.Printf("[%s] %s\n", sender, event)
}

func Send(m Mediator, user, msg string) { m.Notify(user, msg) }
```

### Python

```python
from typing import Protocol

class Mediator(Protocol):
    def notify(self, sender: str, event: str) -> None: ...

class ChatRoom:
    def notify(self, sender: str, event: str) -> None:
        print(f"[{sender}] {event}")

def send(mediator: Mediator, user: str, msg: str) -> None:
    mediator.notify(user, msg)
```

### Typescript

```typescript
interface Mediator {
  notify(sender: string, event: string): void;
}

class ChatRoom implements Mediator {
  notify(sender: string, event: string): void {
    console.log(`[${sender}] ${event}`);
  }
}

function send(m: Mediator, user: string, msg: string): void {
  m.notify(user, msg);
}
```

## When to Use

- When many objects communicate in complex ways and the coupling is hard to manage.
- When you want to centralize control logic that spans multiple objects.
- When reusing a component is difficult because it depends on too many peers.

## When NOT to Use

- When only two objects interact -- direct communication is simpler.
- When the mediator becomes a god object that knows too much about every participant.

## Anti-Patterns

- Letting the mediator grow into a monolithic controller with all business logic.
- Having participants bypass the mediator for "efficiency," reintroducing coupling.
- Creating circular dependencies between the mediator and participants.

## Related Patterns

- [behavioral/observer](observer.md) -- observer is decentralized notification; mediator centralizes it.
- [behavioral/command](command.md) -- commands can be routed through a mediator.
- [structural/facade](../structural/facade.md) -- facade simplifies a subsystem interface; mediator coordinates peer interactions.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 5.
- Refactoring.Guru -- Mediator: https://refactoring.guru/design-patterns/mediator
