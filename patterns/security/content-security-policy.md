---
name: content-security-policy
category: security
tags: [security, csp, headers, browser]
languages: [all]
difficulty: intermediate
---

## Intent

Restrict the sources from which a browser can load scripts, styles, images, and other resources, mitigating XSS and data injection attacks.

## Problem

Even with output encoding, XSS vulnerabilities can slip through. Inline scripts, eval, and third-party resources create attack surfaces that encoding alone cannot eliminate.

## Solution

Set the `Content-Security-Policy` HTTP response header to declare an allowlist of trusted content sources per resource type. Start strict (`default-src 'none'`) and selectively permit needed sources.

## Language Implementations

### All
```http
Content-Security-Policy: default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; font-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'
```

Framework integration:
- **Express**: `helmet.contentSecurityPolicy({ directives: { ... } })`
- **Django**: `django-csp` middleware with `CSP_DEFAULT_SRC`
- **Spring**: `.headers().contentSecurityPolicy("...")`
- **Go**: Set header in middleware: `w.Header().Set("Content-Security-Policy", "...")`

## When to Use

- Every web application serving HTML to browsers.
- Single-page applications with API backends.
- Sites embedding third-party content (ads, widgets).

## When NOT to Use

- Pure API services returning only JSON (no browser rendering).
- Internal CLI tools or desktop applications.

## Anti-Patterns

- Using `unsafe-inline` or `unsafe-eval` (defeats the purpose).
- Report-only mode in production without enforcement.
- Overly permissive policies (`default-src *`).

## Related Patterns

- security/xss
- security/input-validation

## References

- MDN Content-Security-Policy documentation.
- CSP Evaluator tool (Google).
- OWASP CSP Cheat Sheet.
