---
name: entity
category: ddd
tags: [ddd, entity, identity, lifecycle]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Model a domain object that has a unique identity persisting through its lifecycle. Two entities with identical attributes but different identities are distinct objects.

## Problem

Some domain concepts must be tracked across time and state changes. A customer who changes their name is still the same customer. A value-based comparison would treat the renamed customer as a different object, losing continuity.

## Solution

Assign each entity a unique, immutable identity (typically a UUID or domain-specific ID). Define equality and hashing based solely on the identity field. Encapsulate state changes behind methods that enforce invariants.

## Language Implementations

### Rust

```rust
use uuid::Uuid;

#[derive(Debug)]
struct Customer {
    id: Uuid,
    name: String,
    email: String,
}

impl Customer {
    pub fn new(name: String, email: String) -> Self {
        Self { id: Uuid::new_v4(), name, email }
    }

    pub fn rename(&mut self, new_name: String) -> Result<(), DomainError> {
        if new_name.is_empty() {
            return Err(DomainError::EmptyName);
        }
        self.name = new_name;
        Ok(())
    }
}

impl PartialEq for Customer {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl Eq for Customer {}

impl std::hash::Hash for Customer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state); }
}
```

### Go

```go
type Customer struct {
    ID    uuid.UUID
    Name  string
    Email string
}

func NewCustomer(name, email string) Customer {
    return Customer{ID: uuid.New(), Name: name, Email: email}
}

func (c *Customer) Rename(newName string) error {
    if newName == "" {
        return ErrEmptyName
    }
    c.Name = newName
    return nil
}

// Equality by ID: c1.ID == c2.ID
```

### Python

```python
from dataclasses import dataclass, field
from uuid import UUID, uuid4

@dataclass
class Customer:
    name: str
    email: str
    id: UUID = field(default_factory=uuid4)

    def rename(self, new_name: str) -> None:
        if not new_name:
            raise ValueError("name cannot be empty")
        self.name = new_name

    def __eq__(self, other: object) -> bool:
        return isinstance(other, Customer) and self.id == other.id

    def __hash__(self) -> int:
        return hash(self.id)
```

### Typescript

```typescript
class Customer {
  readonly id: string;

  constructor(
    private name: string,
    private email: string,
    id?: string,
  ) {
    this.id = id ?? crypto.randomUUID();
  }

  rename(newName: string): void {
    if (!newName) throw new Error("name cannot be empty");
    this.name = newName;
  }

  equals(other: Customer): boolean {
    return this.id === other.id;
  }
}
```

## When to Use

- When a domain concept has a lifecycle (created, modified, archived).
- When two objects with the same attributes must remain distinct (two customers named "Alice").
- When the object is referenced by other parts of the system by its identity.

## When NOT to Use

- When the concept has no identity or lifecycle (use Value Object).
- For read-only projection or query models where identity tracking adds no value.

## Anti-Patterns

- Defining equality based on all fields instead of identity only.
- Using mutable identity fields.
- Exposing all internal state via public setters, bypassing invariant enforcement.

## Related Patterns

- [ddd/value-object](value-object.md) -- no identity; compared by attributes.
- [ddd/aggregate-root](aggregate-root.md) -- the root entity of an aggregate cluster.
- [ddd/repository](repository.md) -- persist and retrieve entities by identity.

## References

- Eric Evans, "Domain-Driven Design", Chapter 5 -- Entities.
- Vaughn Vernon, "Implementing Domain-Driven Design", Chapter 5.
- **Rust**: `PartialEq`/`Eq`/`Hash` impl on ID field only
- **Go**: compare by ID field; no built-in equality override
- **Python**: `__eq__` and `__hash__` on ID
- **Kotlin**: `data class` with only `id` in primary constructor for correct equality
