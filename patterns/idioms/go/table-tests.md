---
name: table-tests
category: idioms
tags: [idiom, go]
languages: [go]
difficulty: beginner
---

## Intent

Express multiple test cases as a data table and iterate over them in a single test function, reducing boilerplate and making it trivial to add new cases.

## Problem

Writing a separate test function for each input/output combination leads to massive duplication. The test logic (setup, act, assert) is identical across cases; only the data differs. Adding edge cases requires copying entire functions.

## Solution

Define a slice of structs containing test name, inputs, and expected outputs. Range over the slice, running each case as a subtest via `t.Run`. Each subtest has its own name, can fail independently, and can be run in isolation with `-run`.

## Language Implementations

### Go

```go
func TestParsePort(t *testing.T) {
    tests := []struct {
        name    string
        input   string
        want    int
        wantErr bool
    }{
        {name: "valid port", input: "8080", want: 8080},
        {name: "zero port", input: "0", want: 0},
        {name: "max port", input: "65535", want: 65535},
        {name: "negative", input: "-1", wantErr: true},
        {name: "too large", input: "65536", wantErr: true},
        {name: "non-numeric", input: "abc", wantErr: true},
        {name: "empty string", input: "", wantErr: true},
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            got, err := ParsePort(tt.input)
            if tt.wantErr {
                if err == nil {
                    t.Fatalf("ParsePort(%q) expected error, got %d", tt.input, got)
                }
                return
            }
            if err != nil {
                t.Fatalf("ParsePort(%q) unexpected error: %v", tt.input, err)
            }
            if got != tt.want {
                t.Errorf("ParsePort(%q) = %d, want %d", tt.input, got, tt.want)
            }
        })
    }
}

// Run a single case: go test -run TestParsePort/negative
// Parallel subtests: add t.Parallel() inside t.Run
```

## When to Use

- When testing a pure function with multiple input/output pairs.
- When edge cases are best expressed as data rather than code.
- When you want each case to have a named subtest for targeted execution.

## When NOT to Use

- When test logic differs significantly between cases (use separate test functions).
- When setup or teardown varies per case beyond what the struct can express.
- When the table would have so many fields it becomes unreadable.

## Anti-Patterns

- Omitting the `name` field, making failure output impossible to identify.
- Sharing mutable state across subtests without `t.Parallel()` guards.
- Adding conditional logic inside the loop body (splits belong in the struct or separate tests).

## Related Patterns

- [functional-options](functional-options.md) -- the constructor under test often uses this pattern.
- [error-wrapping](error-wrapping.md) -- table tests are ideal for verifying error chain behavior.

## References

- Go Wiki -- TableDrivenTests: https://go.dev/wiki/TableDrivenTests
- Dave Cheney -- Writing Table Driven Tests in Go: https://dave.cheney.net/2019/05/07/prefer-table-driven-tests
