---
name: django-verification
description: "Verification loop for Django projects: migrations, linting, tests with coverage, security scans, and deployment readiness checks before release or PR."
origin: ECC
---

# Django Verification Loop

Run before PRs, after major changes, and pre-deploy.

## When to Activate

- Before PRs, after model/migration/dependency changes
- Pre-deployment verification

## Phase 1: Environment Check

```bash
python --version
which python && pip list --outdated
```

## Phase 2: Code Quality

```bash
mypy . --config-file pyproject.toml
ruff check . --fix
black . --check && black .
isort . --check-only && isort .
python manage.py check --deploy
```

## Phase 3: Migrations

```bash
python manage.py showmigrations
python manage.py makemigrations --check
python manage.py migrate --plan
python manage.py migrate
```

## Phase 4: Tests + Coverage

```bash
pytest --cov=apps --cov-report=html --cov-report=term-missing --reuse-db
```

| Component | Target |
|-----------|--------|
| Models | 90%+ |
| Serializers | 85%+ |
| Views/Services | 80-90%+ |
| Overall | 80%+ |

## Phase 5: Security

```bash
pip-audit
safety check --full-report
python manage.py check --deploy
bandit -r . -f json -o bandit-report.json
gitleaks detect --source . --verbose
```

## Phase 6: Django Commands

```bash
python manage.py check
python manage.py collectstatic --noinput --clear
python manage.py check --database default
```

## Phase 7: Performance

Check for N+1 queries, missing indexes, duplicate queries (< 50 queries per page).

## Phase 8: Configuration Review

```python
checks = {
    'DEBUG is False': not settings.DEBUG,
    'SECRET_KEY set': bool(settings.SECRET_KEY and len(settings.SECRET_KEY) > 30),
    'ALLOWED_HOSTS set': len(settings.ALLOWED_HOSTS) > 0,
    'HTTPS enabled': getattr(settings, 'SECURE_SSL_REDIRECT', False),
    'HSTS enabled': getattr(settings, 'SECURE_HSTS_SECONDS', 0) > 0,
}
```

## Phase 9: Diff Review

```bash
git diff --stat
git diff | grep -i "todo\|fixme\|hack"
git diff | grep "print(\|DEBUG = True\|import pdb"
```

Checklist: No debug statements, no hardcoded secrets, migrations included, error handling present, transactions where needed.

## Quick Reference

| Check | Command |
|-------|---------|
| Type checking | `mypy .` |
| Linting | `ruff check .` |
| Formatting | `black . --check` |
| Migrations | `python manage.py makemigrations --check` |
| Tests | `pytest --cov=apps` |
| Security | `pip-audit && bandit -r .` |
| Django check | `python manage.py check --deploy` |
