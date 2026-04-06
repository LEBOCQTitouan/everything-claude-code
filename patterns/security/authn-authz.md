---
name: authn-authz
category: security
tags: [security, authentication, authorization, jwt, oauth, owasp]
languages: [python, typescript, go, java]
difficulty: intermediate
---

## Intent

Verify the identity of API consumers (authentication) and enforce what actions they are permitted to perform (authorization), implementing defense in depth across every protected endpoint.

## Problem

Without proper authentication, anyone can impersonate legitimate users. Without authorization, authenticated users can access resources and perform actions beyond their privileges. Broken access control is the number one OWASP Top 10 vulnerability (A01:2021).

## Solution

Authenticate via stateless JWT tokens or session-based tokens validated on every request. Authorize via role-based (RBAC) or attribute-based (ABAC) policies evaluated at the service layer. Never rely solely on client-side checks.

## Language Implementations


### Python

```python
from rest_framework.permissions import BasePermission

class IsOrderOwner(BasePermission):
    def has_object_permission(self, request, view, obj):
        return obj.customer_id == request.user.id

# View
class OrderViewSet(viewsets.ModelViewSet):
    permission_classes = [IsAuthenticated, IsOrderOwner]
    queryset = Order.objects.all()

    def get_queryset(self):
        # Always filter by authenticated user — never return all orders
        return self.queryset.filter(customer_id=self.request.user.id)
```

### TypeScript

```typescript
function requireRole(...roles: string[]) {
  return (req: Request, res: Response, next: NextFunction) => {
    const user = req.user; // set by auth middleware
    if (!user || !roles.some(r => user.roles.includes(r))) {
      return res.status(403).json({ error: { code: "FORBIDDEN" } });
    }
    next();
  };
}

// Route
app.delete("/orders/:id", requireRole("admin", "order_manager"), deleteOrder);
```

### Java

```java
@Configuration
@EnableMethodSecurity
public class SecurityConfig {
    @Bean
    public SecurityFilterChain filterChain(HttpSecurity http) throws Exception {
        return http
            .oauth2ResourceServer(oauth2 -> oauth2.jwt(Customizer.withDefaults()))
            .authorizeHttpRequests(auth -> auth
                .requestMatchers("/health").permitAll()
                .requestMatchers("/admin/**").hasRole("ADMIN")
                .anyRequest().authenticated()
            ).build();
    }
}

// Method-level
@PreAuthorize("hasRole('ADMIN') or #userId == authentication.name")
public Order getOrder(String userId, String orderId) { ... }
```

## When to Use

- Every API endpoint that accesses or modifies protected resources.
- Every microservice boundary, even internal ones (zero-trust).
- Any operation where different users have different privilege levels.

## When NOT to Use

- Truly public endpoints (health checks, public documentation, open data feeds).
- Internal service-to-service calls using mutual TLS where identity is established at the transport layer.

## Anti-Patterns

- Checking permissions only in the UI/frontend — always enforce server-side.
- Using a shared API key for all clients instead of per-client credentials.
- Storing JWTs in localStorage (vulnerable to XSS) — use httpOnly cookies.
- Not validating JWT audience and issuer claims, allowing token reuse across services.
- Rolling your own JWT library instead of using a vetted implementation.

## Related Patterns

- [security:csrf](csrf.md) — CSRF protection is needed alongside cookie-based authentication.
- [security:secrets-management](secrets-management.md) — securely store signing keys and API secrets.
- [security:input-validation](input-validation.md) — validate tokens and credentials at boundaries.
- [api-design:rate-limiting](../api-design/rate-limiting.md) — rate limit authentication attempts.

## References

- OWASP A01:2021 Broken Access Control: https://owasp.org/Top10/A01_2021-Broken_Access_Control/
- OWASP Authentication Cheat Sheet: https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html
- RFC 7519 — JSON Web Token: https://datatracker.ietf.org/doc/html/rfc7519
- Spring Security Reference: https://docs.spring.io/spring-security/reference/
