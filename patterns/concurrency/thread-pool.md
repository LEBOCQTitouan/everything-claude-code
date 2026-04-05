---
name: thread-pool
category: concurrency
tags: [concurrency, parallelism, thread-pool, cpu-bound]
languages: [rust, go, python]
difficulty: intermediate
---

## Intent

Amortize thread creation cost by maintaining a pool of reusable worker threads that execute submitted tasks, enabling bounded parallelism for CPU-bound or blocking work.

## Problem

Creating a new OS thread per task is expensive and unbounded — under high load, the system runs out of resources. You need a fixed set of threads that process a queue of work items.

## Solution

Maintain a fixed-size pool of worker threads. Submit tasks to a shared queue; idle workers pick up the next task. The pool bounds resource consumption and amortizes thread creation overhead.

**Language matrix:**

| Language | Support | Mechanism |
|----------|---------|-----------|
| Rust | `rayon`, `threadpool`, `tokio::task::spawn_blocking` | Library |
| Go | Goroutine pool (bounded via semaphore) | Idiomatic pattern |
| Python | `concurrent.futures.ThreadPoolExecutor` | Stdlib |

> **TypeScript/Node.js**: N/A — JavaScript is single-threaded. For CPU-bound parallelism, use the `worker_threads` module, which spawns separate V8 isolates (not a traditional thread pool). Each worker has its own event loop and heap; communication is via `postMessage`.

## Language Implementations

### Rust

Using rayon for data parallelism:

```rust
use rayon::prelude::*;

fn main() {
    let numbers: Vec<u64> = (0..1_000_000).collect();
    let sum: u64 = numbers.par_iter().map(|n| n * n).sum();
    println!("sum of squares = {sum}");
}
```

Using `spawn_blocking` for blocking I/O in async context:

```rust
let result = tokio::task::spawn_blocking(|| {
    std::fs::read_to_string("/etc/hosts")
}).await?;
```

### Go

Bounded worker pool via semaphore channel:

```go
func workerPool(jobs <-chan int, results chan<- int, poolSize int) {
    sem := make(chan struct{}, poolSize)
    var wg sync.WaitGroup
    for job := range jobs {
        wg.Add(1)
        sem <- struct{}{}
        go func(j int) {
            defer func() { <-sem; wg.Done() }()
            results <- j * j
        }(job)
    }
    wg.Wait()
    close(results)
}
```

### Python

```python
from concurrent.futures import ThreadPoolExecutor

def compute_square(n: int) -> int:
    return n * n

with ThreadPoolExecutor(max_workers=8) as pool:
    results = list(pool.map(compute_square, range(1_000_000)))
```

> For CPU-bound Python work, prefer `ProcessPoolExecutor` to bypass the GIL.

## When to Use

- For CPU-bound computation that benefits from multi-core parallelism.
- For blocking I/O operations within an async runtime.
- When you need bounded concurrency to prevent resource exhaustion.

## When NOT to Use

- For I/O-bound work — async/await is more efficient (no thread per task).
- When tasks are trivially fast — thread overhead dominates.
- In single-threaded runtimes without worker thread support.

## Anti-Patterns

- Unbounded pool size — defeats the purpose; can exhaust OS threads.
- Sharing mutable state between pool tasks without synchronization.
- Submitting blocking tasks to an async runtime's cooperative thread pool.

## Related Patterns

- [async-await](async-await.md) — preferred for I/O-bound concurrency.
- [fan-out-fan-in](fan-out-fan-in.md) — distributes work across pool workers and collects results.
- [read-write-lock](read-write-lock.md) — synchronizes shared state accessed by pool workers.

## References

- Rayon crate: https://docs.rs/rayon/
- Python ThreadPoolExecutor: https://docs.python.org/3/library/concurrent.futures.html
- Go concurrency patterns: https://go.dev/blog/pipelines
