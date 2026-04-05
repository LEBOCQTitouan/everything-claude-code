---
name: async-await
category: concurrency
tags: [concurrency, async, non-blocking, cooperative]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Execute non-blocking operations cooperatively, allowing a single thread (or a small thread pool) to multiplex many concurrent tasks by suspending and resuming at await points.

## Problem

Blocking I/O wastes threads — each waiting operation holds a thread hostage. Spawning one thread per connection does not scale. You need a way to express concurrent I/O without dedicating a thread to each operation.

## Solution

Use language-level async/await syntax to write sequential-looking code that compiles to state machines. A runtime polls these state machines to completion, multiplexing many tasks onto few threads.

**Language matrix:**

| Language | Native support | Runtime |
|----------|---------------|---------|
| Rust | `async fn` / `.await` (language) | tokio, async-std (library) |
| Go | Goroutines + channels (native, implicit) | Built-in scheduler |
| Python | `async def` / `await` (language) | asyncio (stdlib) |
| TypeScript | `async` / `await` (language) | Event loop (V8/Node) |

> Go does not use explicit async/await syntax — goroutines are implicitly async. Included here because the concurrency model solves the same problem.

## Language Implementations

### Rust

```rust
use tokio::time::{sleep, Duration};

async fn fetch_data(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::get(url).await?;
    response.text().await
}

#[tokio::main]
async fn main() {
    let (a, b) = tokio::join!(
        fetch_data("https://api.example.com/a"),
        fetch_data("https://api.example.com/b"),
    );
    println!("a={:?}, b={:?}", a, b);
}
```

### Go

```go
func fetchData(ctx context.Context, url string) (string, error) {
    req, err := http.NewRequestWithContext(ctx, "GET", url, nil)
    if err != nil { return "", err }
    resp, err := http.DefaultClient.Do(req)
    if err != nil { return "", err }
    defer resp.Body.Close()
    body, err := io.ReadAll(resp.Body)
    return string(body), err
}

// Concurrent fetch via goroutines — no async/await keyword needed
func main() {
    ctx := context.Background()
    ch := make(chan string, 2)
    go func() { s, _ := fetchData(ctx, "https://api.example.com/a"); ch <- s }()
    go func() { s, _ := fetchData(ctx, "https://api.example.com/b"); ch <- s }()
    fmt.Println(<-ch, <-ch)
}
```

### Python

```python
import asyncio
import aiohttp

async def fetch_data(url: str) -> str:
    async with aiohttp.ClientSession() as session:
        async with session.get(url) as resp:
            return await resp.text()

async def main() -> None:
    a, b = await asyncio.gather(
        fetch_data("https://api.example.com/a"),
        fetch_data("https://api.example.com/b"),
    )
    print(f"a={a}, b={b}")

asyncio.run(main())
```

### TypeScript

```typescript
async function fetchData(url: string): Promise<string> {
  const response = await fetch(url);
  return response.text();
}

async function main(): Promise<void> {
  const [a, b] = await Promise.all([
    fetchData("https://api.example.com/a"),
    fetchData("https://api.example.com/b"),
  ]);
  console.log(`a=${a}, b=${b}`);
}
```

## When to Use

- When performing I/O-bound work (network, file system, database).
- When you need thousands of concurrent tasks without thousands of threads.
- When sequential-looking code is preferred over callback spaghetti.

## When NOT to Use

- For CPU-bound computation — async adds overhead without benefit; use thread pools instead.
- When the runtime/ecosystem lacks async support for your dependencies.

## Anti-Patterns

- Blocking inside an async context (e.g., `std::thread::sleep` in a tokio task).
- Spawning an async runtime per call instead of sharing one.
- Ignoring cancellation — dropping a future in Rust cancels it; Python tasks need explicit cancellation.

## Related Patterns

- [csp-channels](csp-channels.md) — Go's goroutine model is CSP-based; channels complement async.
- [fan-out-fan-in](fan-out-fan-in.md) — structured concurrency over multiple async tasks.
- [thread-pool](thread-pool.md) — better fit for CPU-bound parallelism.

## References

- Rust async book: https://rust-lang.github.io/async-book/
- Python asyncio docs: https://docs.python.org/3/library/asyncio.html
- MDN async/await: https://developer.mozilla.org/en-US/docs/Learn/JavaScript/Asynchronous/Promises
