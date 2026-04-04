---
name: prototype
category: creational
tags: [creational, cloning, copying]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Specify the kinds of objects to create using a prototypical instance, and create new objects by copying this prototype.

## Problem

Creating an object can be expensive (e.g., involves complex initialisation, database lookups, or deep nesting). You need a way to produce new objects that are copies of an existing instance, possibly with small modifications, without going through the full construction process.

## Solution

Implement a clone or copy operation on the object. Clients create new instances by cloning a prototype rather than calling constructors. The prototype registry pattern allows clients to retrieve and clone pre-configured prototypes by key.

## Language Implementations

### Rust

```rust
#[derive(Debug, Clone)]
struct Config {
    host: String,
    port: u16,
    debug: bool,
}

fn main() {
    let base = Config { host: "localhost".to_string(), port: 8080, debug: false };
    // Clone and override one field
    let debug_config = Config { debug: true, ..base.clone() };
    println!("{base:?}");
    println!("{debug_config:?}");
}
```

### Go

```go
type Config struct {
    Host  string
    Port  int
    Debug bool
}

func (c Config) Clone() Config {
    return Config{Host: c.Host, Port: c.Port, Debug: c.Debug}
}

func main() {
    base := Config{Host: "localhost", Port: 8080, Debug: false}
    debugConfig := base.Clone()
    debugConfig.Debug = true
}
```

### Python

```python
import copy
from dataclasses import dataclass, replace

@dataclass
class Config:
    host: str
    port: int
    debug: bool = False

base = Config(host="localhost", port=8080)
debug_config = replace(base, debug=True)  # shallow copy with override
deep_copy = copy.deepcopy(base)           # deep clone
```

### Typescript

```typescript
interface Config {
  host: string;
  port: number;
  debug: boolean;
}

function cloneConfig(config: Config, overrides: Partial<Config> = {}): Config {
  return { ...config, ...overrides };
}

const base: Config = { host: "localhost", port: 8080, debug: false };
const debugConfig = cloneConfig(base, { debug: true });
```

## When to Use

- When object creation is costly and cloning from an existing instance is cheaper.
- When you want to avoid subclassing just to produce slightly different objects.
- When the exact class to instantiate is determined at runtime.

## When NOT to Use

- When objects contain non-cloneable resources (open file handles, mutexes).
- When a shallow copy would silently share mutable state — ensure deep cloning when needed.

## Anti-Patterns

- Returning a shallow copy when the object contains nested mutable state, leading to aliasing bugs.
- Cloning instead of computing — sometimes it is cheaper to create fresh than to copy.

## Related Patterns

- [factory-method](factory-method.md) — creates new instances from scratch rather than copying.
- [builder](builder.md) — constructs complex objects step by step without cloning.
- [abstract-factory](abstract-factory.md) — manages families of related objects.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 3.
- Refactoring.Guru — Prototype: https://refactoring.guru/design-patterns/prototype
