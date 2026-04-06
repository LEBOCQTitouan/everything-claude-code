---
name: xss
category: security
tags: [security, xss, owasp, sanitization]
languages: [typescript, python, java, go]
difficulty: intermediate
---

## Intent

Prevent Cross-Site Scripting (XSS) attacks by sanitizing or encoding user-supplied content before rendering in HTML.

## Problem

Untrusted user input rendered directly into HTML enables attackers to inject malicious scripts that execute in other users' browsers, stealing sessions, credentials, or performing actions on their behalf.

## Solution

Apply output encoding appropriate to the rendering context (HTML body, attribute, JavaScript, URL). Use framework-provided auto-escaping. Sanitize rich content with allowlist-based HTML sanitizers.

## Language Implementations

### TypeScript
```typescript
import DOMPurify from 'dompurify';
const safe = DOMPurify.sanitize(userInput);
// Framework: React auto-escapes JSX by default
// Danger: dangerouslySetInnerHTML bypasses protection
```

### Python
```python
from markupsafe import escape
safe = escape(user_input)
# Django templates auto-escape by default
# Danger: |safe filter bypasses protection
```

### Java
```java
import org.owasp.encoder.Encode;
String safe = Encode.forHtml(userInput);
// Spring Thymeleaf auto-escapes by default
```

### Go
```go
import "html/template"
// html/template auto-escapes by context
// Danger: text/template does NOT escape
tmpl := template.Must(template.New("t").Parse(`<p>{{.}}</p>`))
```

## When to Use

- Any web application rendering user-supplied content.
- APIs returning HTML fragments.
- Email templates with user data.

## When NOT to Use

- Pure API backends returning JSON only (no HTML rendering).
- Internal tools with no untrusted input.

## Anti-Patterns

- Blocklist-based filtering (attackers bypass with encoding tricks).
- Client-side-only sanitization (server must also validate).
- Disabling auto-escaping globally for convenience.

## Related Patterns

- security/input-validation
- security/content-security-policy

## References

- OWASP XSS Prevention Cheat Sheet.
- DOMPurify library.
- OWASP Java Encoder.
