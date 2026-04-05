---
name: versioning
category: api-design
tags: [api-design, versioning, backwards-compatibility, evolution]
languages: [all]
difficulty: intermediate
---

## Intent

Evolve API contracts over time without breaking existing clients, while allowing new clients to adopt improved interfaces.

## Problem

APIs inevitably change: fields are added, renamed, or removed; response shapes evolve. Without a versioning strategy, any breaking change forces all clients to update simultaneously, creating painful coordination and deployment coupling.

## Solution

Choose a versioning strategy that matches your API's lifecycle: URI path versioning for simplicity, header-based versioning for cleaner URIs, or content negotiation for fine-grained control. Maintain backward compatibility within a version and communicate deprecation timelines.

## Language Implementations

### URI Path Versioning (Most Common)

```
GET /v1/users/123
GET /v2/users/123

# Router configuration (framework-agnostic)
/v1/* -> handler_v1
/v2/* -> handler_v2
```

### Header-Based Versioning

```
GET /users/123
Accept: application/vnd.myapi.v2+json

# Or custom header
GET /users/123
X-API-Version: 2
```

### Additive Change Strategy (Preferred)

```json
// v1 response
{ "id": "u_123", "name": "Alice" }

// v1-compatible evolution: add fields, never remove
{ "id": "u_123", "name": "Alice", "display_name": "Alice W." }

// Deprecation header when field will be removed
Sunset: Sat, 01 Jan 2028 00:00:00 GMT
Deprecation: true
Link: <https://api.example.com/docs/migration>; rel="deprecation"
```

### Deprecation Policy

```yaml
versioning:
  strategy: uri_path
  supported_versions: [v2, v3]
  deprecated_versions:
    v1: { sunset_date: "2028-01-01", migration_guide: "/docs/v1-to-v2" }
  current_version: v3
  deprecation_notice_months: 12
```

## When to Use

- Public APIs with external consumers who cannot update on your schedule.
- APIs with contractual SLAs requiring stability guarantees.
- When breaking changes to response schemas are unavoidable.

## When NOT to Use

- Internal APIs where all consumers deploy together (use additive changes and feature flags instead).
- Experimental or alpha APIs where clients accept instability.

## Anti-Patterns

- Versioning every minor change instead of making additive, backward-compatible changes.
- Maintaining more than two active versions simultaneously — operational cost compounds.
- Removing fields without a deprecation period and sunset header.
- Using date-based versioning (`/2024-01-15/`) for REST APIs — this works for some providers but confuses most consumers.

## Related Patterns

- [rest-resources](rest-resources.md) — versioning applies to the resource contract.
- [api-gateway](api-gateway.md) — route version-specific traffic at the gateway.
- [graphql-schema](graphql-schema.md) — GraphQL avoids versioning via additive schema evolution and field deprecation.

## References

- Stripe API Versioning: https://stripe.com/docs/api/versioning
- Microsoft REST API Guidelines — Versioning: https://github.com/microsoft/api-guidelines/blob/vNext/Guidelines.md
- IETF Sunset Header: https://datatracker.ietf.org/doc/rfc8594/
