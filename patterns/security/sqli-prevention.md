---
name: sqli-prevention
category: security
tags: [security, sql-injection, parameterized-queries, owasp]
languages: [python, typescript, go, java, rust]
difficulty: beginner
---

## Intent

Prevent SQL injection attacks by ensuring user-supplied data is never concatenated into SQL queries, using parameterized queries or ORM abstractions as the primary defense.

## Problem

SQL injection occurs when untrusted input is interpolated directly into SQL strings. An attacker can alter query logic to bypass authentication, exfiltrate data, modify records, or execute administrative operations. SQLi remains in the OWASP Top 10 (A03:2021 — Injection) despite being entirely preventable.

## Solution

Use parameterized queries (prepared statements) for all database interactions. The database engine treats parameters as data, never as SQL syntax, making injection structurally impossible. ORMs provide this by default but require vigilance when using raw query escape hatches.

## Language Implementations

### Python (SQLAlchemy)

```python
# CORRECT — parameterized query
from sqlalchemy import text

result = session.execute(
    text("SELECT * FROM users WHERE email = :email AND status = :status"),
    {"email": user_email, "status": "active"}
)

# WRONG — string interpolation (SQL injection vulnerability)
# result = session.execute(f"SELECT * FROM users WHERE email = '{user_email}'")
```

### TypeScript (Prisma / pg)

```typescript
// CORRECT — Prisma (parameterized by default)
const user = await prisma.user.findFirst({
  where: { email: userEmail, status: "active" },
});

// CORRECT — raw parameterized query (pg driver)
const result = await pool.query(
  "SELECT * FROM users WHERE email = $1 AND status = $2",
  [userEmail, "active"]
);

// WRONG — template literal injection
// await pool.query(`SELECT * FROM users WHERE email = '${userEmail}'`);
```

### Go (database/sql)

```go
// CORRECT — parameterized query
row := db.QueryRowContext(ctx,
    "SELECT id, email FROM users WHERE email = $1 AND status = $2",
    userEmail, "active")

// WRONG — fmt.Sprintf into query
// query := fmt.Sprintf("SELECT * FROM users WHERE email = '%s'", userEmail)
```

### Java (JPA / JDBC)

```java
// CORRECT — JPA named parameters
@Query("SELECT u FROM User u WHERE u.email = :email AND u.status = :status")
Optional<User> findByEmail(@Param("email") String email, @Param("status") String status);

// CORRECT — JDBC PreparedStatement
PreparedStatement ps = conn.prepareStatement(
    "SELECT * FROM users WHERE email = ? AND status = ?");
ps.setString(1, userEmail);
ps.setString(2, "active");
```

### Rust (sqlx)

```rust
// CORRECT — compile-time checked query
let user = sqlx::query_as!(User,
    "SELECT id, email FROM users WHERE email = $1 AND status = $2",
    user_email, "active"
).fetch_optional(&pool).await?;
```

## When to Use

- Every database query that includes any external input, without exception.
- Dynamic query builders (search filters, sorting) must use parameterized predicates.
- Stored procedures that accept parameters from application code.

## When NOT to Use

- There is no scenario where string concatenation of user input into SQL is acceptable.

## Anti-Patterns

- Using an ORM but bypassing it with raw string-concatenated queries for "performance."
- Escaping special characters manually instead of using parameterized queries — escaping is fragile.
- Allowlisting table or column names via user input without validation against a static list.
- Trusting "internal" API callers — any input crossing a trust boundary must be parameterized.

## Related Patterns

- [security:input-validation](input-validation.md) — validate input before it reaches the query layer.
- [security:authn-authz](authn-authz.md) — limit database access to authorized users.
- [security:secrets-management](secrets-management.md) — protect database credentials.

## References

- OWASP A03:2021 Injection: https://owasp.org/Top10/A03_2021-Injection/
- OWASP SQL Injection Prevention Cheat Sheet: https://cheatsheetseries.owasp.org/cheatsheets/SQL_Injection_Prevention_Cheat_Sheet.html
- OWASP Query Parameterization Cheat Sheet: https://cheatsheetseries.owasp.org/cheatsheets/Query_Parameterization_Cheat_Sheet.html
- Bobby Tables — A guide to preventing SQL injection: https://bobby-tables.com/
