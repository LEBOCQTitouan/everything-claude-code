---
name: django-security
description: Django security best practices, authentication, authorization, CSRF protection, SQL injection prevention, XSS prevention, and secure deployment configurations.
origin: ECC
---

# Django Security

## When to Activate

- Setting up Django auth, permissions, production security
- Reviewing Django apps for security issues

## Production Settings

```python
DEBUG = False
ALLOWED_HOSTS = os.environ.get('ALLOWED_HOSTS', '').split(',')
SECURE_SSL_REDIRECT = True
SESSION_COOKIE_SECURE = True
CSRF_COOKIE_SECURE = True
SECURE_HSTS_SECONDS = 31536000
SECURE_HSTS_INCLUDE_SUBDOMAINS = True
SECURE_HSTS_PRELOAD = True
SECURE_CONTENT_TYPE_NOSNIFF = True
X_FRAME_OPTIONS = 'DENY'
SESSION_COOKIE_HTTPONLY = True
CSRF_COOKIE_HTTPONLY = True
SESSION_COOKIE_SAMESITE = 'Lax'

SECRET_KEY = os.environ.get('DJANGO_SECRET_KEY')
if not SECRET_KEY:
    raise ImproperlyConfigured('DJANGO_SECRET_KEY required')

PASSWORD_HASHERS = [
    'django.contrib.auth.hashers.Argon2PasswordHasher',
    'django.contrib.auth.hashers.PBKDF2PasswordHasher',
]
```

## Custom User Model

```python
class User(AbstractUser):
    email = models.EmailField(unique=True)
    USERNAME_FIELD = 'email'
    REQUIRED_FIELDS = ['username']
# settings: AUTH_USER_MODEL = 'users.User'
```

## DRF Permissions

```python
class IsOwnerOrReadOnly(permissions.BasePermission):
    def has_object_permission(self, request, view, obj):
        if request.method in permissions.SAFE_METHODS: return True
        return obj.author == request.user

class IsVerifiedUser(permissions.BasePermission):
    def has_permission(self, request, view):
        return request.user and request.user.is_authenticated and request.user.is_verified
```

## RBAC

```python
class User(AbstractUser):
    ROLE_CHOICES = [('admin', 'Administrator'), ('moderator', 'Moderator'), ('user', 'Regular User')]
    role = models.CharField(max_length=20, choices=ROLE_CHOICES, default='user')
    def is_admin(self): return self.role == 'admin' or self.is_superuser
    def is_moderator(self): return self.role in ['admin', 'moderator']
```

## SQL Injection Prevention

```python
# SAFE: ORM
User.objects.get(username=username)
User.objects.filter(Q(username__icontains=query) | Q(email__icontains=query))

# SAFE: Parameterized raw SQL
User.objects.raw('SELECT * FROM users WHERE email = %s', [user_input_email])

# VULNERABLE — NEVER: f'SELECT * FROM users WHERE username = {username}'
```

## XSS Prevention

```django
{{ user_input }}           {# Auto-escaped (safe) #}
{{ trusted_html|safe }}    {# Only for trusted content #}
{{ user_input|striptags }} {# Remove all HTML #}
```

```python
from django.utils.html import format_html
format_html('<span class="user">{}</span>', username)  # GOOD: auto-escapes
# NEVER: mark_safe(user_input)
```

## CSRF Protection

```python
CSRF_COOKIE_SECURE = True
CSRF_COOKIE_HTTPONLY = True
CSRF_TRUSTED_ORIGINS = ['https://example.com']
```

```html
<form method="post">{% csrf_token %}{{ form.as_p }}<button>Submit</button></form>
```

AJAX: include `X-CSRFToken` header from `csrftoken` cookie.

## File Upload Security

```python
def validate_file_extension(value):
    ext = os.path.splitext(value.name)[1]
    if ext.lower() not in ['.jpg', '.jpeg', '.png', '.gif', '.pdf']:
        raise ValidationError('Unsupported file extension.')

def validate_file_size(value):
    if value.size > 5 * 1024 * 1024:
        raise ValidationError('File too large. Max 5MB.')
```

## API Rate Limiting

```python
REST_FRAMEWORK = {
    'DEFAULT_THROTTLE_CLASSES': [
        'rest_framework.throttling.AnonRateThrottle',
        'rest_framework.throttling.UserRateThrottle'
    ],
    'DEFAULT_THROTTLE_RATES': { 'anon': '100/day', 'user': '1000/day' }
}
```

## CSP Middleware

```python
class CSPMiddleware:
    def __init__(self, get_response): self.get_response = get_response
    def __call__(self, request):
        response = self.get_response(request)
        response['Content-Security-Policy'] = "default-src 'self'"
        response['X-Content-Type-Options'] = 'nosniff'
        return response
```

## Quick Security Checklist

| Check | Description |
|-------|-------------|
| `DEBUG = False` | Never True in production |
| HTTPS only | Force SSL, secure cookies |
| Strong secrets | Env vars for SECRET_KEY |
| CSRF enabled | Default on, don't disable |
| XSS prevention | Auto-escape, no `|safe` with user input |
| SQL injection | Use ORM, never concatenate |
| File uploads | Validate type and size |
| Rate limiting | Throttle API endpoints |
| Security headers | CSP, X-Frame-Options, HSTS |
