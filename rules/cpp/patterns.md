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
# C/C++ Patterns

> This file extends [common/patterns.md](../common/patterns.md) with C/C++ specific content.

## RAII (Resource Acquisition Is Initialization)

All resources must be managed via constructors/destructors:

```cpp
class FileHandle {
    std::FILE* file_;
public:
    explicit FileHandle(const char* path) : file_(std::fopen(path, "r")) {
        if (!file_) throw std::runtime_error("Cannot open file");
    }
    ~FileHandle() { if (file_) std::fclose(file_); }
    FileHandle(const FileHandle&) = delete;
    FileHandle& operator=(const FileHandle&) = delete;
};
```

## Smart Pointer Ownership

- `std::unique_ptr` for exclusive ownership (default choice)
- `std::shared_ptr` only when shared ownership is genuinely needed
- Raw pointers only for non-owning observation

## Dependency Injection

Use constructor injection:

```cpp
class UserService {
    std::unique_ptr<UserRepository> repo_;
public:
    explicit UserService(std::unique_ptr<UserRepository> repo)
        : repo_(std::move(repo)) {}
};
```

## Reference

See skill: `cpp-coding-standards` for comprehensive C++ patterns including templates, concurrency, and STL usage.
