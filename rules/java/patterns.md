---
paths:
  - "**/*.java"
  - "**/pom.xml"
  - "**/build.gradle"
  - "**/build.gradle.kts"
applies-to: { languages: [java] }
---
# Java Patterns

> This file extends [common/patterns.md](../common/patterns.md) with Java specific content.

## Builder Pattern

```java
public record CreateUserRequest(String name, String email, int age) {
    public static Builder builder() { return new Builder(); }

    public static class Builder {
        private String name;
        private String email;
        private int age;

        public Builder name(String name) { this.name = name; return this; }
        public Builder email(String email) { this.email = email; return this; }
        public Builder age(int age) { this.age = age; return this; }
        public CreateUserRequest build() { return new CreateUserRequest(name, email, age); }
    }
}
```

## Repository Pattern

Use Spring Data interfaces:

```java
public interface UserRepository extends JpaRepository<User, Long> {
    Optional<User> findByEmail(String email);
    List<User> findByActiveTrue();
}
```

## Dependency Injection

Constructor injection is the default:

```java
@Service
public class UserService {
    private final UserRepository repository;
    private final EmailService emailService;

    public UserService(UserRepository repository, EmailService emailService) {
        this.repository = repository;
        this.emailService = emailService;
    }
}
```

## Reference

See skill: `springboot-patterns` and skill: `jpa-patterns` for comprehensive Java patterns.
