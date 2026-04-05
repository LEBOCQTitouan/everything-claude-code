---
name: api-gateway
category: api-design
tags: [api-design, gateway, routing, cross-cutting, microservices]
languages: [all]
difficulty: advanced
---

## Intent

Provide a single entry point for all API clients that handles cross-cutting concerns (authentication, rate limiting, routing, protocol translation) so backend services remain focused on business logic.

## Problem

When clients call microservices directly, every service must independently implement authentication, rate limiting, CORS, logging, and TLS termination. Client coupling to internal service topology makes refactoring impossible. Mobile clients suffer from chatty multi-service round trips.

## Solution

Deploy a gateway layer that sits between clients and backend services. The gateway handles cross-cutting concerns centrally, routes requests to appropriate services, and can aggregate multiple service calls into a single client response.

## Language Implementations

### Gateway Responsibilities

```yaml
api_gateway:
  cross_cutting:
    - tls_termination
    - authentication      # validate JWT/API keys
    - rate_limiting        # per-client, per-route
    - cors_headers
    - request_logging
    - request_id_injection # add X-Request-ID

  routing:
    - path_based:    /users/* -> user-service
    - header_based:  Accept: application/grpc -> grpc-backend
    - version_based: /v1/* -> service-v1, /v2/* -> service-v2

  transformations:
    - rest_to_grpc:  external REST -> internal gRPC
    - response_aggregation: combine user + orders in one response
    - field_filtering: strip internal fields from responses
```

### Route Configuration (Declarative)

```yaml
routes:
  - path: /api/v1/users/**
    service: user-service
    strip_prefix: /api/v1
    rate_limit: { requests: 100, window: 1m }
    auth: required
    timeout: 5s

  - path: /api/v1/orders/**
    service: order-service
    strip_prefix: /api/v1
    rate_limit: { requests: 50, window: 1m }
    auth: required
    timeout: 10s
    retry: { attempts: 2, backoff: exponential }

  - path: /health
    service: gateway-internal
    auth: none
    rate_limit: none
```

### Backend-for-Frontend (BFF) Pattern

```
# Mobile BFF — aggregates and optimizes for mobile
GET /mobile/v1/home
  -> user-service:    GET /users/{id}
  -> order-service:   GET /users/{id}/orders?limit=5
  -> product-service: GET /recommendations/{id}
  <- aggregated, minimized response

# Web BFF — richer data for desktop
GET /web/v1/dashboard
  -> same services, different field selection and aggregation
```

## When to Use

- Microservice architectures with multiple backend services.
- When external clients need a stable entry point decoupled from internal topology.
- When cross-cutting concerns must be enforced consistently across all services.

## When NOT to Use

- Monolithic applications with a single backend — adds unnecessary complexity.
- When latency overhead of an extra network hop is unacceptable (use service mesh instead).
- Small teams with fewer than three services — direct routing suffices.

## Anti-Patterns

- Putting business logic in the gateway — it should only handle routing and cross-cutting concerns.
- Creating a single monolithic gateway that becomes a deployment bottleneck.
- Not implementing circuit breaking — a failing backend should not cascade through the gateway.
- Allowing the gateway to grow into an ESB (Enterprise Service Bus) with complex orchestration.

## Related Patterns

- [rate-limiting](rate-limiting.md) — centralized rate limiting at the gateway.
- [rest-resources](rest-resources.md) — gateway routes resource requests to backend services.
- [grpc-service](grpc-service.md) — gateway transcodes REST to gRPC for internal services.
- [versioning](versioning.md) — gateway handles version-based routing.

## References

- Kong Gateway: https://docs.konghq.com/
- AWS API Gateway: https://docs.aws.amazon.com/apigateway/
- Netflix Zuul / Spring Cloud Gateway: https://spring.io/projects/spring-cloud-gateway
- Sam Newman, Building Microservices, Chapter 8 — API Gateways.
