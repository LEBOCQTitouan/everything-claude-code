---
name: fan-out-fan-in
category: concurrency
tags: [concurrency, parallelism, pipeline, work-distribution]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Distribute work across multiple concurrent workers (fan-out) and collect their results into a single stream (fan-in), enabling parallel processing of independent tasks with bounded concurrency.

## Problem

Processing a large batch of independent items sequentially is slow. You need to parallelize the work while collecting results in a controlled manner and limiting resource consumption.

## Solution

Fan-out: dispatch items to N concurrent workers. Fan-in: merge worker outputs into a single channel or collection. A concurrency limiter (semaphore, buffered channel, or pool) bounds the number of in-flight workers.

**Language matrix:**

| Language | Fan-out mechanism | Fan-in mechanism |
|----------|------------------|-----------------|
| Rust | `tokio::spawn` / `rayon::par_iter` | `JoinSet`, `FuturesUnordered`, channel |
| Go | Goroutines | Channel merge, `sync.WaitGroup` |
| Python | `asyncio.gather` / `ThreadPoolExecutor` | Awaited list, `as_completed` |
| TypeScript | `Promise.all` / `Promise.allSettled` | Array of resolved promises |

## Language Implementations

### Rust

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

async fn process(id: u32) -> u32 { id * 2 }

#[tokio::main]
async fn main() {
    let sem = Arc::new(Semaphore::new(4)); // max 4 concurrent
    let mut set = tokio::task::JoinSet::new();

    for id in 0..20 {
        let permit = sem.clone().acquire_owned().await.unwrap();
        set.spawn(async move {
            let result = process(id).await;
            drop(permit);
            result
        });
    }

    let mut results = Vec::new();
    while let Some(Ok(val)) = set.join_next().await {
        results.push(val);
    }
    println!("results: {results:?}");
}
```

### Go

```go
func fanOutFanIn(items []int, workers int) []int {
    jobs := make(chan int, len(items))
    results := make(chan int, len(items))

    var wg sync.WaitGroup
    for w := 0; w < workers; w++ {
        wg.Add(1)
        go func() {
            defer wg.Done()
            for id := range jobs {
                results <- id * 2
            }
        }()
    }

    for _, item := range items {
        jobs <- item
    }
    close(jobs)

    go func() { wg.Wait(); close(results) }()

    var out []int
    for r := range results {
        out = append(out, r)
    }
    return out
}
```

### Python

```python
import asyncio

async def process(item: int) -> int:
    await asyncio.sleep(0.01)  # simulate I/O
    return item * 2

async def fan_out_fan_in(items: list[int], max_concurrent: int = 4) -> list[int]:
    sem = asyncio.Semaphore(max_concurrent)

    async def bounded(item: int) -> int:
        async with sem:
            return await process(item)

    return await asyncio.gather(*(bounded(i) for i in items))
```

### TypeScript

```typescript
async function process(id: number): Promise<number> {
  return id * 2;
}

async function fanOutFanIn(items: number[], maxConcurrent = 4): Promise<number[]> {
  const results: number[] = [];
  for (let i = 0; i < items.length; i += maxConcurrent) {
    const batch = items.slice(i, i + maxConcurrent);
    const batchResults = await Promise.all(batch.map(process));
    results.push(...batchResults);
  }
  return results;
}
```

## When to Use

- When processing a batch of independent items that can run in parallel.
- When you need bounded concurrency to avoid resource exhaustion.
- When building data pipelines with parallel processing stages.

## When NOT to Use

- When items have dependencies on each other — use a DAG scheduler instead.
- When the batch is small enough that sequential processing is fast enough.

## Anti-Patterns

- Unbounded fan-out — spawning thousands of tasks without a concurrency limit.
- Ignoring partial failures — one failed worker should not silently drop results.
- Ordering assumptions — fan-in results may arrive in any order.

## Related Patterns

- [thread-pool](thread-pool.md) — fan-out often uses a thread pool as its execution backend.
- [csp-channels](csp-channels.md) — channels are the natural fan-in merge point.
- [async-await](async-await.md) — fan-out/fan-in is typically built on async primitives.

## References

- Go blog — Pipelines and cancellation: https://go.dev/blog/pipelines
- Tokio JoinSet: https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html
- Python asyncio.gather: https://docs.python.org/3/library/asyncio-task.html#asyncio.gather
