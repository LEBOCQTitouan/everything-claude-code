---
name: read-write-lock
category: concurrency
tags: [concurrency, synchronization, rwlock, shared-state]
languages: [rust, go, python]
difficulty: intermediate
---

## Intent

Allow multiple concurrent readers OR a single exclusive writer to access shared data, maximizing read throughput while maintaining write safety.

## Problem

A simple mutex serializes all access — even concurrent reads that do not conflict. When reads vastly outnumber writes, this creates unnecessary contention and reduces throughput.

## Solution

Use a read-write lock (RwLock) that distinguishes between shared read access and exclusive write access. Multiple readers can hold the lock simultaneously; a writer waits for all readers to release and then holds exclusive access.

**Language matrix:**

| Language | Support | Type |
|----------|---------|------|
| Rust | `std::sync::RwLock`, `tokio::sync::RwLock`, `parking_lot::RwLock` | Stdlib + library |
| Go | `sync.RWMutex` | Stdlib |
| Python | `threading.Lock` (no stdlib RwLock); `readerwriterlock` package | Library |

> **TypeScript/Node.js**: N/A — JavaScript is single-threaded; there is no shared memory between concurrent operations in the main thread. For `SharedArrayBuffer` in worker threads, use `Atomics` for low-level synchronization.

## Language Implementations

### Rust

```rust
use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let cache = Arc::new(RwLock::new(vec![1, 2, 3]));

    // Spawn multiple readers
    let handles: Vec<_> = (0..4).map(|i| {
        let cache = Arc::clone(&cache);
        thread::spawn(move || {
            let data = cache.read().unwrap();
            println!("reader {i}: {:?}", *data);
        })
    }).collect();

    // Single writer
    {
        let mut data = cache.write().unwrap();
        data.push(4);
        println!("writer: appended 4");
    }

    for h in handles { h.join().unwrap(); }
}
```

### Go

```go
type Cache struct {
    mu   sync.RWMutex
    data []int
}

func (c *Cache) Read() []int {
    c.mu.RLock()
    defer c.mu.RUnlock()
    result := make([]int, len(c.data))
    copy(result, c.data)
    return result
}

func (c *Cache) Write(val int) {
    c.mu.Lock()
    defer c.mu.Unlock()
    c.data = append(c.data, val)
}
```

### Python

```python
from readerwriterlock import rwlock

marker = rwlock.RWLockFairD()
cache: list[int] = [1, 2, 3]

# Reader
with marker.gen_rlock():
    print(f"read: {cache}")

# Writer
with marker.gen_wlock():
    cache = [*cache, 4]  # immutable append
    print(f"write: {cache}")
```

## When to Use

- When reads vastly outnumber writes (e.g., configuration caches, lookup tables).
- When read operations are expensive enough that serialization causes measurable latency.
- When data consistency is required but exclusive locking is too coarse.

## When NOT to Use

- When reads and writes are equally frequent — a simple mutex has less overhead.
- When the critical section is very short — RwLock overhead may exceed mutex overhead.
- When you can use lock-free data structures (e.g., `arc-swap` in Rust).

## Anti-Patterns

- Writer starvation — continuous readers prevent writers from ever acquiring the lock.
- Holding read locks for extended periods, blocking pending writers.
- Upgrading a read lock to a write lock (causes deadlock in most implementations).

## Related Patterns

- [thread-pool](thread-pool.md) — pool workers often need synchronized access to shared state.
- [actor-model](actor-model.md) — eliminates the need for locks by isolating state.
- [csp-channels](csp-channels.md) — alternative to shared state via message passing.

## References

- Rust std::sync::RwLock: https://doc.rust-lang.org/std/sync/struct.RwLock.html
- Go sync.RWMutex: https://pkg.go.dev/sync#RWMutex
- readerwriterlock (Python): https://pypi.org/project/readerwriterlock/
