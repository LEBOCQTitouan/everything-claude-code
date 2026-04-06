---
name: aaa
category: testing
tags: [testing, structure, arrange-act-assert]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Structure each test into three distinct phases -- Arrange, Act, Assert -- so that tests are readable, consistent, and easy to maintain.

## Problem

Tests without clear structure become hard to read. Setup, execution, and verification blur together, making it difficult to understand what is being tested, what the inputs are, and what the expected outcome is.

## Solution

Divide every test into three labelled sections: Arrange (set up inputs and dependencies), Act (execute the behaviour under test), and Assert (verify the outcome). Keep each section focused and minimal.

## Language Implementations

### Rust

```rust
#[test]
fn test_discount_applied_to_order() {
    // Arrange
    let items = vec![Item::new("widget", 100)];
    let coupon = Coupon::percentage(10);

    // Act
    let order = Order::new(items).apply_coupon(coupon);

    // Assert
    assert_eq!(order.total(), 90);
}
```

### Go

```go
func TestDiscountAppliedToOrder(t *testing.T) {
    // Arrange
    items := []Item{{Name: "widget", Price: 100}}
    coupon := NewPercentageCoupon(10)

    // Act
    order := NewOrder(items).ApplyCoupon(coupon)

    // Assert
    if order.Total() != 90 {
        t.Errorf("expected 90, got %d", order.Total())
    }
}
```

### Python

```python
def test_discount_applied_to_order():
    # Arrange
    items = [Item("widget", 100)]
    coupon = Coupon.percentage(10)

    # Act
    order = Order(items).apply_coupon(coupon)

    # Assert
    assert order.total == 90
```

### Typescript

```typescript
test("discount applied to order", () => {
  // Arrange
  const items = [new Item("widget", 100)];
  const coupon = Coupon.percentage(10);

  // Act
  const order = new Order(items).applyCoupon(coupon);

  // Assert
  expect(order.total).toBe(90);
});
```

## When to Use

- For every unit test as a default structure.
- When onboarding team members who need consistent test patterns.
- When tests need to be reviewed or maintained by others.

## When NOT to Use

- When the test is so trivial that the three phases collapse to a single line.
- For property-based tests where the framework manages inputs and assertions.

## Anti-Patterns

- Mixing assertions between act steps (assert-act-assert-act).
- Arranging too much -- test setup should be minimal and focused.
- Multiple Act phases in a single test -- split into separate tests.

## Related Patterns

- [testing/given-when-then](given-when-then.md) -- BDD variant of the same three-phase structure.
- [testing/test-doubles](test-doubles.md) -- stubs and mocks used in the Arrange phase.
- [testing/table-driven](table-driven.md) -- parameterise the Arrange phase across multiple inputs.

## References

- Bill Wake, "3A -- Arrange, Act, Assert": https://xp123.com/articles/3a-arrange-act-assert/
- **Rust**: `#[test]`, `rstest` for parameterised AAA
- **Go**: `testing` package, `testify/assert`
- **Python**: `pytest`, `unittest`
- **TypeScript**: `vitest`, `jest`
