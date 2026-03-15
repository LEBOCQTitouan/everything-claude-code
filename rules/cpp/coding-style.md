---
paths:
  - "**/*.c"
  - "**/*.cpp"
  - "**/*.h"
  - "**/*.hpp"
  - "**/*.cc"
  - "**/*.cxx"
  - "**/CMakeLists.txt"
---
# C/C++ Coding Style

> This file extends [common/coding-style.md](../common/coding-style.md) with C/C++ specific content.

## Formatting

- **clang-format** is mandatory — configure `.clang-format` in project root
- Prefer `clang-tidy` for static analysis

## Modern C++ (C++17/20/23)

- Use `auto` when the type is obvious from context
- Prefer `std::string_view` over `const std::string&` for read-only parameters
- Use structured bindings: `auto [key, value] = map_entry;`
- Prefer `std::optional` over sentinel values or output parameters
- Use `constexpr` wherever possible

## Naming

- Types, classes, enums: `PascalCase`
- Functions, variables: `snake_case` or `camelCase` (be consistent per project)
- Constants and macros: `SCREAMING_SNAKE_CASE`
- Namespaces: `lowercase`

## Memory Safety

- Prefer smart pointers (`std::unique_ptr`, `std::shared_ptr`) over raw pointers
- RAII for all resource management — no manual `new`/`delete`
- Use `std::span` for non-owning array views (C++20)
- Prefer `std::array` over C-style arrays

## Error Handling

- Use exceptions for truly exceptional cases
- Prefer `std::expected` (C++23) or `Result`-type patterns for expected failures
- Never catch and silently ignore exceptions

## Reference

See skill: `cpp-coding-standards` for comprehensive C++ idioms and patterns.
