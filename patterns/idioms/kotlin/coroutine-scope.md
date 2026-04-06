---
name: coroutine-scope
category: idioms
tags: [idiom, kotlin, coroutines, structured-concurrency]
languages: [kotlin]
difficulty: intermediate
---

## Intent

Use structured concurrency via CoroutineScope to ensure child coroutines are bounded to a lifecycle and automatically cancelled when the scope ends.

## Problem

Launching coroutines without a scope creates fire-and-forget tasks that leak resources, swallow exceptions, and outlive their intended lifecycle.

## Solution

Always launch coroutines within a CoroutineScope. Use coroutineScope {} for sequential decomposition, supervisorScope {} when child failures should not cancel siblings.

## Language Implementations

### Kotlin
```kotlin
suspend fun fetchUserData(userId: String): UserData = coroutineScope {
    val profile = async { api.getProfile(userId) }
    val orders = async { api.getOrders(userId) }
    UserData(profile.await(), orders.await())
}
// If either async fails, the other is automatically cancelled
```

## When to Use

- Any concurrent operation in Kotlin (Android, backend, KMP).
- Parallel data fetching with automatic cancellation.

## When NOT to Use

- Simple sequential code with no concurrency.
- GlobalScope is almost never correct.

## Anti-Patterns

- Using GlobalScope (leaks coroutines, no lifecycle binding).
- Catching CancellationException (breaks structured concurrency).

## Related Patterns

- concurrency/async-await
- concurrency/fan-out-fan-in

## References

- Kotlin Coroutines documentation.
- Roman Elizarov, Structured Concurrency talk.
