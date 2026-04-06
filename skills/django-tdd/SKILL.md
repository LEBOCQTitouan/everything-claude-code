---
name: django-tdd
description: Django testing strategies with pytest-django, TDD methodology, factory_boy, mocking, coverage, and testing Django REST Framework APIs.
origin: ECC
---

# Django Testing with TDD

## When to Activate

- Writing/testing Django apps or DRF APIs
- Setting up Django test infrastructure

## Setup

### pytest.ini

```ini
[pytest]
DJANGO_SETTINGS_MODULE = config.settings.test
testpaths = tests
addopts = --reuse-db --nomigrations --cov=apps --cov-report=term-missing --strict-markers
markers =
    slow: marks tests as slow
    integration: marks tests as integration tests
```

### Test Settings

```python
# config/settings/test.py
from .base import *
DATABASES = {'default': {'ENGINE': 'django.db.backends.sqlite3', 'NAME': ':memory:'}}

class DisableMigrations:
    def __contains__(self, item): return True
    def __getitem__(self, item): return None

MIGRATION_MODULES = DisableMigrations()
PASSWORD_HASHERS = ['django.contrib.auth.hashers.MD5PasswordHasher']
EMAIL_BACKEND = 'django.core.mail.backends.console.EmailBackend'
CELERY_TASK_ALWAYS_EAGER = True
```

### conftest.py

```python
@pytest.fixture
def user(db):
    return User.objects.create_user(email='test@example.com', password='testpass123', username='testuser')

@pytest.fixture
def authenticated_client(client, user):
    client.force_login(user)
    return client

@pytest.fixture
def api_client():
    from rest_framework.test import APIClient
    return APIClient()

@pytest.fixture
def authenticated_api_client(api_client, user):
    api_client.force_authenticate(user=user)
    return api_client
```

## Factory Boy

```python
class UserFactory(factory.django.DjangoModelFactory):
    class Meta:
        model = User
    email = factory.Sequence(lambda n: f"user{n}@example.com")
    username = factory.Sequence(lambda n: f"user{n}")
    password = factory.PostGenerationMethodCall('set_password', 'testpass123')

class ProductFactory(factory.django.DjangoModelFactory):
    class Meta:
        model = Product
    name = factory.Faker('sentence', nb_words=3)
    slug = factory.LazyAttribute(lambda obj: obj.name.lower().replace(' ', '-'))
    price = fuzzy.FuzzyDecimal(10.00, 1000.00, 2)
    stock = fuzzy.FuzzyInteger(0, 100)
    is_active = True
    category = factory.SubFactory(CategoryFactory)

    @factory.post_generation
    def tags(self, create, extracted, **kwargs):
        if create and extracted:
            for tag in extracted:
                self.tags.add(tag)
```

## Model Testing

```python
class TestProductModel:
    def test_product_creation(self, db):
        product = ProductFactory()
        assert product.id is not None
        assert product.is_active is True

    def test_product_slug_generation(self, db):
        product = ProductFactory(name='Test Product')
        assert product.slug == 'test-product'

    def test_product_price_validation(self, db):
        product = ProductFactory(price=-10)
        with pytest.raises(ValidationError):
            product.full_clean()

    def test_product_manager_active(self, db):
        ProductFactory.create_batch(5, is_active=True)
        ProductFactory.create_batch(3, is_active=False)
        assert Product.objects.active().count() == 5
```

## View Testing

```python
class TestProductViews:
    def test_product_list(self, client, db):
        ProductFactory.create_batch(10)
        response = client.get(reverse('products:list'))
        assert response.status_code == 200
        assert len(response.context['products']) == 10

    def test_product_create_requires_login(self, client, db):
        response = client.get(reverse('products:create'))
        assert response.status_code == 302
```

## DRF API Testing

```python
class TestProductAPI:
    def test_list_products(self, api_client, db):
        ProductFactory.create_batch(10)
        response = api_client.get(reverse('api:product-list'))
        assert response.status_code == status.HTTP_200_OK
        assert response.data['count'] == 10

    def test_create_product_unauthorized(self, api_client, db):
        response = api_client.post(reverse('api:product-list'), {'name': 'Test', 'price': '99.99'})
        assert response.status_code == status.HTTP_401_UNAUTHORIZED

    def test_create_product_authorized(self, authenticated_api_client, db):
        data = {'name': 'Test Product', 'price': '99.99', 'stock': 10}
        response = authenticated_api_client.post(reverse('api:product-list'), data)
        assert response.status_code == status.HTTP_201_CREATED

    def test_filter_products_by_price(self, api_client, db):
        ProductFactory(price=50)
        ProductFactory(price=150)
        response = api_client.get(reverse('api:product-list'), {'price_min': 100})
        assert response.data['count'] == 1
```

## Mocking External Services

```python
@patch('apps.payments.services.stripe')
def test_successful_payment(mock_stripe, client, user, product):
    mock_stripe.Charge.create.return_value = {'id': 'ch_123', 'status': 'succeeded'}
    client.force_login(user)
    response = client.post(reverse('payments:process'), {'product_id': product.id, 'token': 'tok_visa'})
    assert response.status_code == 302
    mock_stripe.Charge.create.assert_called_once()

# Email testing
@override_settings(EMAIL_BACKEND='django.core.mail.backends.locmem.EmailBackend')
def test_order_confirmation_email(db, order):
    order.send_confirmation_email()
    assert len(mail.outbox) == 1
    assert 'Order Confirmation' in mail.outbox[0].subject
```

## Best Practices

**DO**: Use factories, one assertion per test, descriptive names, mock external services, test permissions, use `--reuse-db`

**DON'T**: Test Django internals, test third-party code, make tests dependent, over-mock, test private methods

## Coverage Goals

| Component | Target |
|-----------|--------|
| Models | 90%+ |
| Serializers | 85%+ |
| Views/Services | 80-90%+ |
| Overall | 80%+ |

## Quick Reference

| Pattern | Usage |
|---------|-------|
| `@pytest.mark.django_db` | Enable DB access |
| `factory.create_batch(n)` | Create multiple objects |
| `patch('module.function')` | Mock dependencies |
| `override_settings` | Change settings in test |
| `force_authenticate()` | Bypass auth in tests |
| `mail.outbox` | Check sent emails |
