---
name: query-builder
category: data-access
tags: [data-access, persistence, query, builder]
languages: [rust, go, python, typescript, java]
difficulty: intermediate
---

## Intent

Construct database queries programmatically using a fluent API instead of raw SQL strings.

## Problem

Raw SQL strings are error-prone, hard to compose, and vulnerable to injection when concatenated with user input.

## Solution

Provide a builder API that constructs queries via method chaining. The builder handles escaping, parameterization, and dialect differences.

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
