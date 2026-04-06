---
name: raii
category: idioms
tags: [idiom, rust]
languages: [rust]
difficulty: intermediate
---

## Intent

Tie resource lifecycle (files, locks, connections, temporary state) to object scope so that cleanup happens automatically and deterministically when the object is dropped, even on early return or panic.

## Problem

Manual resource management requires explicit cleanup calls that are easy to forget, especially on error paths. In languages with `finally` blocks, cleanup is verbose and error-prone. Leaked resources cause file descriptor exhaustion, deadlocks, or data corruption.

## Solution

Implement the `Drop` trait on a guard type that acquires the resource on construction and releases it on drop. Rust's ownership system guarantees `Drop::drop` runs exactly once when the value goes out of scope, including unwinding from panics.

## Language Implementations

### Rust

```rust
use std::fs::File;
use std::io::Write;

// Custom RAII guard for a temporary file
struct TempFile {
    path: std::path::PathBuf,
    file: File,
}

impl TempFile {
    fn new(path: impl Into<std::path::PathBuf>) -> std::io::Result<Self> {
        let path = path.into();
        let file = File::create(&path)?;
        Ok(Self { path, file })
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

// Usage -- file is deleted when guard goes out of scope
fn process() -> std::io::Result<()> {
    let mut tmp = TempFile::new("/tmp/work.dat")?;
    writeln!(tmp.file, "data")?;
    // tmp dropped here -- file is automatically deleted
    Ok(())
}

// Standard library RAII examples:
// - MutexGuard: releases lock on drop
// - File: closes fd on drop
// - Vec: frees heap memory on drop
```

**ECC codebase usage:** Hook scripts in ECC use atomic writes via `mktemp` + `mv` (shell-level RAII). In the Rust layer, `ecc-infra` relies on `Drop`-based guards for temporary file cleanup and POSIX flock release in `ecc-flock`, ensuring locks are freed even if the process panics.

## When to Use

- When a resource must be released deterministically (files, locks, connections, temp dirs).
- When error paths must guarantee cleanup without explicit `finally` blocks.
- When wrapping C/FFI resources that require manual deallocation.

## When NOT to Use

- When the resource has no meaningful cleanup (plain data structs).
- When you need non-deterministic cleanup timing (use `Arc` + weak references).
- When `Drop` ordering between fields matters and is hard to reason about.

## Anti-Patterns

- Calling `std::mem::forget` on RAII guards, preventing cleanup.
- Performing fallible operations in `Drop` (panicking in drop causes abort).
- Creating RAII types for trivial resources where `defer`-style would suffice.

## Related Patterns

- [interior-mutability](interior-mutability.md) -- `MutexGuard` combines RAII with interior mutability.
- [context-managers](../python/context-managers.md) -- Python's `with` statement is the closest equivalent.
- [builder](../../creational/builder.md) -- builders often produce RAII-managed resources.

## References

- The Rust Book, Chapter 15.3 -- Drop Trait: https://doc.rust-lang.org/book/ch15-03-drop.html
- Rust by Example -- RAII: https://doc.rust-lang.org/rust-by-example/scope/raii.html
