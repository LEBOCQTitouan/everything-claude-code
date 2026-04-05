---
name: chain-of-responsibility
category: behavioral
tags: [behavioral, middleware, pipeline, decoupling]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Avoid coupling the sender of a request to its receiver by giving more than one object a chance to handle the request. Chain the receiving objects and pass the request along until one handles it.

## Problem

A request must be processed, but the handler is not known in advance. Hard-coding the handler selection couples the sender to specific receivers and makes adding new handlers require modifying the dispatch logic.

## Solution

Create a chain of handler objects, each with a reference to the next handler. Each handler either processes the request or forwards it to the next handler in the chain. The sender only knows the first handler.

## Language Implementations

### Rust

```rust
type Handler = Box<dyn Fn(u32) -> Option<String>>;

fn chain(handlers: &[Handler], request: u32) -> String {
    handlers.iter()
        .find_map(|h| h(request))
        .unwrap_or_else(|| "unhandled".into())
}
```

### Go

```go
type Handler func(req int) (string, bool)

func Chain(handlers []Handler, req int) string {
    for _, h := range handlers {
        if result, ok := h(req); ok { return result }
    }
    return "unhandled"
}
```

### Python

```python
from typing import Callable

Handler = Callable[[int], str | None]

def chain(handlers: list[Handler], request: int) -> str:
    for handler in handlers:
        if (result := handler(request)) is not None:
            return result
    return "unhandled"
```

### Typescript

```typescript
type Handler = (req: number) => string | null;

function chain(handlers: Handler[], request: number): string {
  for (const handler of handlers) {
    const result = handler(request);
    if (result !== null) return result;
  }
  return "unhandled";
}
```

## When to Use

- When more than one object may handle a request and the handler is not known a priori.
- When you want to issue a request without specifying the receiver explicitly.
- When the set of handlers should be configurable dynamically (middleware stacks, plugin systems).

## When NOT to Use

- When there is always exactly one handler -- direct dispatch is simpler.
- When the chain is long and performance-critical, since every handler is consulted.

## Anti-Patterns

- Chains where no handler ever processes the request, silently dropping it.
- Handlers that modify the request in unexpected ways as it passes through.
- Excessively long chains that make debugging difficult.

## Related Patterns

- [behavioral/command](command.md) -- commands can be passed through a chain.
- [behavioral/mediator](mediator.md) -- mediator centralizes dispatch; chain distributes it.
- [structural/decorator](../structural/decorator.md) -- decorators wrap behavior; chain passes requests along.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 5.
- Refactoring.Guru -- Chain of Responsibility: https://refactoring.guru/design-patterns/chain-of-responsibility
