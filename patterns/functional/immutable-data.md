---
name: immutable-data
category: functional
tags: [functional, immutability, pure, value-objects]
languages: [rust, python, typescript, haskell, scala]
difficulty: beginner
---

## Intent

Prevent hidden side effects by ensuring data structures cannot be modified after creation, making code easier to reason about, test, and parallelize.

## Problem

Mutable data creates hidden coupling — one function modifies a shared object, breaking another function's assumptions. Debugging requires tracking every mutation across the codebase. Concurrent access to mutable data requires synchronization. You need data that is stable once created.

## Solution

Create data structures that cannot be changed after construction. To "update" data, create a new copy with the desired changes. Languages provide various mechanisms: frozen dataclasses, readonly types, ownership semantics, or inherent immutability.

## Language Implementations

**Relevance**: Rust (native — ownership), Python (native — frozen dataclasses), Typescript (native — readonly), Haskell (native — all data immutable), Scala (native — val + case class)

### Rust

```rust
#[derive(Clone)]
struct Config {
    host: String,
    port: u16,
    timeout_ms: u64,
}

impl Config {
    fn with_port(&self, port: u16) -> Self {
        Self { port, ..self.clone() }
    }

    fn with_timeout(&self, timeout_ms: u64) -> Self {
        Self { timeout_ms, ..self.clone() }
    }
}

// Usage: let prod = base_config.with_port(443).with_timeout(30_000);
```

### Python

```python
from dataclasses import dataclass, replace

@dataclass(frozen=True)
class Config:
    host: str
    port: int
    timeout_ms: int = 5000

# "Update" creates a new instance:
base = Config(host="localhost", port=8080)
prod = replace(base, port=443, timeout_ms=30_000)
```

### Typescript

```typescript
interface Config {
  readonly host: string;
  readonly port: number;
  readonly timeoutMs: number;
}

function withPort(config: Config, port: number): Config {
  return { ...config, port };
}

function withTimeout(config: Config, timeoutMs: number): Config {
  return { ...config, timeoutMs };
}
```

### Haskell

```haskell
data Config = Config
  { host      :: String
  , port      :: Int
  , timeoutMs :: Int
  } deriving (Show)

withPort :: Int -> Config -> Config
withPort p cfg = cfg { port = p }
```

## When to Use

- Always, as the default approach. Mutability should be the exception.
- When data is shared across functions, threads, or modules.
- When debugging is easier with stable, traceable values.

## When NOT to Use

- When performance-critical inner loops require in-place mutation.
- When the data structure is large and copying is prohibitively expensive.
- When the language makes immutability excessively verbose.

## Anti-Patterns

- Using "defensive copies" everywhere instead of structural sharing.
- Making data immutable at the surface but storing mutable internals.
- Ignoring immutability in concurrent code, relying on locks instead.

## Related Patterns

- [functional/lenses](lenses.md) — update deeply nested immutable data ergonomically.
- [functional/adts](adts.md) — algebraic data types are naturally immutable.
- [functional/map-filter-reduce](map-filter-reduce.md) — operations that preserve immutability.

## References

- ECC Coding Style — Immutability (CRITICAL): `rules/common/coding-style.md`
- Rust Ownership: https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html
- Rich Hickey — The Value of Values: https://www.infoq.com/presentations/Value-Values/
