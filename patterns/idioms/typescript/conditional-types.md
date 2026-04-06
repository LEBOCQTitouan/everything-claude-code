---
name: conditional-types
category: idioms
tags: [idiom, typescript]
languages: [typescript]
difficulty: advanced
---

## Intent

Perform type-level computation using `extends` and `infer` to derive new types conditionally, enabling generic APIs that return different types based on input type structure without runtime overhead.

## Problem

Generic functions sometimes need to return different types depending on the input. A `parse` function might return `number` for `"number"` format or `string` for `"string"` format. Without conditional types, the return type must be a loose union or `any`, losing type safety at call sites.

## Solution

Use the `T extends U ? X : Y` syntax to branch at the type level. Use `infer` to extract parts of a type within the condition. Conditional types distribute over unions automatically, enabling powerful type transformations.

## Language Implementations

### TypeScript

```typescript
// Basic conditional type
type IsString<T> = T extends string ? true : false;
// IsString<"hello"> = true
// IsString<42> = false

// Extracting types with infer
type UnwrapPromise<T> = T extends Promise<infer U> ? U : T;
// UnwrapPromise<Promise<string>> = string
// UnwrapPromise<number> = number

// Recursive unwrapping
type DeepUnwrapPromise<T> = T extends Promise<infer U>
  ? DeepUnwrapPromise<U>
  : T;

// Practical: type-safe event emitter
type EventMap = {
  click: { x: number; y: number };
  keydown: { key: string; code: number };
  resize: { width: number; height: number };
};

type EventHandler<K extends keyof EventMap> = (
  event: EventMap[K]
) => void;

function on<K extends keyof EventMap>(
  event: K,
  handler: EventHandler<K>
): void {
  // ...
}

// Compiler infers the event payload type:
on("click", (e) => console.log(e.x, e.y));
on("keydown", (e) => console.log(e.key));

// Utility types built from conditional types
type NonNullableFields<T> = {
  [K in keyof T]: NonNullable<T[K]>;
};

type ExtractMethods<T> = {
  [K in keyof T as T[K] extends (...args: unknown[]) => unknown
    ? K
    : never]: T[K];
};
```

## When to Use

- When building generic utility types that transform type structure.
- When API return types depend on input types (format-dependent parsing).
- When creating type-safe wrappers around dynamic libraries.

## When NOT to Use

- When a simple generic parameter or union type suffices.
- When the conditional type is so complex it hinders readability.
- When the type computation causes excessive compiler slowdown on large projects.

## Anti-Patterns

- Nesting more than 2-3 levels of conditional types (unreadable).
- Relying on distributive behavior without understanding it (use `[T] extends [U]` to prevent).
- Using `infer` in positions where the type cannot be reliably inferred.

## Related Patterns

- [discriminated-unions](discriminated-unions.md) -- value-level branching that mirrors type-level branching.
- [type-guards](type-guards.md) -- runtime narrowing that complements compile-time conditional types.
- [branded-types](branded-types.md) -- conditional types can detect and transform branded types.

## References

- TypeScript Handbook -- Conditional Types: https://www.typescriptlang.org/docs/handbook/2/conditional-types.html
- TypeScript Handbook -- Template Literal Types: https://www.typescriptlang.org/docs/handbook/2/template-literal-types.html
