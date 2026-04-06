---
name: actor-model
category: concurrency
tags: [concurrency, actors, message-passing, isolation]
languages: [rust, go]
difficulty: advanced
---

## Intent

Encapsulate state within independent actors that communicate exclusively through asynchronous messages, eliminating shared mutable state and enabling location-transparent concurrency.

## Problem

Shared-state concurrency requires careful locking and is prone to deadlocks, priority inversion, and subtle data races. You need isolated units of computation that can scale across cores (or machines) without shared memory.

## Solution

Each actor owns private state, processes one message at a time from its mailbox, and can send messages to other actors or spawn new ones. No locks are needed because state is never shared.

**Language matrix:**

| Language | Support | Library |
|----------|---------|---------|
| Rust | Library-based | actix, ractor, xactor |
| Go | Idiomatic goroutine + channel pattern | No framework needed |

> Python has `pykka` and `thespian` but the GIL limits true parallelism. TypeScript lacks native actor support; use worker_threads with message passing for a similar effect.

## Language Implementations

### Rust

Using a minimal hand-rolled actor with tokio:

```rust
use tokio::sync::mpsc;

enum CounterMsg {
    Increment,
    Get(tokio::sync::oneshot::Sender<u64>),
}

async fn counter_actor(mut rx: mpsc::Receiver<CounterMsg>) {
    let mut count: u64 = 0;
    while let Some(msg) = rx.recv().await {
        match msg {
            CounterMsg::Increment => count += 1,
            CounterMsg::Get(reply) => { let _ = reply.send(count); }
        }
    }
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);
    tokio::spawn(counter_actor(rx));

    tx.send(CounterMsg::Increment).await.unwrap();
    tx.send(CounterMsg::Increment).await.unwrap();

    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    tx.send(CounterMsg::Get(reply_tx)).await.unwrap();
    println!("count = {}", reply_rx.await.unwrap());
}
```

### Go

```go
type CounterMsg struct {
    Increment bool
    Reply     chan uint64
}

func counterActor(mailbox <-chan CounterMsg) {
    var count uint64
    for msg := range mailbox {
        if msg.Increment {
            count++
        }
        if msg.Reply != nil {
            msg.Reply <- count
        }
    }
}

func main() {
    mailbox := make(chan CounterMsg, 32)
    go counterActor(mailbox)

    mailbox <- CounterMsg{Increment: true}
    mailbox <- CounterMsg{Increment: true}

    reply := make(chan uint64, 1)
    mailbox <- CounterMsg{Reply: reply}
    fmt.Println("count =", <-reply)
}
```

## When to Use

- When you need isolated, independently-failing units of work.
- When building distributed systems that may span multiple nodes.
- When state encapsulation and fault isolation are more important than raw throughput.

## When NOT to Use

- For simple request-response flows — async/await is simpler.
- When tight coupling between components requires shared transactional state.
- When latency from message passing is unacceptable (e.g., tight inner loops).

## Anti-Patterns

- Sending mutable references through messages — breaks actor isolation.
- Creating actors for trivial stateless operations — function calls are simpler.
- Unbounded mailboxes causing memory exhaustion under load.

## Related Patterns

- [csp-channels](csp-channels.md) — channels without per-entity encapsulation; actors add identity and state.
- [async-await](async-await.md) — actors often run on an async runtime.
- [fan-out-fan-in](fan-out-fan-in.md) — actors can implement fan-out by spawning child actors.

## References

- Hewitt, C. "A Universal Modular Actor Formalism for Artificial Intelligence" (1973).
- Actix framework: https://actix.rs/
- Go patterns — Actor model: https://www.youtube.com/watch?v=yCbon_9yGVs
