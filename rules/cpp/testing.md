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
# C/C++ Testing

> This file extends [common/testing.md](../common/testing.md) with C/C++ specific content.

## Frameworks

- **GoogleTest (gtest)** for unit and integration tests
- **CTest** for test orchestration via CMake
- **Catch2** as alternative

## Sanitizers

Always run tests with sanitizers enabled:

```bash
cmake -DCMAKE_CXX_FLAGS="-fsanitize=address,undefined" ..
ctest --output-on-failure
```

## Coverage

```bash
cmake -DCMAKE_CXX_FLAGS="--coverage" ..
make && ctest
lcov --capture --directory . --output-file coverage.info
```

## Reference

See skill: `cpp-testing` for detailed C++ testing patterns and helpers.
