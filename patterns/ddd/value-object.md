---
name: value-object
category: ddd
tags: [ddd, value-object, immutability, equality]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Model a domain concept that is defined entirely by its attributes, has no identity, and is immutable. Two value objects with the same attributes are considered equal.

## Problem

Primitive types (strings, integers) carry no domain meaning and no validation. A "price" represented as a raw integer can be negative, mixed with unrelated integers, or used in the wrong currency. You need a type that encapsulates domain rules and enforces them at construction.

## Solution

Create a dedicated type for the domain concept. Make it immutable. Define equality based on all attributes. Validate invariants in the constructor, rejecting invalid states. Operations return new instances rather than mutating.

## Language Implementations

### Rust

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Money {
    amount_cents: i64,
    currency: Currency,
}

impl Money {
    pub fn new(amount_cents: i64, currency: Currency) -> Result<Self, MoneyError> {
        if amount_cents < 0 {
            return Err(MoneyError::NegativeAmount);
        }
        Ok(Self { amount_cents, currency })
    }

    pub fn add(&self, other: &Money) -> Result<Money, MoneyError> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch);
        }
        Money::new(self.amount_cents + other.amount_cents, self.currency.clone())
    }
}
// #[derive(Eq, Hash)] enables use as HashMap key
```

### Go

```go
type Money struct {
    amountCents int64
    currency    Currency
}

func NewMoney(cents int64, currency Currency) (Money, error) {
    if cents < 0 {
        return Money{}, ErrNegativeAmount
    }
    return Money{amountCents: cents, currency: currency}, nil
}

func (m Money) Add(other Money) (Money, error) {
    if m.currency != other.currency {
        return Money{}, ErrCurrencyMismatch
    }
    return NewMoney(m.amountCents+other.amountCents, m.currency)
}

// Equality: Go struct comparison works automatically for value types
```

### Python

```python
from dataclasses import dataclass

@dataclass(frozen=True)
class Money:
    amount_cents: int
    currency: str

    def __post_init__(self) -> None:
        if self.amount_cents < 0:
            raise ValueError("amount cannot be negative")

    def add(self, other: "Money") -> "Money":
        if self.currency != other.currency:
            raise ValueError("currency mismatch")
        return Money(self.amount_cents + other.amount_cents, self.currency)
# frozen=True makes it immutable and hashable
```

### Typescript

```typescript
class Money {
  readonly amountCents: number;
  readonly currency: string;

  constructor(amountCents: number, currency: string) {
    if (amountCents < 0) throw new Error("amount cannot be negative");
    this.amountCents = amountCents;
    this.currency = currency;
  }

  add(other: Money): Money {
    if (this.currency !== other.currency) throw new Error("currency mismatch");
    return new Money(this.amountCents + other.amountCents, this.currency);
  }

  equals(other: Money): boolean {
    return this.amountCents === other.amountCents && this.currency === other.currency;
  }
}
```

## When to Use

- For domain concepts defined by their attributes: money, addresses, date ranges, coordinates.
- When you need to eliminate primitive obsession.
- When equality should be based on content, not identity.

## When NOT to Use

- When the concept has a lifecycle and needs to be tracked by identity (use Entity).
- When mutability is required for performance in a hot path (rare).

## Anti-Patterns

- Making value objects mutable -- defeats the purpose of structural equality.
- Using value objects with identity (database primary keys).
- Not validating invariants in the constructor.

## Related Patterns

- [ddd/entity](entity.md) -- has identity; value object does not.
- [ddd/aggregate-root](aggregate-root.md) -- aggregates contain value objects as attributes.
- [ddd/specification](specification.md) -- specifications are often value objects.

## References

- Eric Evans, "Domain-Driven Design", Chapter 5 -- Value Objects.
- Martin Fowler, "ValueObject": https://martinfowler.com/bliki/ValueObject.html
- **Rust**: `#[derive(Eq, Hash, Clone)]` for value semantics
- **Go**: unexported fields + constructor for immutability
- **Python**: `@dataclass(frozen=True)` or `NamedTuple`
- **Kotlin**: `data class` with `val` properties
- **TypeScript**: `readonly` properties + `equals()` method
