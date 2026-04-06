---
name: grpc-service
category: api-design
tags: [api-design, grpc, protobuf, rpc, streaming]
languages: [all]
difficulty: advanced
---

## Intent

Define high-performance, strongly-typed service interfaces using Protocol Buffers and gRPC, enabling efficient binary communication with built-in streaming, code generation, and cross-language interoperability.

## Problem

REST over JSON incurs serialization overhead, lacks a formal contract (OpenAPI is optional and often drifts), and cannot natively express streaming or bidirectional communication. Internal microservice communication needs lower latency, smaller payloads, and compile-time safety across language boundaries.

## Solution

Define service contracts in `.proto` files using Protocol Buffers IDL. Generate client and server stubs in any supported language. Use gRPC's four communication patterns: unary, server streaming, client streaming, and bidirectional streaming.

## Language Implementations

### Proto Definition

```protobuf
syntax = "proto3";
package orders.v1;

service OrderService {
  // Unary
  rpc CreateOrder(CreateOrderRequest) returns (CreateOrderResponse);
  rpc GetOrder(GetOrderRequest) returns (Order);

  // Server streaming
  rpc WatchOrderStatus(WatchOrderRequest) returns (stream OrderStatus);

  // Client streaming
  rpc BatchCreateOrders(stream CreateOrderRequest) returns (BatchCreateResponse);
}

message CreateOrderRequest {
  string customer_id = 1;
  repeated OrderItem items = 2;
  string idempotency_key = 3;
}

message CreateOrderResponse {
  Order order = 1;
}

message Order {
  string id = 1;
  string customer_id = 2;
  repeated OrderItem items = 3;
  OrderStatus status = 4;
  google.protobuf.Timestamp created_at = 5;
}

enum OrderStatus {
  ORDER_STATUS_UNSPECIFIED = 0;
  ORDER_STATUS_PENDING = 1;
  ORDER_STATUS_CONFIRMED = 2;
  ORDER_STATUS_SHIPPED = 3;
}

message OrderItem {
  string product_id = 1;
  int32 quantity = 2;
  int64 price_cents = 3;
}
```

### Proto Best Practices

```protobuf
// 1. Always version packages: orders.v1, orders.v2
// 2. First enum value must be UNSPECIFIED = 0
// 3. Use wrapper types for nullable fields
// 4. Reserve removed field numbers to prevent reuse
message Order {
  reserved 6, 7;
  reserved "legacy_status";
}

// 5. Use FieldMask for partial updates
rpc UpdateOrder(UpdateOrderRequest) returns (Order);
message UpdateOrderRequest {
  string id = 1;
  Order order = 2;
  google.protobuf.FieldMask update_mask = 3;
}
```

### Error Handling

```
# gRPC uses status codes (not HTTP status codes)
# Include structured error details via google.rpc.Status

Status {
  code: INVALID_ARGUMENT (3)
  message: "customer_id is required"
  details: [
    BadRequest {
      field_violations: [
        { field: "customer_id", description: "must not be empty" }
      ]
    }
  ]
}
```

## When to Use

- Internal microservice communication where latency and payload size matter.
- Polyglot environments needing a single contract shared across languages.
- Streaming use cases: real-time feeds, log tailing, bidirectional chat.
- When compile-time type safety across service boundaries is critical.

## When NOT to Use

- Browser-facing APIs without a gRPC-Web proxy — browsers cannot make native gRPC calls.
- Simple public APIs where JSON readability and curl-friendliness matter.
- When the team lacks Protocol Buffers experience and REST suffices.

## Anti-Patterns

- Reusing field numbers after deleting fields — always `reserved` removed numbers.
- Designing services with huge request/response messages instead of streaming.
- Ignoring deadlines and cancellation — always propagate context deadlines.
- Using gRPC without TLS in production — gRPC has first-class TLS support.

## Related Patterns

- [rest-resources](rest-resources.md) — alternative for public-facing, browser-friendly APIs.
- [api-gateway](api-gateway.md) — gateway can transcode REST to gRPC for external clients.
- [idempotency-keys](idempotency-keys.md) — include idempotency keys in gRPC request messages.

## References

- gRPC Documentation: https://grpc.io/docs/
- Google API Design Guide (gRPC): https://cloud.google.com/apis/design
- Protocol Buffers Language Guide: https://protobuf.dev/programming-guides/proto3/
- gRPC Error Handling: https://grpc.io/docs/guides/error/
