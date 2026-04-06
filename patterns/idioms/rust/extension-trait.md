---
name: extension-trait
category: idioms
tags: [idiom, rust]
languages: [rust]
difficulty: intermediate
---

## Intent

Add methods to types you do not own by defining a new trait with a blanket implementation, extending foreign types without modifying their source or using the newtype wrapper.

## Problem

Rust's orphan rule prevents implementing foreign traits on foreign types. You want to add domain-specific convenience methods to standard library or third-party types (e.g., adding `.truncate_to(n)` on `String`) without wrapping them in a newtype.

## Solution

Define a trait with the desired methods and provide a blanket `impl<T>` or a targeted `impl` for the foreign type. Callers import the trait to gain access to the new methods. The original type is unchanged; the extension is opt-in via `use`.

## Language Implementations

### Rust

```rust
// Extension trait for str slices
trait StrExt {
    fn truncate_to(&self, max_len: usize) -> &str;
    fn is_blank(&self) -> bool;
}

impl StrExt for str {
    fn truncate_to(&self, max_len: usize) -> &str {
        if self.len() <= max_len { return self; }
        // Find a valid char boundary
        let mut end = max_len;
        while !self.is_char_boundary(end) { end -= 1; }
        &self[..end]
    }

    fn is_blank(&self) -> bool {
        self.trim().is_empty()
    }
}

// Usage -- import the trait to unlock methods
// use crate::StrExt;
// let s = "hello world".truncate_to(5); // "hello"

// Blanket impl for Iterator extension
trait IterExt: Iterator {
    fn take_until<P>(self, predicate: P) -> TakeUntil<Self, P>
    where
        Self: Sized,
        P: FnMut(&Self::Item) -> bool;
}

impl<I: Iterator> IterExt for I {
    fn take_until<P>(self, predicate: P) -> TakeUntil<Self, P>
    where P: FnMut(&Self::Item) -> bool {
        TakeUntil { iter: self, predicate, done: false }
    }
}

struct TakeUntil<I, P> { iter: I, predicate: P, done: bool }
```

**ECC codebase usage:** `ecc-domain` and `ecc-app` define extension traits to add domain-specific methods to standard types and port trait objects. For example, `ResultExt` provides `.context()` and `.map_domain_err()` to bridge between infrastructure errors and domain error types without coupling to a specific error crate.

## When to Use

- When adding convenience methods to standard library types (`str`, `Vec`, `Iterator`).
- When multiple crates need the same utility methods on a shared type.
- When the orphan rule blocks a direct trait impl.

## When NOT to Use

- When a newtype is more appropriate (you need to restrict the type's API, not extend it).
- When the extension methods are only used in one place (just write a function).
- When the trait would be imported so widely it pollutes every module's namespace.

## Anti-Patterns

- Creating extension traits with too many methods, making them hard to discover.
- Naming extension methods identically to future standard library additions.
- Blanket-implementing for all `T` when only specific types make sense.

## Related Patterns

- [newtype](newtype.md) -- alternative: wrap the type to add methods without traits.
- [decorator](../../structural/decorator.md) -- adds behavior by wrapping; extension traits add behavior by trait import.

## References

- Rust API Guidelines -- Extensions: https://rust-lang.github.io/api-guidelines/naming.html#c-ext
- itertools crate -- canonical extension trait example: https://docs.rs/itertools
