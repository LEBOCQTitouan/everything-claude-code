---
name: given-when-then
category: testing
tags: [testing, bdd, gherkin, specification]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Express tests as specifications using Given (preconditions), When (action), Then (expected outcome) to bridge the gap between business requirements and executable tests.

## Problem

Technical test names and assertions are opaque to non-developers. Business rules get lost in implementation details, making it hard to verify that tests actually cover the specified behaviour.

## Solution

Write tests using Given-When-Then language, either as structured comments, BDD framework steps, or descriptive test names. This makes tests readable as living documentation of system behaviour.

## Language Implementations

### Rust

```rust
#[test]
fn given_premium_user_when_checkout_then_free_shipping() {
    // Given
    let user = User::premium("alice");
    let cart = Cart::with_items(vec![Item::new("book", 25)]);

    // When
    let order = checkout(&user, &cart);

    // Then
    assert_eq!(order.shipping_cost(), 0);
}
```

### Go

```go
func TestGivenPremiumUser_WhenCheckout_ThenFreeShipping(t *testing.T) {
    // Given
    user := NewPremiumUser("alice")
    cart := NewCart([]Item{{Name: "book", Price: 25}})

    // When
    order := Checkout(user, cart)

    // Then
    assert.Equal(t, 0, order.ShippingCost())
}
```

### Python

```python
def test_given_premium_user_when_checkout_then_free_shipping():
    # Given
    user = User.premium("alice")
    cart = Cart(items=[Item("book", 25)])

    # When
    order = checkout(user, cart)

    # Then
    assert order.shipping_cost == 0
```

### Typescript

```typescript
describe("checkout", () => {
  it("given premium user, when checkout, then free shipping", () => {
    // Given
    const user = User.premium("alice");
    const cart = new Cart([new Item("book", 25)]);

    // When
    const order = checkout(user, cart);

    // Then
    expect(order.shippingCost).toBe(0);
  });
});
```

## When to Use

- When tests serve as living documentation for business rules.
- When non-technical stakeholders need to understand test coverage.
- When using BDD frameworks (Cucumber, Behave, SpecFlow).

## When NOT to Use

- For low-level unit tests where AAA is more concise.
- When the overhead of Gherkin tooling is not justified by team needs.

## Anti-Patterns

- Writing Given-When-Then steps that are too implementation-specific.
- Overly long Given sections that obscure the intent.
- Using BDD syntax without stakeholder involvement, adding ceremony without value.

## Related Patterns

- [testing/aaa](aaa.md) -- the technical equivalent of the same structure.
- [testing/test-doubles](test-doubles.md) -- fakes and stubs used in Given setup.
- [testing/approval](approval.md) -- capture expected output for Then verification.

## References

- Dan North, "Introducing BDD": https://dannorth.net/introducing-bdd/
- **Rust**: `rstest`, `speculate`
- **Go**: `goconvey`, `goblin`
- **Python**: `pytest-bdd`, `behave`
- **Kotlin**: Kotest `BehaviorSpec`
- **TypeScript**: `jest` describe/it blocks, `cucumber-js`
