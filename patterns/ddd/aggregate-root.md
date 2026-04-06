---
name: aggregate-root
category: ddd
tags: [ddd, aggregate, consistency, domain-model]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Define a cluster of domain objects treated as a single unit for data changes, with one root entity controlling all access and enforcing invariants across the cluster.

## Problem

When multiple entities have complex relationships, allowing direct modification of any entity breaks consistency rules. Concurrent modifications to related objects cause race conditions and invalid states. Without a clear transactional boundary, invariants span multiple objects with no single enforcer.

## Solution

Designate one entity as the aggregate root. All external access to the cluster goes through the root. The root enforces all invariants before accepting changes. Persistence operations load and save the entire aggregate atomically. References between aggregates use IDs, not direct object references.

## Language Implementations

### Rust

```rust
use uuid::Uuid;

struct Order {
    id: Uuid,
    items: Vec<OrderItem>,
    status: OrderStatus,
}

impl Order {
    pub fn add_item(&mut self, product_id: Uuid, qty: u32, price: u64) -> Result<(), OrderError> {
        if self.status != OrderStatus::Draft {
            return Err(OrderError::NotDraft);
        }
        if qty == 0 {
            return Err(OrderError::InvalidQuantity);
        }
        let item = OrderItem { product_id, quantity: qty, unit_price: price };
        self.items.push(item);
        Ok(())
    }

    pub fn total(&self) -> u64 {
        self.items.iter().map(|i| i.unit_price * i.quantity as u64).sum()
    }

    pub fn submit(&mut self) -> Result<(), OrderError> {
        if self.items.is_empty() {
            return Err(OrderError::EmptyOrder);
        }
        self.status = OrderStatus::Submitted;
        Ok(())
    }
}

struct OrderItem { product_id: Uuid, quantity: u32, unit_price: u64 }
```

### Go

```go
type Order struct {
    ID     uuid.UUID
    items  []OrderItem
    status OrderStatus
}

func (o *Order) AddItem(productID uuid.UUID, qty int, price int64) error {
    if o.status != Draft {
        return ErrNotDraft
    }
    if qty <= 0 {
        return ErrInvalidQuantity
    }
    o.items = append(o.items, OrderItem{ProductID: productID, Quantity: qty, UnitPrice: price})
    return nil
}

func (o *Order) Submit() error {
    if len(o.items) == 0 {
        return ErrEmptyOrder
    }
    o.status = Submitted
    return nil
}
```

### Python

```python
from dataclasses import dataclass, field
from uuid import UUID

@dataclass
class Order:
    id: UUID
    _items: list["OrderItem"] = field(default_factory=list)
    _status: OrderStatus = OrderStatus.DRAFT

    def add_item(self, product_id: UUID, qty: int, price: int) -> None:
        if self._status != OrderStatus.DRAFT:
            raise OrderError("order is not a draft")
        if qty <= 0:
            raise OrderError("quantity must be positive")
        self._items.append(OrderItem(product_id, qty, price))

    def submit(self) -> None:
        if not self._items:
            raise OrderError("cannot submit empty order")
        self._status = OrderStatus.SUBMITTED
```

### Typescript

```typescript
class Order {
  private items: OrderItem[] = [];
  private status: OrderStatus = "draft";

  constructor(readonly id: string) {}

  addItem(productId: string, qty: number, price: number): void {
    if (this.status !== "draft") throw new Error("order not draft");
    if (qty <= 0) throw new Error("quantity must be positive");
    this.items.push({ productId, quantity: qty, unitPrice: price });
  }

  submit(): void {
    if (this.items.length === 0) throw new Error("cannot submit empty order");
    this.status = "submitted";
  }

  get total(): number {
    return this.items.reduce((sum, i) => sum + i.unitPrice * i.quantity, 0);
  }
}
```

## When to Use

- When a group of entities must maintain consistency invariants together.
- When you need clear transactional boundaries in domain logic.
- When concurrent access requires well-defined locking granularity.

## When NOT to Use

- For simple CRUD operations with no business invariants.
- When entities are independently modifiable with no shared constraints.

## Anti-Patterns

- Aggregates that are too large, locking too many objects together.
- Allowing direct access to child entities, bypassing root invariant checks.
- References between aggregates using direct object references instead of IDs.

## Related Patterns

- [ddd/entity](entity.md) -- the root is itself an entity with identity.
- [ddd/value-object](value-object.md) -- child components are often value objects.
- [ddd/repository](repository.md) -- repositories load and save entire aggregates.
- [ddd/domain-event](domain-event.md) -- aggregates emit events for cross-aggregate communication.

## References

- Eric Evans, "Domain-Driven Design", Chapter 6 -- Aggregates.
- Vaughn Vernon, "Implementing Domain-Driven Design", Chapter 10.
- Vaughn Vernon, "Effective Aggregate Design" (series of 3 essays).
