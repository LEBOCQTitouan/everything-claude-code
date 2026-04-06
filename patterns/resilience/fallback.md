---
name: fallback
category: resilience
tags: [resilience, graceful-degradation, fallback]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Provide a degraded but acceptable response when the primary operation fails, keeping the system functional rather than returning an error to the user.

## Problem

When a dependency fails, returning a raw error to the user degrades the experience and may break downstream consumers. Many scenarios have acceptable fallback values: cached data, default configurations, simplified responses, or alternative data sources.

## Solution

Wrap the primary operation with a fallback handler that catches failures and returns a pre-defined alternative. Chain multiple fallback levels (primary, cache, default) for defence in depth.

## Language Implementations

### Rust

```rust
async fn get_price(product_id: &str) -> Result<Price, PriceError> {
    match live_price_service(product_id).await {
        Ok(price) => Ok(price),
        Err(_) => match cached_price(product_id).await {
            Some(cached) => Ok(cached),
            None => Ok(Price::default()),
        },
    }
}
```

### Go

```go
func GetPrice(ctx context.Context, productID string) (Price, error) {
    price, err := livePriceService(ctx, productID)
    if err == nil {
        return price, nil
    }
    if cached, ok := cachedPrice(productID); ok {
        return cached, nil
    }
    return DefaultPrice(), nil
}
```

### Python

```python
async def get_price(product_id: str) -> Price:
    try:
        return await live_price_service(product_id)
    except ServiceError:
        cached = await cached_price(product_id)
        if cached is not None:
            return cached
        return Price.default()
```

### Typescript

```typescript
async function getPrice(productId: string): Promise<Price> {
  try {
    return await livePriceService(productId);
  } catch {
    const cached = await cachedPrice(productId);
    if (cached) return cached;
    return DEFAULT_PRICE;
  }
}
```

## When to Use

- When stale or approximate data is better than no data.
- When you can pre-compute or cache fallback values.
- When different failure modes warrant different fallback strategies.

## When NOT to Use

- When the operation must succeed with fresh data (financial transactions, writes).
- When a fallback would silently hide a critical failure that needs immediate attention.
- When there is no meaningful degraded response.

## Anti-Patterns

- Using a fallback as a substitute for fixing the root cause of failures.
- Returning stale data without indicating to the caller that it is stale.
- Chaining so many fallbacks that failures become invisible to monitoring.

## Related Patterns

- [resilience/circuit-breaker](circuit-breaker.md) -- trigger fallback when the circuit is open.
- [resilience/retry-backoff](retry-backoff.md) -- attempt retries before falling back.
- [resilience/timeout](timeout.md) -- trigger fallback when timeout expires.

## References

- Michael Nygard, "Release It!", Chapter 5 -- Steady State.
- **Rust**: `Result::unwrap_or_else`, `Option::unwrap_or_default`
- **Go**: explicit error checking with fallback branches
- **Python**: try/except with fallback logic
- **Java/Kotlin**: Resilience4j Fallback decorator
- **TypeScript**: `Promise.catch` chains, `cockatiel` fallback policy
