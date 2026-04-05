---
name: state
category: behavioral
tags: [behavioral, state-machine, transition, polymorphism]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Allow an object to alter its behavior when its internal state changes, making it appear as if the object changed its class.

## Problem

An object's behavior depends on its state, and it must change behavior at runtime based on that state. Conditionals (if/switch on a state field) spread state logic across many methods, making it hard to add new states.

## Solution

Encapsulate state-specific behavior in separate state objects that share a common interface. The context delegates behavior to its current state object and transitions by swapping the state reference.

## Language Implementations

### Rust

Rust's enum + match is idiomatic for state machines with a fixed set of states:

```rust
enum Door { Locked, Open, Closed }

impl Door {
    fn on_push(self) -> Self {
        match self {
            Door::Locked => Door::Locked,
            Door::Closed => Door::Open,
            Door::Open => Door::Open,
        }
    }
    fn on_lock(self) -> Self {
        match self {
            Door::Open => Door::Open,
            _ => Door::Locked,
        }
    }
}
```

### Go

```go
type State interface {
    OnPush() State
    OnLock() State
}

type Closed struct{}
type Open struct{}

func (Closed) OnPush() State { return Open{} }
func (Closed) OnLock() State { return Closed{} }
func (Open) OnPush() State   { return Open{} }
func (Open) OnLock() State   { return Closed{} }
```

### Python

```python
from typing import Protocol

class State(Protocol):
    def on_push(self) -> "State": ...
    def on_lock(self) -> "State": ...

class Closed:
    def on_push(self) -> State: return Open()
    def on_lock(self) -> State: return self

class Open:
    def on_push(self) -> State: return self
    def on_lock(self) -> State: return Closed()
```

### Typescript

```typescript
interface State {
  onPush(): State;
  onLock(): State;
}

class Closed implements State {
  onPush(): State { return new Open(); }
  onLock(): State { return this; }
}

class Open implements State {
  onPush(): State { return this; }
  onLock(): State { return new Closed(); }
}
```

## When to Use

- When an object's behavior changes based on its internal state.
- When state-specific logic is complex enough that conditionals become unwieldy.
- When you need a clear, explicit state machine with well-defined transitions.

## When NOT to Use

- When there are only two simple states -- a boolean flag is sufficient.
- When state transitions are trivial and do not warrant separate types.

## Anti-Patterns

- Allowing states to hold references to the context, creating circular dependencies.
- Having states with partially overlapping behavior without a shared base.
- Using the state pattern when a simple enum and match/switch is clearer.

## Related Patterns

- [behavioral/strategy](strategy.md) -- strategy swaps algorithms; state swaps behavior based on internal state.
- [behavioral/command](command.md) -- state transitions can be modeled as commands.
- [behavioral/memento](memento.md) -- can capture state snapshots for rollback.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 5.
- Refactoring.Guru -- State: https://refactoring.guru/design-patterns/state
