---
name: builder
category: creational
tags: [creational, step-by-step, typestate]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Separate the construction of a complex object from its representation so that the same construction process can create different representations.

## Problem

An object requires many parameters for construction, many of which may be optional. Using a single large constructor leads to telescoping constructors or mutable intermediate state. You need a way to build objects step by step with compile-time or runtime safety.

## Solution

Provide a builder type that accumulates configuration through method chaining. In Rust, use the typestate pattern to encode required-field completion in the type system, catching missing fields at compile time rather than runtime.

## Language Implementations

### Rust

Typestate builder — required fields enforced at compile time:

```rust
use std::marker::PhantomData;

struct NeedsHost;
struct NeedsPort;
struct Ready;

struct ConnectionBuilder<State> {
    host: Option<String>,
    port: Option<u16>,
    timeout_ms: u64,
    _state: PhantomData<State>,
}

impl ConnectionBuilder<NeedsHost> {
    pub fn new() -> Self {
        Self { host: None, port: None, timeout_ms: 5000, _state: PhantomData }
    }

    pub fn host(self, h: impl Into<String>) -> ConnectionBuilder<NeedsPort> {
        ConnectionBuilder { host: Some(h.into()), port: None, timeout_ms: self.timeout_ms, _state: PhantomData }
    }
}

impl ConnectionBuilder<NeedsPort> {
    pub fn port(self, p: u16) -> ConnectionBuilder<Ready> {
        ConnectionBuilder { host: self.host, port: Some(p), timeout_ms: self.timeout_ms, _state: PhantomData }
    }
}

impl ConnectionBuilder<Ready> {
    pub fn timeout_ms(mut self, ms: u64) -> Self { self.timeout_ms = ms; self }

    pub fn build(self) -> Connection {
        Connection {
            host: self.host.unwrap(),
            port: self.port.unwrap(),
            timeout_ms: self.timeout_ms,
        }
    }
}

struct Connection { host: String, port: u16, timeout_ms: u64 }

// Usage — compile error if host or port is missing:
// let conn = ConnectionBuilder::new().host("localhost").port(5432).build();
```

### Go

```go
type Connection struct {
    Host      string
    Port      uint16
    TimeoutMs int64
}

type ConnectionBuilder struct {
    host      string
    port      uint16
    timeoutMs int64
}

func NewConnectionBuilder() *ConnectionBuilder {
    return &ConnectionBuilder{timeoutMs: 5000}
}

func (b *ConnectionBuilder) Host(h string) *ConnectionBuilder { b.host = h; return b }
func (b *ConnectionBuilder) Port(p uint16) *ConnectionBuilder { b.port = p; return b }
func (b *ConnectionBuilder) TimeoutMs(ms int64) *ConnectionBuilder { b.timeoutMs = ms; return b }

func (b *ConnectionBuilder) Build() (Connection, error) {
    if b.host == "" { return Connection{}, fmt.Errorf("host is required") }
    if b.port == 0  { return Connection{}, fmt.Errorf("port is required") }
    return Connection{Host: b.host, Port: b.port, TimeoutMs: b.timeoutMs}, nil
}
```

### Python

```python
from dataclasses import dataclass, field

@dataclass
class Connection:
    host: str
    port: int
    timeout_ms: int = 5000

class ConnectionBuilder:
    def __init__(self) -> None:
        self._host: str | None = None
        self._port: int | None = None
        self._timeout_ms: int = 5000

    def host(self, h: str) -> "ConnectionBuilder":
        self._host = h
        return self

    def port(self, p: int) -> "ConnectionBuilder":
        self._port = p
        return self

    def timeout_ms(self, ms: int) -> "ConnectionBuilder":
        self._timeout_ms = ms
        return self

    def build(self) -> Connection:
        if self._host is None:
            raise ValueError("host is required")
        if self._port is None:
            raise ValueError("port is required")
        return Connection(host=self._host, port=self._port, timeout_ms=self._timeout_ms)
```

### Typescript

```typescript
class ConnectionBuilder {
  private host?: string;
  private port?: number;
  private timeoutMs = 5000;

  withHost(h: string): this { this.host = h; return this; }
  withPort(p: number): this { this.port = p; return this; }
  withTimeoutMs(ms: number): this { this.timeoutMs = ms; return this; }

  build(): Connection {
    if (!this.host) throw new Error("host is required");
    if (!this.port) throw new Error("port is required");
    return { host: this.host, port: this.port, timeoutMs: this.timeoutMs };
  }
}

interface Connection { host: string; port: number; timeoutMs: number; }
```

## When to Use

- When constructing objects with many optional parameters.
- When the construction process must produce different representations of the same type.
- When you want fluent, readable object construction.

## When NOT to Use

- When the object has few required fields and no optional ones — a plain constructor is simpler.
- When immutability is not critical and a mutable default constructor suffices.

## Anti-Patterns

- Allowing `build()` to silently use defaults for required fields — fail fast with errors.
- Using a builder just to wrap a constructor call with no added value.
- Mutating the builder after `build()` has been called.

## Related Patterns

- [abstract-factory](abstract-factory.md) — creates families of related objects; builder focuses on one complex object.
- [factory-method](factory-method.md) — simpler creation without step-by-step construction.
- [prototype](prototype.md) — clone an existing object instead of building from scratch.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 3.
- Refactoring.Guru — Builder: https://refactoring.guru/design-patterns/builder
- Rust typestate pattern: https://cliffle.com/blog/rust-typestate/
