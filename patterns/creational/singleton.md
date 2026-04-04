---
name: singleton
category: creational
tags: [creational, global-state, anti-pattern]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Ensure a class has only one instance and provide a global point of access to it.

## Problem

Some resources (configuration registry, connection pool, logger) should have exactly one instance shared across the system. Allowing multiple instances would cause inconsistency or waste resources.

## Solution

Control construction by making the constructor private (or equivalent) and providing a static accessor that returns the single cached instance. In languages without native singleton syntax, use a module-level variable or a lazy initialisation primitive.

## Language Implementations

### Rust

```rust
use std::sync::OnceLock;

struct Config {
    debug: bool,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

fn config() -> &'static Config {
    CONFIG.get_or_init(|| Config { debug: false })
}
```

### Go

```go
import "sync"

type Config struct{ Debug bool }

var (
    instance *Config
    once     sync.Once
)

func GetConfig() *Config {
    once.Do(func() { instance = &Config{Debug: false} })
    return instance
}
```

### Python

```python
class Config:
    _instance: "Config | None" = None

    def __new__(cls) -> "Config":
        if cls._instance is None:
            cls._instance = super().__new__(cls)
            cls._instance.debug = False
        return cls._instance
```

### Typescript

```typescript
class Config {
  private static instance: Config | null = null;
  readonly debug: boolean;

  private constructor() { this.debug = false; }

  static getInstance(): Config {
    if (!Config.instance) Config.instance = new Config();
    return Config.instance;
  }
}
```

## When to Use

- When exactly one object is needed to coordinate actions across the system (e.g., a logger, a metrics registry).
- When controlling concurrent access to a shared resource (combine with proper synchronisation).

## When NOT to Use

- When you can inject the dependency instead — prefer Dependency Injection (DI) over global singletons.
- When unit testing is important — singletons carry state between tests, breaking isolation.
- When you need different configurations in different contexts (e.g., test vs. production).

## Anti-Patterns

**Global singletons harm testability.** When a singleton holds mutable global state, tests cannot run in isolation — one test's side effects bleed into the next. Prefer injecting the shared dependency through constructors or function parameters so that tests can supply a fresh instance.

Avoid:

```rust
// Hidden global dependency — impossible to stub in tests
fn process() {
    let cfg = config(); // uses OnceLock singleton
}
```

Prefer:

```rust
fn process(cfg: &Config) { /* cfg injected, easily replaced in tests */ }
```

## Related Patterns

- [factory-method](factory-method.md) — controls creation without restricting count.
- [prototype](prototype.md) — creates copies; opposite of enforcing a single instance.

## References

- Design Patterns: Elements of Reusable Object-Oriented Software (GoF), Chapter 3.
- Refactoring.Guru — Singleton: https://refactoring.guru/design-patterns/singleton
