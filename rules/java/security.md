---
paths:
  - "**/*.java"
  - "**/pom.xml"
  - "**/build.gradle"
  - "**/build.gradle.kts"
applies-to: { languages: [java] }
---
# Java Security

> This file extends [common/security.md](../common/security.md) with Java specific content.

## SQL Injection Prevention

Always use parameterized queries:

```java
// GOOD
PreparedStatement ps = conn.prepareStatement("SELECT * FROM users WHERE id = ?");
ps.setString(1, userId);

// BAD
Statement s = conn.createStatement();
s.executeQuery("SELECT * FROM users WHERE id = '" + userId + "'");
```

## Dependency Scanning

- Use **OWASP Dependency-Check** in CI:
  ```bash
  mvn org.owasp:dependency-check-maven:check
  ```
- Use **SpotBugs** with **Find Security Bugs** plugin for static analysis

## Deserialization

- Never deserialize untrusted data with `ObjectInputStream`
- Use JSON/protobuf for serialization instead
- If Java serialization is required, use allowlists

## Secrets

```java
String apiKey = System.getenv("API_KEY");
if (apiKey == null || apiKey.isBlank()) {
    throw new IllegalStateException("API_KEY not configured");
}
```
