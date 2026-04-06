---
name: input-validation
category: security
tags: [security, validation, owasp, injection-prevention]
languages: [python, typescript, go, rust, java]
difficulty: beginner
---

## Intent

Validate all external input at system boundaries to reject malformed, malicious, or out-of-range data before it reaches business logic, preventing injection attacks and data corruption.

## Problem

Unvalidated input is the root cause of most security vulnerabilities: SQL injection, XSS, command injection, path traversal, and buffer overflows all exploit missing or insufficient input checks. Trusting any external data — user input, API responses, file contents, environment variables — is a security defect.

## Solution

Apply schema-based validation at every system boundary using an allowlist approach: define what is valid and reject everything else. Validate type, format, length, range, and business rules. Return clear error messages without leaking internal details.

## Language Implementations

### Python

```python
from pydantic import BaseModel, EmailStr, Field, field_validator
import re

class CreateUserRequest(BaseModel):
    email: EmailStr
    username: str = Field(min_length=3, max_length=30, pattern=r"^[a-zA-Z0-9_]+$")
    age: int = Field(ge=13, le=150)

    @field_validator("username")
    @classmethod
    def no_reserved_names(cls, v: str) -> str:
        if v.lower() in {"admin", "root", "system"}:
            raise ValueError("reserved username")
        return v
```

### TypeScript

```typescript
import { z } from "zod";

const CreateUserSchema = z.object({
  email: z.string().email(),
  username: z.string().min(3).max(30).regex(/^[a-zA-Z0-9_]+$/),
  age: z.number().int().min(13).max(150),
}).strict(); // reject unknown fields

type CreateUserRequest = z.infer<typeof CreateUserSchema>;

// Usage at boundary
function handleRequest(raw: unknown): CreateUserRequest {
  return CreateUserSchema.parse(raw); // throws ZodError on invalid input
}
```

### Go

```go
type CreateUserRequest struct {
    Email    string `json:"email" validate:"required,email"`
    Username string `json:"username" validate:"required,min=3,max=30,alphanum"`
    Age      int    `json:"age" validate:"required,gte=13,lte=150"`
}

func (r CreateUserRequest) Validate() error {
    return validator.New().Struct(r)
}
```

### Java

```java
public record CreateUserRequest(
    @NotBlank @Email String email,
    @NotBlank @Size(min = 3, max = 30) @Pattern(regexp = "^[a-zA-Z0-9_]+$") String username,
    @NotNull @Min(13) @Max(150) Integer age
) {}
```

## When to Use

- Every API endpoint, form handler, file parser, and message consumer.
- Every boundary between your code and external systems (databases, third-party APIs).
- Configuration files and environment variables at startup.

## When NOT to Use

- Internal function calls within a trusted boundary where types already enforce constraints.
- Performance-critical hot paths where validation was already performed at the boundary.

## Anti-Patterns

- Validating only on the client side — server must always re-validate.
- Using blocklists ("reject these characters") instead of allowlists ("accept only these characters").
- Returning raw validation errors that expose internal field names or stack traces.
- Validating input after it has already been used in a query or command.

## Related Patterns

- [security:sqli-prevention](sqli-prevention.md) — input validation is the first defense layer against SQL injection.
- [security:xss](xss.md) — validated input reduces XSS surface but output encoding is still required.
- [security:authn-authz](authn-authz.md) — validate authentication tokens at the boundary.

## References

- OWASP Input Validation Cheat Sheet: https://cheatsheetseries.owasp.org/cheatsheets/Input_Validation_Cheat_Sheet.html
- Pydantic Documentation: https://docs.pydantic.dev/
- Zod Documentation: https://zod.dev/
- OWASP A03:2021 Injection: https://owasp.org/Top10/A03_2021-Injection/
