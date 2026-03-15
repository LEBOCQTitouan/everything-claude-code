---
name: yaml-patterns
description: YAML best practices including yamllint configuration, anchors/aliases, schema validation, and CI/CD pipeline patterns.
origin: ECC
---

# YAML Patterns

Best practices for writing clean, safe, and maintainable YAML files.

## When to Activate

- Writing or reviewing YAML configuration files
- Setting up CI/CD pipelines (GitHub Actions, GitLab CI, etc.)
- Configuring Docker Compose, Kubernetes, or Helm charts
- Debugging YAML parsing issues

## Linting with yamllint

### Configuration (`.yamllint.yml`)

```yaml
extends: default
rules:
  line-length:
    max: 120
    allow-non-breakable-words: true
  truthy:
    check-keys: true
    allowed-values: ['true', 'false', 'yes', 'no']
  comments:
    require-starting-space: true
    min-spaces-from-content: 1
  indentation:
    spaces: 2
    indent-sequences: true
```

### Running

```bash
yamllint .
yamllint -d relaxed file.yml
```

## GitHub Actions Patterns

```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      - run: npm ci
      - run: npm test
```

## Docker Compose Patterns

```yaml
services:
  app:
    build:
      context: .
      dockerfile: Dockerfile
    env_file: .env
    ports:
      - "${PORT:-3000}:3000"
    depends_on:
      db:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: ${DB_NAME}
      POSTGRES_PASSWORD: ${DB_PASSWORD}
    volumes:
      - pgdata:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 5s

volumes:
  pgdata:
```

## Kubernetes Patterns

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app
  labels:
    app.kubernetes.io/name: app
    app.kubernetes.io/version: "1.0.0"
spec:
  replicas: 3
  selector:
    matchLabels:
      app.kubernetes.io/name: app
  template:
    metadata:
      labels:
        app.kubernetes.io/name: app
    spec:
      containers:
        - name: app
          image: app:1.0.0
          ports:
            - containerPort: 8080
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 512Mi
          livenessProbe:
            httpGet:
              path: /healthz
              port: 8080
            initialDelaySeconds: 15
          readinessProbe:
            httpGet:
              path: /readyz
              port: 8080
```

## Quick Reference

| Tool | Purpose |
|------|---------|
| `yamllint` | YAML linting and style checking |
| `yq` | Command-line YAML processor (jq for YAML) |
| `kubeval` | Kubernetes YAML validation |
| `actionlint` | GitHub Actions workflow linting |
| `docker compose config` | Validate Docker Compose files |
