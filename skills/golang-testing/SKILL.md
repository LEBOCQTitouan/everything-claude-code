---
name: golang-testing
description: Go testing patterns including table-driven tests, subtests, benchmarks, fuzzing, and test coverage. Follows TDD methodology with idiomatic Go practices.
origin: ECC
---

# Go Testing Patterns

## When to Activate

- Writing/testing Go code, benchmarks, or fuzz tests
- Following TDD workflow in Go projects

## TDD Cycle

```
RED → Write failing test | GREEN → Minimal pass | REFACTOR → Improve, keep green
```

## Table-Driven Tests

```go
func TestAdd(t *testing.T) {
    tests := []struct {
        name     string
        a, b     int
        expected int
    }{
        {"positive numbers", 2, 3, 5},
        {"negative numbers", -1, -2, -3},
        {"zero values", 0, 0, 0},
        {"mixed signs", -1, 1, 0},
    }
    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            if got := Add(tt.a, tt.b); got != tt.expected {
                t.Errorf("Add(%d, %d) = %d; want %d", tt.a, tt.b, got, tt.expected)
            }
        })
    }
}
```

### With Error Cases

```go
func TestParseConfig(t *testing.T) {
    tests := []struct {
        name    string
        input   string
        want    *Config
        wantErr bool
    }{
        {"valid config", `{"host": "localhost", "port": 8080}`, &Config{Host: "localhost", Port: 8080}, false},
        {"invalid JSON", `{invalid}`, nil, true},
        {"empty input", "", nil, true},
    }
    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            got, err := ParseConfig(tt.input)
            if tt.wantErr {
                if err == nil { t.Error("expected error, got nil") }
                return
            }
            if err != nil { t.Fatalf("unexpected error: %v", err) }
            if !reflect.DeepEqual(got, tt.want) { t.Errorf("got %+v; want %+v", got, tt.want) }
        })
    }
}
```

## Subtests

```go
func TestUser(t *testing.T) {
    db := setupTestDB(t)
    t.Run("Create", func(t *testing.T) { /* ... */ })
    t.Run("Get", func(t *testing.T) { /* ... */ })
}

// Parallel subtests
for _, tt := range tests {
    tt := tt
    t.Run(tt.name, func(t *testing.T) {
        t.Parallel()
        result := Process(tt.input)
        _ = result
    })
}
```

## Test Helpers

```go
func setupTestDB(t *testing.T) *sql.DB {
    t.Helper()
    db, err := sql.Open("sqlite3", ":memory:")
    if err != nil { t.Fatalf("failed to open database: %v", err) }
    t.Cleanup(func() { db.Close() })
    if _, err := db.Exec(schema); err != nil { t.Fatalf("failed to create schema: %v", err) }
    return db
}

func assertEqual[T comparable](t *testing.T, got, want T) {
    t.Helper()
    if got != want { t.Errorf("got %v; want %v", got, want) }
}
```

## Golden Files

```go
var update = flag.Bool("update", false, "update golden files")

func TestRender(t *testing.T) {
    got := Render(input)
    golden := filepath.Join("testdata", name+".golden")
    if *update {
        os.WriteFile(golden, got, 0644)
    }
    want, _ := os.ReadFile(golden)
    if !bytes.Equal(got, want) { t.Errorf("output mismatch") }
}
```

## Interface-Based Mocking

```go
type UserRepository interface {
    GetUser(id string) (*User, error)
    SaveUser(user *User) error
}

type MockUserRepository struct {
    GetUserFunc  func(id string) (*User, error)
    SaveUserFunc func(user *User) error
}

func (m *MockUserRepository) GetUser(id string) (*User, error) { return m.GetUserFunc(id) }
func (m *MockUserRepository) SaveUser(user *User) error        { return m.SaveUserFunc(user) }

func TestUserService(t *testing.T) {
    mock := &MockUserRepository{
        GetUserFunc: func(id string) (*User, error) {
            if id == "123" { return &User{ID: "123", Name: "Alice"}, nil }
            return nil, ErrNotFound
        },
    }
    service := NewUserService(mock)
    user, err := service.GetUserProfile("123")
    if err != nil { t.Fatalf("unexpected error: %v", err) }
    if user.Name != "Alice" { t.Errorf("got %q; want Alice", user.Name) }
}
```

## Benchmarks

```go
func BenchmarkProcess(b *testing.B) {
    data := generateTestData(1000)
    b.ResetTimer()
    for i := 0; i < b.N; i++ { Process(data) }
}

// Size comparison
func BenchmarkSort(b *testing.B) {
    for _, size := range []int{100, 1000, 10000} {
        b.Run(fmt.Sprintf("size=%d", size), func(b *testing.B) {
            data := generateRandomSlice(size)
            b.ResetTimer()
            for i := 0; i < b.N; i++ {
                tmp := make([]int, len(data))
                copy(tmp, data)
                sort.Ints(tmp)
            }
        })
    }
}
```

## Fuzzing (Go 1.18+)

```go
func FuzzParseJSON(f *testing.F) {
    f.Add(`{"name": "test"}`)
    f.Add(`[]`)
    f.Fuzz(func(t *testing.T, input string) {
        var result map[string]interface{}
        if err := json.Unmarshal([]byte(input), &result); err != nil { return }
        if _, err := json.Marshal(result); err != nil {
            t.Errorf("Marshal failed after Unmarshal: %v", err)
        }
    })
}
```

## HTTP Handler Testing

```go
func TestHealthHandler(t *testing.T) {
    req := httptest.NewRequest(http.MethodGet, "/health", nil)
    w := httptest.NewRecorder()
    HealthHandler(w, req)
    if w.Code != http.StatusOK { t.Errorf("got %d; want 200", w.Code) }
}
```

## Coverage

```bash
go test -cover ./...
go test -coverprofile=coverage.out ./...
go tool cover -html=coverage.out
go test -race -coverprofile=coverage.out ./...
```

| Code Type | Target |
|-----------|--------|
| Critical business logic | 100% |
| Public APIs | 90%+ |
| General code | 80%+ |

## Commands

```bash
go test ./...                          # all tests
go test -v ./...                       # verbose
go test -run TestAdd ./...             # specific test
go test -race ./...                    # race detector
go test -bench=. -benchmem ./...       # benchmarks
go test -fuzz=FuzzParse -fuzztime=30s  # fuzzing
go test -count=10 ./...                # flaky detection
```

## Best Practices

**DO**: Write tests first (TDD), use table-driven tests, use `t.Helper()`, use `t.Parallel()`, clean up with `t.Cleanup()`

**DON'T**: Test private functions directly, use `time.Sleep()` in tests, ignore flaky tests, skip error path testing
