---
name: adapter
category: structural
tags: [structural, wrapper, interface-conversion]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Convert the interface of a class into another interface clients expect. Adapter lets classes work together that could not otherwise because of incompatible interfaces.

## Problem

You need to integrate a third-party or legacy component whose interface does not match the one your system expects. Rewriting either side is impractical or impossible — you need a translation layer that bridges the gap without modifying existing code.

## Solution

Create a wrapper type that implements the target interface and delegates calls to the adaptee, translating arguments and return values as needed. In Rust, implement the target trait for a newtype wrapping the adaptee. In Go, embed the adaptee and implement the target interface.

## Language Implementations

### Rust

Newtype adapter implementing a target trait:

```rust
trait Logger {
    fn log(&self, msg: &str);
}

struct ExternalLogger; // third-party, cannot modify
impl ExternalLogger {
    fn write_entry(&self, entry: &str) { /* ... */ }
}

struct LoggerAdapter(ExternalLogger);

impl Logger for LoggerAdapter {
    fn log(&self, msg: &str) {
        self.0.write_entry(msg);
    }
}
```

### Go

```go
type Logger interface {
    Log(msg string)
}

type ExternalLogger struct{}
func (e *ExternalLogger) WriteEntry(entry string) { /* ... */ }

type LoggerAdapter struct {
    ext *ExternalLogger
}

func (a *LoggerAdapter) Log(msg string) {
    a.ext.WriteEntry(msg)
}
```

### Python

```python
from abc import ABC, abstractmethod

class Logger(ABC):
    @abstractmethod
    def log(self, msg: str) -> None: ...

class ExternalLogger:
    def write_entry(self, entry: str) -> None: ...

class LoggerAdapter(Logger):
    def __init__(self, ext: ExternalLogger) -> None:
        self._ext = ext

    def log(self, msg: str) -> None:
        self._ext.write_entry(msg)
```

### Typescript

```typescript
interface Logger {
  log(msg: string): void;
}

class ExternalLogger {
  writeEntry(entry: string): void { /* ... */ }
}

class LoggerAdapter implements Logger {
  constructor(private ext: ExternalLogger) {}
  log(msg: string): void { this.ext.writeEntry(msg); }
}
```

## When to Use

- When integrating third-party libraries with incompatible interfaces.
- When migrating from one API to another incrementally.
- When you want to decouple domain logic from infrastructure details (ports and adapters / hexagonal architecture).

## When NOT to Use

- When the interfaces are already compatible — adding an adapter is unnecessary indirection.
- When you control both sides and can simply align the interfaces directly.

## Anti-Patterns

- Creating adapters that add business logic beyond simple translation — keep adapters thin.
- Chaining multiple adapters instead of designing a clean intermediate interface.
- Using adapters to paper over a fundamentally broken API rather than fixing the design.

## Related Patterns

- [structural/facade](facade.md) — simplifies a complex subsystem; adapter translates a single interface.
- [structural/decorator](decorator.md) — adds behavior without changing interface; adapter changes the interface.
- [structural/bridge](bridge.md) — separates abstraction from implementation upfront; adapter retrofits compatibility.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 4.
- Refactoring.Guru — Adapter: https://refactoring.guru/design-patterns/adapter
- Hexagonal Architecture (Ports and Adapters): https://alistair.cockburn.us/hexagonal-architecture/
