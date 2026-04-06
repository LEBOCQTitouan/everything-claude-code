---
name: unit-of-work
category: data-access
tags: [data-access, persistence, unit, of, work]
languages: [python, java, csharp]
difficulty: intermediate
---

## Intent

Track all changes made during a business transaction and commit them atomically.

## Problem

Multiple repository operations within a use case may partially succeed, leaving data in an inconsistent state.

## Solution

Wrap repository operations in a unit-of-work that tracks inserts, updates, and deletes. Commit flushes all changes in a single transaction. Rollback discards them.

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
