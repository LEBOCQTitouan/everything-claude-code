---
name: csp-channels
category: concurrency
tags: [concurrency, channels, message-passing, csp]
languages: [rust, go]
difficulty: intermediate
---

## Intent

Coordinate concurrent processes by communicating through typed channels rather than sharing memory. Based on Hoare's Communicating Sequential Processes (CSP) model.

## Problem

Shared mutable state requires locks, which are error-prone (deadlocks, data races, forgotten unlocks). You need a way for concurrent tasks to exchange data safely without sharing memory.

## Solution

Use channels — typed, optionally buffered conduits — to send messages between concurrent tasks. Each task owns its local state and communicates only through channel send/receive operations.

**Language matrix:**

| Language | Support | Channel type |
|----------|---------|-------------|
| Rust | `std::sync::mpsc`, `tokio::sync::mpsc`, crossbeam | Library (std + ecosystem) |
| Go | `chan` keyword | Native language primitive |

> Python and TypeScript lack native CSP channels. Python has `asyncio.Queue` (similar but not CSP); TypeScript has no built-in equivalent.

## Language Implementations

### Rust

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();

    thread::spawn(move || { tx.send("from worker 1").unwrap(); });
    thread::spawn(move || { tx2.send("from worker 2").unwrap(); });

    for msg in rx.iter().take(2) {
        println!("received: {msg}");
    }
}
```

Async variant with tokio:

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let tx2 = tx.clone();

    tokio::spawn(async move { tx.send("async worker 1").await.unwrap(); });
    tokio::spawn(async move { tx2.send("async worker 2").await.unwrap(); });

    while let Some(msg) = rx.recv().await {
        println!("received: {msg}");
    }
}
```

### Go

```go
func main() {
    ch := make(chan string, 2)

    go func() { ch <- "from worker 1" }()
    go func() { ch <- "from worker 2" }()

    fmt.Println(<-ch)
    fmt.Println(<-ch)
}
```

Select for multiplexing:

```go
select {
case msg := <-ch1:
    fmt.Println("ch1:", msg)
case msg := <-ch2:
    fmt.Println("ch2:", msg)
case <-time.After(5 * time.Second):
    fmt.Println("timeout")
}
```

## When to Use

- When multiple goroutines/threads need to coordinate without shared state.
- When implementing producer-consumer, pipeline, or fan-out/fan-in patterns.
- When you want to enforce ownership transfer of data between tasks.

## When NOT to Use

- When tasks are independent and do not need to communicate — channels add unnecessary overhead.
- When a simple mutex suffices for infrequent shared access.

## Anti-Patterns

- Sending pointers through channels while retaining a reference — defeats the ownership-transfer guarantee.
- Using unbuffered channels when producers and consumers run at different speeds (causes blocking).
- Forgetting to close channels, causing goroutine leaks in Go.

## Related Patterns

- [actor-model](actor-model.md) — actors also use message passing but with mailboxes and per-actor state.
- [fan-out-fan-in](fan-out-fan-in.md) — structured use of channels for parallel work distribution.
- [async-await](async-await.md) — often combined with channels in Rust's async ecosystem.

## References

- Hoare, C.A.R. "Communicating Sequential Processes" (1978).
- Go blog — Share Memory By Communicating: https://go.dev/blog/codelab-share
- Rust std::sync::mpsc: https://doc.rust-lang.org/std/sync/mpsc/
