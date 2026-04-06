---
name: domain-event
category: ddd
tags: [ddd, event, decoupling, eventual-consistency]
languages: [rust, go, python, typescript]
difficulty: intermediate
---

## Intent

Capture something that happened in the domain as an explicit, immutable object that other parts of the system can react to, decoupling the trigger from its side effects.

## Problem

When a domain action requires multiple side effects (send email, update analytics, notify another bounded context), embedding all side effects in the action creates tight coupling. Adding a new reaction requires modifying the original code.

## Solution

Define immutable event objects representing domain-significant occurrences. The aggregate records events during its operation. After the transaction commits, an event dispatcher publishes them to registered handlers. Handlers execute side effects independently.

## Language Implementations

### Rust

```rust
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
enum DomainEvent {
    OrderPlaced { order_id: Uuid, total: u64, at: DateTime<Utc> },
    OrderShipped { order_id: Uuid, tracking: String, at: DateTime<Utc> },
}

struct Order {
    id: Uuid,
    events: Vec<DomainEvent>,
    // ... other fields
}

impl Order {
    pub fn place(&mut self) -> Result<(), OrderError> {
        // ... validate and transition state
        self.events.push(DomainEvent::OrderPlaced {
            order_id: self.id,
            total: self.total(),
            at: Utc::now(),
        });
        Ok(())
    }

    pub fn drain_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.events)
    }
}
```

### Go

```go
type DomainEvent interface {
    EventName() string
    OccurredAt() time.Time
}

type OrderPlaced struct {
    OrderID    uuid.UUID
    Total      int64
    OccurredAt time.Time
}

func (e OrderPlaced) EventName() string      { return "order.placed" }
func (e OrderPlaced) OccurredAt() time.Time  { return e.OccurredAt }

type Order struct {
    ID     uuid.UUID
    events []DomainEvent
}

func (o *Order) Place() error {
    o.events = append(o.events, OrderPlaced{
        OrderID: o.ID, Total: o.Total(), OccurredAt: time.Now(),
    })
    return nil
}

func (o *Order) DrainEvents() []DomainEvent {
    events := o.events
    o.events = nil
    return events
}
```

### Python

```python
from dataclasses import dataclass, field
from datetime import datetime, timezone
from uuid import UUID

@dataclass(frozen=True)
class OrderPlaced:
    order_id: UUID
    total: int
    occurred_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))

class Order:
    def __init__(self, id: UUID) -> None:
        self.id = id
        self._events: list = []

    def place(self) -> None:
        self._events.append(OrderPlaced(order_id=self.id, total=self.total))

    def drain_events(self) -> list:
        events = self._events.copy()
        self._events.clear()
        return events
```

### Typescript

```typescript
interface DomainEvent {
  readonly eventName: string;
  readonly occurredAt: Date;
}

interface OrderPlaced extends DomainEvent {
  readonly eventName: "order.placed";
  readonly orderId: string;
  readonly total: number;
}

class Order {
  private events: DomainEvent[] = [];

  place(): void {
    this.events.push({
      eventName: "order.placed",
      orderId: this.id,
      total: this.total,
      occurredAt: new Date(),
    } satisfies OrderPlaced);
  }

  drainEvents(): DomainEvent[] {
    const events = [...this.events];
    this.events = [];
    return events;
  }
}
```

## When to Use

- When side effects must be decoupled from the triggering action.
- When multiple bounded contexts need to react to the same occurrence.
- When you need an audit trail or event sourcing foundation.

## When NOT to Use

- For simple CRUD operations with no cross-cutting side effects.
- When synchronous, tightly-coupled logic is simpler and sufficient.

## Anti-Patterns

- Mutable events that can be changed after creation.
- Publishing events before the transaction commits (observers see uncommitted state).
- Events that contain the entire aggregate state instead of just the change.

## Related Patterns

- [ddd/aggregate-root](aggregate-root.md) -- aggregates produce events.
- [ddd/domain-service](domain-service.md) -- services may coordinate event handling.
- [ddd/anti-corruption-layer](anti-corruption-layer.md) -- translate events at bounded context boundaries.

## References

- Eric Evans, "Domain-Driven Design", Chapter 8 -- Domain Events (added in later editions).
- Martin Fowler, "Domain Event": https://martinfowler.com/eaaDev/DomainEvent.html
- Vaughn Vernon, "Implementing Domain-Driven Design", Chapter 8.
