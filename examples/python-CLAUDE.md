# Python — Project CLAUDE.md

> Generic Python project template.
> Copy this to your project root and customize for your project.

## Project Overview

**Stack:** Python 3.12+, pytest, ruff, mypy, virtualenv/uv

**Architecture:** [Describe your architecture — layered, hexagonal, script-based, etc.]

## Critical Rules

### Python Conventions

- Type annotations required on all public functions and methods
- Use `pathlib.Path` over `os.path` for file operations
- Prefer dataclasses or Pydantic models over plain dicts for structured data
- No mutable default arguments — use `None` and initialize inside the function
- Use `logging` module, never `print()` in production code

### Code Style

- Follow PEP 8; ruff enforces it automatically
- No emojis in code, comments, or documentation
- Single responsibility — keep functions under 30 lines
- Explicit is better than implicit — no magic, no monkey-patching
- All exceptions must be typed (`except ValueError` not bare `except`)

### Testing

- TDD: Write tests first with pytest
- 80% minimum coverage (`pytest --cov`)
- Use `pytest.mark.parametrize` for table-driven tests
- Fixtures in `conftest.py` — no shared mutable state between tests
- Mock external I/O; never make real network calls in unit tests

### Security

- No hardcoded secrets — use environment variables or `.env` with python-dotenv
- Validate all external input with Pydantic or manual checks
- Parameterized queries — never f-string SQL
- Keep dependencies pinned in `requirements.txt` / `pyproject.toml`

## File Structure

```
src/
  mypackage/
    domain/        # Business types, entities, interfaces
    services/      # Business logic
    repositories/  # Data access
    api/           # HTTP or CLI entry points
    config.py      # Settings loaded from environment
tests/
  unit/
  integration/
  conftest.py
pyproject.toml
```

## Key Patterns

### Settings with Pydantic

```python
from pydantic_settings import BaseSettings

class Settings(BaseSettings):
    database_url: str
    debug: bool = False
    log_level: str = "info"

    class Config:
        env_file = ".env"

settings = Settings()
```

### Typed Error Hierarchy

```python
class AppError(Exception):
    """Base application error."""

class NotFoundError(AppError):
    """Resource not found."""

class ValidationError(AppError):
    """Input validation failed."""
```

### Parametrized Tests

```python
import pytest

@pytest.mark.parametrize("input,expected", [
    ("hello", "HELLO"),
    ("world", "WORLD"),
])
def test_upper(input: str, expected: str) -> None:
    assert input.upper() == expected
```

## Environment Variables

```bash
# Required
DATABASE_URL=

# Optional
DEBUG=false
LOG_LEVEL=info
```

## Available Commands

```bash
# Setup
python -m venv .venv && source .venv/bin/activate
pip install -e ".[dev]"

# Type checking
mypy src/

# Lint and format
ruff check src/ tests/
ruff format src/ tests/

# Test
pytest                          # Run all tests
pytest --cov=src --cov-report=term-missing   # With coverage
pytest -x                       # Stop on first failure
pytest -k "test_user"           # Filter by name

# Build / package
python -m build
```

## ECC Workflow

```bash
/plan          # Implementation planning (includes TDD workflow)
/verify        # Quality review
/build-fix     # Fix import / type errors
```

## Git Workflow

- Conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`
- Never commit to main directly
- PRs require review and passing CI
- CI: `mypy`, `ruff check`, `pytest --cov`
