---
name: python-testing
description: Python testing strategies using pytest, TDD methodology, fixtures, mocking, parametrization, and coverage requirements.
origin: ECC
---

# Python Testing Patterns

## When to Activate

- Writing/testing Python code (TDD: red, green, refactor)
- Setting up pytest infrastructure or reviewing coverage

## TDD Cycle

1. **RED**: Write failing test
2. **GREEN**: Minimal code to pass
3. **REFACTOR**: Improve, keep green

**Coverage target**: 80%+ overall, 100% critical paths.

```bash
pytest --cov=mypackage --cov-report=term-missing --cov-report=html
```

## pytest Fundamentals

### Assertions

```python
assert result == expected
assert result != unexpected
assert result is None
assert item in collection
assert isinstance(result, str)

# Exception testing
with pytest.raises(ValueError, match="invalid input"):
    raise ValueError("invalid input provided")

with pytest.raises(ValueError) as exc_info:
    raise ValueError("error message")
assert str(exc_info.value) == "error message"
```

## Fixtures

```python
@pytest.fixture
def sample_data():
    return {"name": "Alice", "age": 30}

# Setup/teardown
@pytest.fixture
def database():
    db = Database(":memory:")
    db.create_tables()
    yield db
    db.close()
```

### Fixture Scopes

```python
@pytest.fixture                       # function (default)
@pytest.fixture(scope="module")       # once per module
@pytest.fixture(scope="session")      # once per session
```

### Parameterized & Autouse Fixtures

```python
@pytest.fixture(params=[1, 2, 3])
def number(request):
    return request.param

@pytest.fixture(autouse=True)
def reset_config():
    Config.reset()
    yield
    Config.cleanup()
```

### conftest.py

```python
# tests/conftest.py
@pytest.fixture
def client():
    app = create_app(testing=True)
    with app.test_client() as client:
        yield client

@pytest.fixture
def auth_headers(client):
    response = client.post("/api/login", json={"username": "test", "password": "test"})
    return {"Authorization": f"Bearer {response.json['token']}"}
```

## Parametrization

```python
@pytest.mark.parametrize("input,expected", [
    ("hello", "HELLO"),
    ("world", "WORLD"),
    ("PyThOn", "PYTHON"),
])
def test_uppercase(input, expected):
    assert input.upper() == expected

@pytest.mark.parametrize("input,expected", [
    ("valid@email.com", True),
    ("invalid", False),
], ids=["valid-email", "missing-at"])
def test_email_validation(input, expected):
    assert is_valid_email(input) is expected
```

## Markers

```python
@pytest.mark.slow
@pytest.mark.integration
@pytest.mark.unit
```

```bash
pytest -m "not slow"
pytest -m "integration or slow"
```

```ini
[pytest]
markers =
    slow: marks tests as slow
    integration: marks tests as integration tests
```

## Mocking

```python
from unittest.mock import patch, Mock, mock_open, PropertyMock

@patch("mypackage.external_api_call")
def test_with_mock(api_call_mock):
    api_call_mock.return_value = {"status": "success"}
    result = my_function()
    api_call_mock.assert_called_once()

@patch("mypackage.api_call")
def test_api_error(api_call_mock):
    api_call_mock.side_effect = ConnectionError("Network error")
    with pytest.raises(ConnectionError):
        api_call()

@patch("builtins.open", new_callable=mock_open)
def test_file_reading(mock_file):
    mock_file.return_value.read.return_value = "file content"
    result = read_file("test.txt")
    assert result == "file content"

# Autospec catches API misuse
@patch("mypackage.DBConnection", autospec=True)
def test_autospec(db_mock):
    db = db_mock.return_value
    db.query("SELECT * FROM users")
```

## Async Tests

```python
@pytest.mark.asyncio
async def test_async_function():
    result = await async_add(2, 3)
    assert result == 5

@pytest.fixture
async def async_client():
    app = create_app()
    async with app.test_client() as client:
        yield client
```

## Testing Side Effects

```python
def test_with_tmp_path(tmp_path):
    test_file = tmp_path / "test.txt"
    test_file.write_text("hello world")
    result = process_file(str(test_file))
    assert result == "hello world"
```

## Test Organization

```
tests/
├── conftest.py
├── unit/
│   ├── test_models.py
│   └── test_services.py
├── integration/
│   └── test_api.py
└── e2e/
    └── test_user_flow.py
```

## Common Patterns

### API Endpoint Testing

```python
def test_get_user(client):
    response = client.get("/api/users/1")
    assert response.status_code == 200
    assert response.json["id"] == 1

def test_create_user(client):
    response = client.post("/api/users", json={"name": "Alice", "email": "alice@example.com"})
    assert response.status_code == 201
```

### Database Testing

```python
@pytest.fixture
def db_session():
    session = Session(bind=engine)
    session.begin_nested()
    yield session
    session.rollback()
    session.close()
```

## Configuration

```toml
[tool.pytest.ini_options]
testpaths = ["tests"]
addopts = ["--strict-markers", "--cov=mypackage", "--cov-report=term-missing"]
markers = ["slow: marks tests as slow", "integration: integration tests"]
```

## Running Tests

```bash
pytest                              # all tests
pytest tests/test_utils.py          # specific file
pytest -v                           # verbose
pytest --cov=mypackage              # with coverage
pytest -m "not slow"                # skip slow
pytest -x                           # stop on first failure
pytest --lf                         # last failed only
pytest -k "test_user"               # pattern match
pytest --pdb                        # debugger on failure
```

## Quick Reference

| Pattern | Usage |
|---------|-------|
| `pytest.raises()` | Test expected exceptions |
| `@pytest.fixture()` | Reusable test fixtures |
| `@pytest.mark.parametrize()` | Multiple inputs |
| `@patch()` | Mock functions/classes |
| `tmp_path` | Automatic temp directory |
| `pytest --cov` | Coverage report |
