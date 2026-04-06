---
name: type-guards
category: idioms
tags: [idiom, typescript]
languages: [typescript]
difficulty: intermediate
---

## Intent

Define user-defined type predicate functions using the `is` keyword to narrow union types at runtime, enabling the compiler to refine the type within guarded branches.

## Problem

TypeScript's built-in narrowing (`typeof`, `instanceof`, `in`) covers common cases but fails for complex types, branded types, or API response validation. Without a type guard, the developer must use type assertions (`as`) which bypass safety checks.

## Solution

Write a function returning `paramName is TargetType`. Inside the function, perform the actual runtime check. The compiler trusts the predicate and narrows the type in the calling scope's `if` branch.

## Language Implementations

### TypeScript

```typescript
interface ApiError {
  readonly code: number;
  readonly message: string;
}

interface ApiSuccess<T> {
  readonly data: T;
}

type ApiResponse<T> = ApiError | ApiSuccess<T>;

// Type guard function
function isApiError<T>(response: ApiResponse<T>): response is ApiError {
  return "code" in response && "message" in response;
}

// Usage -- compiler narrows after the guard
function handleResponse<T>(response: ApiResponse<T>): T {
  if (isApiError(response)) {
    // response is narrowed to ApiError
    throw new Error(`API error ${response.code}: ${response.message}`);
  }
  // response is narrowed to ApiSuccess<T>
  return response.data;
}

// Array filtering with type guards
function isNonNull<T>(value: T | null | undefined): value is T {
  return value != null;
}

const items: (string | null)[] = ["a", null, "b", null, "c"];
const filtered: string[] = items.filter(isNonNull);

// Assertion function (throws instead of returning boolean)
function assertIsString(value: unknown): asserts value is string {
  if (typeof value !== "string") {
    throw new TypeError(`Expected string, got ${typeof value}`);
  }
}
```

## When to Use

- When narrowing union types that built-in checks cannot handle.
- When filtering arrays to a specific type (`.filter(isX)` preserves the narrowed type).
- When validating external data (API responses, parsed JSON) at system boundaries.

## When NOT to Use

- When `typeof`, `instanceof`, or discriminated unions provide sufficient narrowing.
- When the guard logic is so simple it belongs inline (`if (x !== null)`).
- When the predicate cannot be reliably verified at runtime (guard lies to the compiler).

## Anti-Patterns

- Writing a type guard that returns `true` when the value does not match the asserted type.
- Using type guards as a substitute for proper input validation with schema libraries.
- Creating overly broad guards that narrow to a type with many optional fields.

## Related Patterns

- [discriminated-unions](discriminated-unions.md) -- built-in narrowing that often eliminates the need for guards.
- [branded-types](branded-types.md) -- type guards complement brands for runtime validation.

## References

- TypeScript Handbook -- Type Guards and Narrowing: https://www.typescriptlang.org/docs/handbook/2/narrowing.html
- TypeScript Handbook -- Using type predicates: https://www.typescriptlang.org/docs/handbook/2/narrowing.html#using-type-predicates
