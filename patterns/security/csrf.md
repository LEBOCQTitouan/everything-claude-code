---
name: csrf
category: security
tags: [security, csrf, tokens, owasp, web]
languages: [python, typescript, java]
difficulty: intermediate
---

## Intent

Prevent cross-site request forgery attacks where a malicious site tricks a user's browser into making authenticated requests to your application without the user's knowledge.

## Problem

When authentication relies on cookies (session cookies, JWTs in cookies), the browser automatically attaches credentials to every request to your domain. A malicious page can embed a form or script that submits a request to your API — the browser sends the cookie, and your server cannot distinguish this forged request from a legitimate one.

## Solution

Require a secret token in every state-changing request that the attacker cannot predict. The token is tied to the user's session and validated server-side. Modern approaches combine synchronizer tokens with SameSite cookie attributes for defense in depth.

## Language Implementations

### Python

```python
# settings.py — enabled by default
MIDDLEWARE = [
    "django.middleware.csrf.CsrfViewMiddleware",  # auto-enabled
]

# For API views using session auth, include CSRF token
# Django sets csrftoken cookie; client reads it and sends as X-CSRFToken header

# Exempt specific views (use sparingly — only for webhook receivers)
from django.views.decorators.csrf import csrf_exempt

@csrf_exempt
def stripe_webhook(request):
    # Verify via Stripe signature instead of CSRF token
    verify_stripe_signature(request)
```

### TypeScript

```typescript
import csrf from "csrf";

const tokens = new csrf();
const secret = tokens.secretSync();

// Set token in cookie
app.use((req, res, next) => {
  const token = tokens.create(secret);
  res.cookie("csrf-token", token, { httpOnly: false, sameSite: "strict" });
  next();
});

// Verify on state-changing requests
app.use((req, res, next) => {
  if (["POST", "PUT", "PATCH", "DELETE"].includes(req.method)) {
    const token = req.headers["x-csrf-token"] as string;
    if (!tokens.verify(secret, token)) {
      return res.status(403).json({ error: { code: "CSRF_INVALID" } });
    }
  }
  next();
});
```

### Java

```java
@Configuration
public class SecurityConfig {
    @Bean
    public SecurityFilterChain filterChain(HttpSecurity http) throws Exception {
        return http
            .csrf(csrf -> csrf
                .csrfTokenRepository(CookieCsrfTokenRepository.withHttpOnlyFalse())
                .csrfTokenRequestHandler(new CsrfTokenRequestAttributeHandler())
                // Exempt webhook endpoints that use signature verification
                .ignoringRequestMatchers("/webhooks/**")
            ).build();
    }
}
// Client reads XSRF-TOKEN cookie, sends as X-XSRF-TOKEN header
```


## When to Use

- Every web application using cookie-based authentication (session cookies, JWT in cookies).
- Server-rendered forms that submit POST requests.
- SPAs using cookie-based auth that make API calls to the same origin.

## When NOT to Use

- APIs authenticated exclusively via Authorization header (Bearer tokens) — cookies are not involved.
- Stateless APIs consumed only by non-browser clients (mobile apps, server-to-server).

## Anti-Patterns

- Disabling CSRF protection globally because "we use an SPA" — if cookies carry auth, CSRF applies.
- Using GET requests for state-changing operations — CSRF is trivial via `<img>` tags.
- Storing the CSRF token in an httpOnly cookie that JavaScript cannot read for the header.
- Not regenerating the CSRF token after login — enables session fixation attacks.

## Related Patterns

- [security:authn-authz](authn-authz.md) — CSRF protection is required for cookie-based authentication.
- [security:xss](xss.md) — XSS can steal CSRF tokens, so both defenses are needed.
- [security:content-security-policy](content-security-policy.md) — CSP reduces XSS risk that could bypass CSRF.

## References

- OWASP CSRF Prevention Cheat Sheet: https://cheatsheetseries.owasp.org/cheatsheets/Cross-Site_Request_Forgery_Prevention_Cheat_Sheet.html
- OWASP A01:2021 Broken Access Control: https://owasp.org/Top10/A01_2021-Broken_Access_Control/
- Django CSRF Documentation: https://docs.djangoproject.com/en/5.0/ref/csrf/
- Spring Security CSRF: https://docs.spring.io/spring-security/reference/servlet/exploits/csrf.html
