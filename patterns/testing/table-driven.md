---
name: table-driven
category: testing
tags: [testing, parameterised, data-driven, table-driven]
languages: [rust, go, python, typescript]
difficulty: beginner
---

## Intent

Define test inputs and expected outputs as rows in a table, running the same test logic against each row to maximise coverage with minimal code duplication.

## Problem

Writing a separate test function for each input/output combination creates duplication. Adding a new test case requires copying boilerplate. Missing edge cases is easy when each test is isolated.

## Solution

Define a collection of test cases as data (name, input, expected output). Iterate over the collection, running the same assertion logic for each case. Each row is a self-contained scenario with a descriptive name for failure reporting.

## Language Implementations

### Rust

```rust
#[test]
fn test_parse_status_code() {
    let cases = vec![
        ("ok",          200, StatusCategory::Success),
        ("not found",   404, StatusCategory::ClientError),
        ("server error",500, StatusCategory::ServerError),
        ("redirect",    301, StatusCategory::Redirect),
    ];

    for (name, code, expected) in cases {
        assert_eq!(categorise(code), expected, "case: {name}");
    }
}

// Or with rstest:
#[rstest]
#[case(200, StatusCategory::Success)]
#[case(404, StatusCategory::ClientError)]
#[case(500, StatusCategory::ServerError)]
fn test_categorise(#[case] code: u16, #[case] expected: StatusCategory) {
    assert_eq!(categorise(code), expected);
}
```

### Go

```go
func TestCategorise(t *testing.T) {
    tests := []struct {
        name     string
        code     int
        expected StatusCategory
    }{
        {"ok", 200, Success},
        {"not found", 404, ClientError},
        {"server error", 500, ServerError},
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            got := Categorise(tt.code)
            if got != tt.expected {
                t.Errorf("Categorise(%d) = %v, want %v", tt.code, got, tt.expected)
            }
        })
    }
}
```

### Python

```python
import pytest

@pytest.mark.parametrize("code,expected", [
    (200, StatusCategory.SUCCESS),
    (404, StatusCategory.CLIENT_ERROR),
    (500, StatusCategory.SERVER_ERROR),
], ids=["ok", "not-found", "server-error"])
def test_categorise(code: int, expected: StatusCategory):
    assert categorise(code) == expected
```

### Typescript

```typescript
test.each([
  ["ok",           200, "success"],
  ["not found",    404, "client_error"],
  ["server error", 500, "server_error"],
])("%s: categorise(%i) = %s", (_name, code, expected) => {
  expect(categorise(code)).toBe(expected);
});
```

## When to Use

- When the same logic needs testing with many different inputs.
- When edge cases can be enumerated as data rows.
- Idiomatic default in Go testing; strongly recommended everywhere.

## When NOT to Use

- When each test case requires significantly different setup or assertions.
- When the test table grows so large it becomes hard to read (split into groups).

## Anti-Patterns

- Omitting descriptive names for each case, making failures hard to diagnose.
- Mixing different behaviours into a single table when they need different assertions.
- Tables with dozens of columns -- simplify inputs or split into multiple tables.

## Related Patterns

- [testing/aaa](aaa.md) -- each table row follows the AAA structure.
- [testing/property-based](property-based.md) -- auto-generate inputs instead of hand-picking.
- [testing/given-when-then](given-when-then.md) -- table rows can follow GWT naming.

## References

- Go Wiki, "Table Driven Tests": https://go.dev/wiki/TableDrivenTests
- **Rust**: `rstest` `#[case]`, manual vec iteration
- **Go**: `testing` package subtests (idiomatic)
- **Python**: `pytest.mark.parametrize`
- **Kotlin**: Kotest `forAll`, `withData`
- **TypeScript**: `test.each` (vitest/jest)
