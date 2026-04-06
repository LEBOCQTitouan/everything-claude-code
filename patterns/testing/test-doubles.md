---
name: test-doubles
category: testing
tags: [testing, mocking, stubbing, faking, isolation]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Replace real dependencies with controlled substitutes (stubs, mocks, fakes, spies) to isolate the unit under test and make tests fast, deterministic, and focused.

## Problem

Tests that depend on real databases, HTTP services, or file systems are slow, flaky, and hard to set up. They test integration rather than the unit's logic, and failures may come from the dependency rather than the code under test.

## Solution

Use test doubles that implement the same interface as the real dependency. Stubs return canned responses, mocks verify interactions, fakes provide simplified working implementations, and spies record calls for later assertion.

## Language Implementations

### Rust

```rust
// Port trait
trait PriceService: Send + Sync {
    fn get_price(&self, product_id: &str) -> Result<u64, PriceError>;
}

// Stub for testing
struct StubPriceService { price: u64 }

impl PriceService for StubPriceService {
    fn get_price(&self, _id: &str) -> Result<u64, PriceError> {
        Ok(self.price)
    }
}

#[test]
fn test_order_uses_price_service() {
    let service = StubPriceService { price: 42 };
    let order = create_order("widget", &service);
    assert_eq!(order.total(), 42);
}
```

### Go

```go
type PriceService interface {
    GetPrice(productID string) (int64, error)
}

type stubPriceService struct{ price int64 }

func (s *stubPriceService) GetPrice(_ string) (int64, error) {
    return s.price, nil
}

func TestOrderUsesPriceService(t *testing.T) {
    svc := &stubPriceService{price: 42}
    order := CreateOrder("widget", svc)
    assert.Equal(t, int64(42), order.Total())
}
```

### Python

```python
from unittest.mock import Mock

def test_order_uses_price_service():
    service = Mock()
    service.get_price.return_value = 42

    order = create_order("widget", service)

    assert order.total == 42
    service.get_price.assert_called_once_with("widget")
```

### Typescript

```typescript
test("order uses price service", () => {
  const service: PriceService = {
    getPrice: vi.fn().mockReturnValue(42),
  };

  const order = createOrder("widget", service);

  expect(order.total).toBe(42);
  expect(service.getPrice).toHaveBeenCalledWith("widget");
});
```

## When to Use

- When the real dependency is slow, unreliable, or hard to set up.
- When you need deterministic test inputs and outputs.
- When testing error paths that are hard to trigger with real dependencies.

## When NOT to Use

- When you need to verify actual integration behaviour (use integration tests).
- When the double becomes so complex it needs its own tests (use a fake or real dependency).

## Anti-Patterns

- Over-mocking: mocking the class under test rather than its dependencies.
- Brittle mocks that assert on implementation details rather than outcomes.
- Not testing with real implementations at the integration level.

## Related Patterns

- [testing/aaa](aaa.md) -- doubles are set up in the Arrange phase.
- [testing/contract](contract.md) -- verify doubles honour the real contract.
- [testing/testcontainers](testcontainers.md) -- use real dependencies in containers when doubles are insufficient.

## References

- Gerard Meszaros, "xUnit Test Patterns", Chapter 11 -- Test Doubles.
- Martin Fowler, "Mocks Aren't Stubs": https://martinfowler.com/articles/mocksArentStubs.html
- **Rust**: `mockall`, `mockito`, trait-based manual stubs
- **Go**: interface-based stubs, `gomock`, `testify/mock`
- **Python**: `unittest.mock`, `pytest-mock`, `responses`
- **Kotlin**: `MockK`, `mockito-kotlin`
- **TypeScript**: `vitest` vi.fn(), `jest.mock`, `sinon`
