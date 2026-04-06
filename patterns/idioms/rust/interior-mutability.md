---
name: interior-mutability
category: idioms
tags: [idiom, rust]
languages: [rust]
difficulty: advanced
---

## Intent

Allow mutation of data behind a shared (`&T`) reference by moving the borrow check from compile time to runtime, enabling patterns that the standard ownership model cannot express.

## Problem

Rust's borrow checker enforces exclusive access for mutation at compile time. Some patterns -- caches, reference-counted graphs, shared configuration -- require mutation through shared references. Without interior mutability, these patterns require unsafe code or awkward architectural workarounds.

## Solution

Use standard library types that provide safe interior mutability: `Cell<T>` for `Copy` types (zero overhead), `RefCell<T>` for runtime-checked borrows (single-threaded), or `Mutex<T>` / `RwLock<T>` for thread-safe mutation. Each moves the borrow invariant from compile time to runtime with appropriate trade-offs.

## Language Implementations

### Rust

```rust
use std::cell::{Cell, RefCell};

// Cell -- zero-overhead for Copy types
struct Counter {
    count: Cell<u32>,
}

impl Counter {
    fn new() -> Self { Self { count: Cell::new(0) } }
    fn increment(&self) { self.count.set(self.count.get() + 1); }
    fn value(&self) -> u32 { self.count.get() }
}

// RefCell -- runtime borrow checking for non-Copy types
struct Cache {
    entries: RefCell<Vec<String>>,
}

impl Cache {
    fn new() -> Self { Self { entries: RefCell::new(Vec::new()) } }

    fn get_or_insert(&self, key: &str) -> String {
        let entries = self.entries.borrow();
        if let Some(e) = entries.iter().find(|e| e.as_str() == key) {
            return e.clone();
        }
        drop(entries); // release immutable borrow before mutable borrow
        self.entries.borrow_mut().push(key.to_string());
        key.to_string()
    }
}

// Mutex -- thread-safe interior mutability
use std::sync::Mutex;
let shared = Mutex::new(Vec::new());
shared.lock().unwrap().push(42);
```

**ECC codebase usage:** `ecc-domain` avoids interior mutability in the pure domain layer (no I/O, no mutation). The infra layer uses `Mutex` for thread-safe shared state in test harnesses and `RefCell` for in-memory port implementations during testing.

## When to Use

- When implementing caches, memoization, or lazy initialization behind shared references.
- When working with callback-heavy APIs that require `&self` receivers.
- When shared ownership (`Rc<RefCell<T>>`) is needed for graph-like structures.

## When NOT to Use

- When restructuring ownership can avoid the need entirely (prefer compile-time borrows).
- When `Cell`/`RefCell` would be shared across threads (use `Mutex`/`RwLock` instead).
- When performance-critical code cannot tolerate runtime borrow panics.

## Anti-Patterns

- Using `RefCell` everywhere to "fight the borrow checker" instead of fixing ownership.
- Holding `RefCell` borrows across `.await` points, causing runtime panics.
- Nesting `RefCell` inside `RefCell`, creating confusing borrow hierarchies.

## Related Patterns

- [raii](raii.md) -- `MutexGuard` is an RAII pattern ensuring lock release.
- [newtype](newtype.md) -- wrapping `RefCell` in a newtype to restrict the mutation API.

## References

- The Rust Book, Chapter 15.5 -- Interior Mutability: https://doc.rust-lang.org/book/ch15-05-interior-mutability.html
- Rust Reference -- Cell and RefCell: https://doc.rust-lang.org/std/cell/index.html
