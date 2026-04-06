---
name: functional-options
category: idioms
tags: [idiom, go]
languages: [go]
difficulty: intermediate
---

## Intent

Provide a clean, extensible API for configuring complex objects using variadic function arguments, avoiding telescoping constructors and large config structs with many zero-value fields.

## Problem

Go lacks named parameters, default arguments, and method overloading. Constructors with many parameters are brittle and hard to read. Config structs work but force callers to understand every field, and zero values may be ambiguous (is `0` a deliberate value or an unset default?).

## Solution

Define an `Option` function type that mutates a config struct. The constructor accepts variadic `...Option` arguments and applies them in order over sensible defaults. Each option is a closure returned by a named constructor function.

## Language Implementations

### Go

```go
type Server struct {
    host       string
    port       int
    timeoutMs  int
    maxRetries int
}

// Option is a function that configures a Server
type Option func(*Server)

func WithPort(p int) Option {
    return func(s *Server) { s.port = p }
}

func WithTimeout(ms int) Option {
    return func(s *Server) { s.timeoutMs = ms }
}

func WithMaxRetries(n int) Option {
    return func(s *Server) { s.maxRetries = n }
}

func NewServer(host string, opts ...Option) *Server {
    s := &Server{
        host:       host,
        port:       8080,     // sensible default
        timeoutMs:  5000,     // sensible default
        maxRetries: 3,        // sensible default
    }
    for _, opt := range opts {
        opt(s)
    }
    return s
}

// Usage:
// srv := NewServer("localhost", WithPort(9090), WithTimeout(10000))
```

## When to Use

- When a constructor has more than 2-3 optional parameters.
- When you want a self-documenting, backward-compatible API.
- When defaults should be sensible and overrides should be explicit.

## When NOT to Use

- When there are only 1-2 required parameters and no optional ones.
- When a plain struct literal with named fields is clear enough.
- When options need validation that should fail at construction time (add error return).

## Anti-Patterns

- Allowing conflicting options without detecting or documenting the conflict.
- Mutating option closures after passing them (options should be stateless).
- Using functional options for simple cases where a struct literal is clearer.

## Related Patterns

- [builder](../../creational/builder.md) -- step-by-step construction; functional options are Go's idiomatic alternative.
- [factory-method](../../creational/factory-method.md) -- simpler creation without configuration.

## References

- Dave Cheney -- Functional Options for Friendly APIs: https://dave.cheney.net/2014/10/17/functional-options-for-friendly-apis
- Rob Pike -- Self-referential functions: https://commandcenter.blogspot.com/2014/01/self-referential-functions-and-design.html
