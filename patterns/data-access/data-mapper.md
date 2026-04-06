---
name: data-mapper
category: data-access
tags: [data-access, persistence, data, mapper]
languages: [rust, go, java, typescript]
difficulty: intermediate
---

## Intent

Separate domain objects from database schema, mapping between them explicitly.

## Problem

Domain objects become polluted with persistence concerns (annotations, column mappings), violating separation of concerns.

## Solution

A mapper layer translates between domain entities (pure logic) and database records (persistence). Neither side knows about the other.

## Language Implementations

### Rust
```rust
// sqlx / diesel / sea-orm
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", id)
    .fetch_optional(&pool).await?;
```

### Python
```python
# SQLAlchemy / Django ORM
user = session.query(User).filter_by(id=id).first()
```

### Java
```java
// Hibernate / JPA / Spring Data
Optional<User> user = userRepository.findById(id);
```

## When to Use

- Applications with database persistence requirements.
- Systems needing testable data access layers.

## When NOT to Use

- In-memory-only applications or CLI tools without persistence.
- Prototypes where direct SQL is faster to write.

## Anti-Patterns

- Exposing ORM internals in domain layer.
- N+1 query problems from lazy loading.

## Related Patterns

- data-access/repository
- ddd/aggregate-root

## References

- Martin Fowler, Patterns of Enterprise Application Architecture.
- sqlx (Rust). SQLAlchemy (Python). Hibernate (Java). Prisma (TypeScript).
