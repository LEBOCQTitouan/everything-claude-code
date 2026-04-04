---
paths:
  - "**/*.c"
  - "**/*.cpp"
  - "**/*.h"
  - "**/*.hpp"
  - "**/*.cc"
  - "**/*.cxx"
  - "**/CMakeLists.txt"
applies-to: { languages: [cpp] }
---
# C/C++ Security

> This file extends [common/security.md](../common/security.md) with C/C++ specific content.

## Buffer Safety

- Never use `strcpy`, `sprintf`, `gets` — use `strncpy`, `snprintf`, `fgets`
- Prefer `std::string` and `std::vector` over raw buffers
- Validate all buffer sizes before operations

## Memory Safety

- Use AddressSanitizer (ASan) in CI: `-fsanitize=address`
- Use MemorySanitizer for uninitialized reads: `-fsanitize=memory`
- Run Valgrind for leak detection in integration tests

## Static Analysis

- Use **cppcheck** for static bug detection:
  ```bash
  cppcheck --enable=all --error-exitcode=1 src/
  ```
- Use **clang-tidy** with security-focused checks:
  ```bash
  clang-tidy -checks='bugprone-*,cert-*,security-*' src/*.cpp
  ```

## Integer Safety

- Check for integer overflow before arithmetic on untrusted input
- Use `<cstdint>` fixed-width types (`int32_t`, `uint64_t`)
- Avoid implicit narrowing conversions
