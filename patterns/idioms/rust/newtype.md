---
name: newtype
category: idioms
tags: [idiom, rust]
languages: [rust]
difficulty: beginner
---

## Intent

Wrap a primitive type in a single-field struct to create a distinct type with zero runtime cost, preventing accidental misuse of semantically different values that share the same underlying representation.

## Problem

Functions accept generic primitives (`u64`, `String`) for conceptually distinct domains. A user ID and an order ID are both `u64`, so the compiler cannot catch accidental swaps. Adding validation or domain meaning to a raw type requires discipline that the type system does not enforce.

## Solution

Define a tuple struct wrapping the inner type. Derive or implement only the traits the domain requires. The compiler treats the newtype as a distinct type, catching misuse at compile time with zero runtime overhead.

## Language Implementations

### Rust

```rust
/// ECC codebase example: ecc-domain uses newtypes like MemoryId, WorktreeId (AC-015.4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemoryId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorktreeId(pub u64);

// Compile error: cannot pass WorktreeId where MemoryId is expected
fn fetch_memory(id: MemoryId) -> Option<String> {
    // ...
    None
}

// Zero-cost: MemoryId has the same layout as u64
impl From<u64> for MemoryId {
    fn from(v: u64) -> Self { Self(v) }
}

// Domain-specific validation in constructor
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meters(f64);

impl Meters {
    pub fn new(value: f64) -> Result<Self, &'static str> {
        if value < 0.0 { return Err("meters cannot be negative"); }
        Ok(Self(value))
    }

    pub fn as_f64(&self) -> f64 { self.0 }
}
```

**ECC codebase usage:** `ecc-domain` defines `MemoryId`, `WorktreeId`, `BacklogId`, and other newtypes to prevent mixing identifiers across bounded contexts. Each newtype derives only the traits required by the domain, keeping the API surface minimal.

## When to Use

- When two or more values share a primitive type but have different semantic meaning.
- When you want to attach validation to construction without runtime wrapper overhead.
- When implementing the orphan rule workaround to add foreign trait impls.

## When NOT to Use

- When the value truly is a generic primitive with no domain meaning.
- When you need transparent interop with C FFI (use `#[repr(transparent)]` if you do).
- When the wrapping adds boilerplate without meaningful type safety.

## Anti-Patterns

- Exposing the inner field as `pub` without validation, defeating the purpose.
- Deriving every trait (`Deref`, `AsRef`, arithmetic ops) making the newtype behave exactly like the inner type.
- Creating newtypes for types already distinct (wrapping another newtype).

## Related Patterns

- [typestate](typestate.md) -- newtypes can serve as state markers in typestate machines.
- [branded-types](../typescript/branded-types.md) -- TypeScript equivalent using intersection with unique symbol.
- [extension-trait](extension-trait.md) -- alternative approach to adding behavior to foreign types.

## References

- The Rust Book, Chapter 19.3 -- Advanced Types, Newtype Pattern: https://doc.rust-lang.org/book/ch19-04-advanced-types.html
- Rust API Guidelines -- Newtype for type safety: https://rust-lang.github.io/api-guidelines/type-safety.html
