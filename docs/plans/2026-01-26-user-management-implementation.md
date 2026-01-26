# User Management Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement user management with subscriptions (Freemium + Teams), Stripe billing, and team management.

**Architecture:** Extend existing User model with subscription fields. Create new `billing` and `teams` Django apps. Use Stripe Checkout for payments and Customer Portal for subscription management. OAuth-only authentication (already implemented).

**Tech Stack:** Django 5.x, DRF, PostgreSQL, Stripe API, django-allauth (existing)

---

## Task 1: Add Stripe Dependency

**Files:**
- Modify: `backend/pyproject.toml`

**Step 1: Add stripe to dependencies**

In `backend/pyproject.toml`, add to `[project.dependencies]`:

```toml
stripe = ">=7.0.0"
```

**Step 2: Install dependencies**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && uv sync`

Expected: Dependencies installed successfully

**Step 3: Commit**

```bash
git add backend/pyproject.toml backend/uv.lock
git commit -m "chore: add stripe dependency"
```

---

## Task 2: Add Environment Variables to Settings

**Files:**
- Modify: `backend/config/settings/base.py`
- Modify: `backend/.env.example`

**Step 1: Add Stripe settings to base.py**

Add at the end of `backend/config/settings/base.py`:

```python
# Stripe
STRIPE_SECRET_KEY = os.environ.get("STRIPE_SECRET_KEY", "")
STRIPE_PUBLISHABLE_KEY = os.environ.get("STRIPE_PUBLISHABLE_KEY", "")
STRIPE_WEBHOOK_SECRET = os.environ.get("STRIPE_WEBHOOK_SECRET", "")

# Stripe Price IDs
STRIPE_PRICES = {
    "premium_monthly": os.environ.get("STRIPE_PRICE_PREMIUM_MONTHLY", ""),
    "premium_yearly": os.environ.get("STRIPE_PRICE_PREMIUM_YEARLY", ""),
    "teams_monthly": os.environ.get("STRIPE_PRICE_TEAMS_MONTHLY", ""),
    "teams_yearly": os.environ.get("STRIPE_PRICE_TEAMS_YEARLY", ""),
    "extra_seat_monthly": os.environ.get("STRIPE_PRICE_EXTRA_SEAT_MONTHLY", ""),
    "extra_seat_yearly": os.environ.get("STRIPE_PRICE_EXTRA_SEAT_YEARLY", ""),
}

# Super Admin
SUPER_ADMIN_EMAIL = os.environ.get("SUPER_ADMIN_EMAIL", "")
```

**Step 2: Update .env.example**

Add to `backend/.env.example`:

```bash
# Super Admin (created on first deploy)
SUPER_ADMIN_EMAIL=admin@example.com

# Stripe
STRIPE_SECRET_KEY=sk_test_xxx
STRIPE_PUBLISHABLE_KEY=pk_test_xxx
STRIPE_WEBHOOK_SECRET=whsec_xxx

# Stripe Price IDs (create in Stripe Dashboard)
STRIPE_PRICE_PREMIUM_MONTHLY=price_xxx
STRIPE_PRICE_PREMIUM_YEARLY=price_xxx
STRIPE_PRICE_TEAMS_MONTHLY=price_xxx
STRIPE_PRICE_TEAMS_YEARLY=price_xxx
STRIPE_PRICE_EXTRA_SEAT_MONTHLY=price_xxx
STRIPE_PRICE_EXTRA_SEAT_YEARLY=price_xxx
```

**Step 3: Commit**

```bash
git add backend/config/settings/base.py backend/.env.example
git commit -m "chore: add Stripe and super admin settings"
```

---

## Task 3: Extend User Model with Subscription Fields

**Files:**
- Modify: `backend/apps/users/models.py`
- Create: `backend/apps/users/migrations/0002_user_subscription_fields.py` (auto-generated)

**Step 1: Write the test for new fields**

Create `backend/apps/users/tests/test_models.py`:

```python
"""Tests for User model."""

import pytest
from django.contrib.auth import get_user_model
from django.utils import timezone

User = get_user_model()


@pytest.mark.django_db
class TestUserModel:
    """Tests for User model subscription fields."""

    def test_user_default_subscription_tier_is_free(self):
        """New users should have free tier by default."""
        user = User.objects.create_user(email="test@example.com", password="testpass123")
        assert user.subscription_tier == "free"

    def test_user_default_subscription_status_is_none(self):
        """New users should have no subscription status."""
        user = User.objects.create_user(email="test@example.com", password="testpass123")
        assert user.subscription_status == "none"

    def test_user_profile_fields_have_defaults(self):
        """Profile fields should have sensible defaults."""
        user = User.objects.create_user(email="test@example.com", password="testpass123")
        assert user.display_name == ""
        assert user.timezone == "UTC"
        assert user.language == "en"

    def test_user_stripe_customer_id_default_empty(self):
        """Stripe customer ID should be empty by default."""
        user = User.objects.create_user(email="test@example.com", password="testpass123")
        assert user.stripe_customer_id == ""

    def test_user_subscription_ends_at_default_none(self):
        """Subscription end date should be None by default."""
        user = User.objects.create_user(email="test@example.com", password="testpass123")
        assert user.subscription_ends_at is None
```

**Step 2: Run test to verify it fails**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/users/tests/test_models.py -v`

Expected: FAIL - fields don't exist yet

**Step 3: Update User model**

Replace content of `backend/apps/users/models.py`:

```python
"""Custom User model - email-based authentication."""

from django.contrib.auth.models import AbstractUser, BaseUserManager
from django.db import models


class UserManager(BaseUserManager):
    """Custom user manager for email-based authentication."""

    def create_user(self, email, password=None, **extra_fields):
        """Create and return a regular user."""
        if not email:
            raise ValueError("Email is required")
        email = self.normalize_email(email)
        user = self.model(email=email, **extra_fields)
        user.set_password(password)
        user.save(using=self._db)
        return user

    def create_superuser(self, email, password=None, **extra_fields):
        """Create and return a superuser."""
        extra_fields.setdefault("is_staff", True)
        extra_fields.setdefault("is_superuser", True)

        if extra_fields.get("is_staff") is not True:
            raise ValueError("Superuser must have is_staff=True")
        if extra_fields.get("is_superuser") is not True:
            raise ValueError("Superuser must have is_superuser=True")

        return self.create_user(email, password, **extra_fields)


class User(AbstractUser):
    """Custom user model - email-based, no username."""

    # Tier choices
    TIER_FREE = "free"
    TIER_PREMIUM = "premium"
    TIER_TEAMS = "teams"
    TIER_CHOICES = [
        (TIER_FREE, "Free"),
        (TIER_PREMIUM, "Premium"),
        (TIER_TEAMS, "Teams"),
    ]

    # Subscription status choices
    STATUS_NONE = "none"
    STATUS_ACTIVE = "active"
    STATUS_PAST_DUE = "past_due"
    STATUS_CANCELLED = "cancelled"
    STATUS_CHOICES = [
        (STATUS_NONE, "None"),
        (STATUS_ACTIVE, "Active"),
        (STATUS_PAST_DUE, "Past Due"),
        (STATUS_CANCELLED, "Cancelled"),
    ]

    # Auth fields
    username = None
    email = models.EmailField("email address", unique=True)

    # OAuth provider IDs
    google_id = models.CharField(max_length=255, blank=True, default="")
    discord_id = models.CharField(max_length=255, blank=True, default="")

    # Profile fields
    avatar_url = models.URLField(blank=True, default="")
    display_name = models.CharField(max_length=50, blank=True, default="")
    timezone = models.CharField(max_length=50, default="UTC")
    language = models.CharField(max_length=10, default="en")

    # Subscription fields
    stripe_customer_id = models.CharField(max_length=255, blank=True, default="")
    subscription_tier = models.CharField(
        max_length=20, choices=TIER_CHOICES, default=TIER_FREE
    )
    subscription_status = models.CharField(
        max_length=20, choices=STATUS_CHOICES, default=STATUS_NONE
    )
    subscription_ends_at = models.DateTimeField(null=True, blank=True)

    objects = UserManager()

    USERNAME_FIELD = "email"
    REQUIRED_FIELDS = []

    class Meta:
        db_table = "users"

    def __str__(self):
        return self.email

    @property
    def has_active_subscription(self):
        """Check if user has an active paid subscription."""
        return (
            self.subscription_tier in [self.TIER_PREMIUM, self.TIER_TEAMS]
            and self.subscription_status == self.STATUS_ACTIVE
        )
```

**Step 4: Create and apply migration**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard/backend
python manage.py makemigrations users --name user_subscription_fields
python manage.py migrate
```

**Step 5: Run tests to verify they pass**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/users/tests/test_models.py -v`

Expected: All 5 tests PASS

**Step 6: Commit**

```bash
git add backend/apps/users/models.py backend/apps/users/migrations/ backend/apps/users/tests/
git commit -m "feat(users): add subscription and profile fields to User model"
```

---

## Task 4: Update User Serializer

**Files:**
- Modify: `backend/apps/users/serializers.py`

**Step 1: Write test for serializer**

Add to `backend/apps/users/tests/test_models.py`:

```python
from apps.users.serializers import UserSerializer


@pytest.mark.django_db
class TestUserSerializer:
    """Tests for UserSerializer."""

    def test_serializer_includes_subscription_fields(self):
        """Serializer should include subscription fields."""
        user = User.objects.create_user(email="test@example.com", password="testpass123")
        serializer = UserSerializer(user)
        data = serializer.data

        assert "subscription_tier" in data
        assert "subscription_status" in data
        assert "display_name" in data
        assert "timezone" in data
        assert "language" in data

    def test_serializer_subscription_fields_are_readonly(self):
        """Subscription tier/status should be read-only."""
        user = User.objects.create_user(email="test@example.com", password="testpass123")
        serializer = UserSerializer(
            user,
            data={"subscription_tier": "premium", "subscription_status": "active"},
            partial=True,
        )
        assert serializer.is_valid()
        serializer.save()
        user.refresh_from_db()
        # Should not have changed
        assert user.subscription_tier == "free"
        assert user.subscription_status == "none"

    def test_serializer_allows_profile_updates(self):
        """Should allow updating profile fields."""
        user = User.objects.create_user(email="test@example.com", password="testpass123")
        serializer = UserSerializer(
            user,
            data={"display_name": "Test User", "timezone": "Europe/Paris", "language": "fr"},
            partial=True,
        )
        assert serializer.is_valid()
        serializer.save()
        user.refresh_from_db()
        assert user.display_name == "Test User"
        assert user.timezone == "Europe/Paris"
        assert user.language == "fr"
```

**Step 2: Run test to verify it fails**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/users/tests/test_models.py::TestUserSerializer -v`

Expected: FAIL - fields not in serializer

**Step 3: Update serializer**

Replace content of `backend/apps/users/serializers.py`:

```python
"""User serializers for REST API."""

from django.contrib.auth import get_user_model
from rest_framework import serializers

User = get_user_model()


class UserSerializer(serializers.ModelSerializer):
    """Serializer for user profile."""

    class Meta:
        model = User
        fields = [
            "id",
            "email",
            "first_name",
            "last_name",
            "avatar_url",
            "display_name",
            "timezone",
            "language",
            "subscription_tier",
            "subscription_status",
            "subscription_ends_at",
            "date_joined",
        ]
        read_only_fields = [
            "id",
            "email",
            "subscription_tier",
            "subscription_status",
            "subscription_ends_at",
            "date_joined",
        ]


class UserPublicSerializer(serializers.ModelSerializer):
    """Public serializer for user (limited fields)."""

    class Meta:
        model = User
        fields = ["id", "display_name", "avatar_url"]
        read_only_fields = fields
```

**Step 4: Run tests to verify they pass**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/users/tests/test_models.py -v`

Expected: All 8 tests PASS

**Step 5: Commit**

```bash
git add backend/apps/users/serializers.py backend/apps/users/tests/
git commit -m "feat(users): update serializer with subscription and profile fields"
```

---

## Task 5: Create Super Admin Command

**Files:**
- Create: `backend/apps/core/management/__init__.py`
- Create: `backend/apps/core/management/commands/__init__.py`
- Create: `backend/apps/core/management/commands/setup_superadmin.py`

**Step 1: Write test for command**

Create `backend/apps/core/tests/__init__.py` (empty file)

Create `backend/apps/core/tests/test_commands.py`:

```python
"""Tests for management commands."""

import pytest
from django.core.management import call_command
from django.contrib.auth import get_user_model
from io import StringIO

User = get_user_model()


@pytest.mark.django_db
class TestSetupSuperadminCommand:
    """Tests for setup_superadmin command."""

    def test_creates_superadmin_when_env_set(self, settings):
        """Should create superadmin from SUPER_ADMIN_EMAIL."""
        settings.SUPER_ADMIN_EMAIL = "admin@test.com"
        out = StringIO()

        call_command("setup_superadmin", stdout=out)

        user = User.objects.get(email="admin@test.com")
        assert user.is_superuser is True
        assert user.is_staff is True
        assert user.subscription_tier == "teams"
        assert user.subscription_status == "active"
        assert "Super admin created" in out.getvalue()

    def test_skips_when_env_not_set(self, settings):
        """Should skip when SUPER_ADMIN_EMAIL not set."""
        settings.SUPER_ADMIN_EMAIL = ""
        out = StringIO()

        call_command("setup_superadmin", stdout=out)

        assert User.objects.filter(is_superuser=True).count() == 0
        assert "not set" in out.getvalue()

    def test_skips_when_superadmin_exists(self, settings):
        """Should skip when a superadmin already exists."""
        User.objects.create_superuser(email="existing@test.com", password="test")
        settings.SUPER_ADMIN_EMAIL = "new@test.com"
        out = StringIO()

        call_command("setup_superadmin", stdout=out)

        assert not User.objects.filter(email="new@test.com").exists()
        assert "already exists" in out.getvalue()

    def test_promotes_existing_user(self, settings):
        """Should promote existing user to superadmin."""
        user = User.objects.create_user(email="admin@test.com", password="test")
        assert user.is_superuser is False

        settings.SUPER_ADMIN_EMAIL = "admin@test.com"
        out = StringIO()

        call_command("setup_superadmin", stdout=out)

        user.refresh_from_db()
        assert user.is_superuser is True
        assert user.is_staff is True
        assert "Super admin promoted" in out.getvalue()
```

**Step 2: Run test to verify it fails**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/core/tests/test_commands.py -v`

Expected: FAIL - command doesn't exist

**Step 3: Create management command directories**

Run:
```bash
mkdir -p /Users/didouye/Workspace/voiceboard/backend/apps/core/management/commands
touch /Users/didouye/Workspace/voiceboard/backend/apps/core/management/__init__.py
touch /Users/didouye/Workspace/voiceboard/backend/apps/core/management/commands/__init__.py
mkdir -p /Users/didouye/Workspace/voiceboard/backend/apps/core/tests
touch /Users/didouye/Workspace/voiceboard/backend/apps/core/tests/__init__.py
```

**Step 4: Create the command**

Create `backend/apps/core/management/commands/setup_superadmin.py`:

```python
"""Management command to create super admin from environment variable."""

from django.conf import settings
from django.contrib.auth import get_user_model
from django.core.management.base import BaseCommand

User = get_user_model()


class Command(BaseCommand):
    """Create super admin from SUPER_ADMIN_EMAIL environment variable."""

    help = "Create super admin from SUPER_ADMIN_EMAIL env var"

    def handle(self, *args, **options):
        """Execute the command."""
        email = settings.SUPER_ADMIN_EMAIL

        if not email:
            self.stdout.write("SUPER_ADMIN_EMAIL not set, skipping")
            return

        if User.objects.filter(is_superuser=True).exists():
            self.stdout.write("Super admin already exists, skipping")
            return

        user, created = User.objects.get_or_create(
            email=email,
            defaults={
                "is_staff": True,
                "is_superuser": True,
                "subscription_tier": User.TIER_TEAMS,
                "subscription_status": User.STATUS_ACTIVE,
            },
        )

        if created:
            self.stdout.write(self.style.SUCCESS(f"Super admin created: {email}"))
        else:
            user.is_staff = True
            user.is_superuser = True
            user.subscription_tier = User.TIER_TEAMS
            user.subscription_status = User.STATUS_ACTIVE
            user.save()
            self.stdout.write(self.style.SUCCESS(f"Super admin promoted: {email}"))
```

**Step 5: Run tests to verify they pass**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/core/tests/test_commands.py -v`

Expected: All 4 tests PASS

**Step 6: Update docker-entrypoint.sh**

Add after `python manage.py migrate` in `backend/docker-entrypoint.sh`:

```bash
python manage.py setup_superadmin
```

**Step 7: Commit**

```bash
git add backend/apps/core/management/ backend/apps/core/tests/ backend/docker-entrypoint.sh
git commit -m "feat(core): add setup_superadmin management command"
```

---

## Task 6: Create Billing App Structure

**Files:**
- Create: `backend/apps/billing/__init__.py`
- Create: `backend/apps/billing/admin.py`
- Create: `backend/apps/billing/apps.py`
- Create: `backend/apps/billing/constants.py`
- Create: `backend/apps/billing/urls.py`

**Step 1: Create billing app directory and files**

Run:
```bash
mkdir -p /Users/didouye/Workspace/voiceboard/backend/apps/billing
touch /Users/didouye/Workspace/voiceboard/backend/apps/billing/__init__.py
```

**Step 2: Create apps.py**

Create `backend/apps/billing/apps.py`:

```python
"""Billing app configuration."""

from django.apps import AppConfig


class BillingConfig(AppConfig):
    """Configuration for billing app."""

    default_auto_field = "django.db.models.BigAutoField"
    name = "apps.billing"
```

**Step 3: Create admin.py**

Create `backend/apps/billing/admin.py`:

```python
"""Billing admin configuration."""

# No models to register yet
```

**Step 4: Create constants.py**

Create `backend/apps/billing/constants.py`:

```python
"""Billing constants."""

from django.conf import settings

# Valid plan identifiers
VALID_PLANS = [
    "premium_monthly",
    "premium_yearly",
    "teams_monthly",
    "teams_yearly",
]


def get_price_id(plan: str) -> str:
    """Get Stripe price ID for a plan."""
    return settings.STRIPE_PRICES.get(plan, "")


def get_tier_for_plan(plan: str) -> str:
    """Get subscription tier for a plan."""
    if plan.startswith("premium"):
        return "premium"
    elif plan.startswith("teams"):
        return "teams"
    return "free"
```

**Step 5: Create urls.py**

Create `backend/apps/billing/urls.py`:

```python
"""Billing URL patterns."""

from django.urls import path

from .views import (
    CheckoutView,
    CustomerPortalView,
    SubscriptionView,
    stripe_webhook,
)

urlpatterns = [
    path("checkout/", CheckoutView.as_view(), name="billing-checkout"),
    path("portal/", CustomerPortalView.as_view(), name="billing-portal"),
    path("subscription/", SubscriptionView.as_view(), name="billing-subscription"),
    path("webhook/", stripe_webhook, name="billing-webhook"),
]
```

**Step 6: Add billing app to INSTALLED_APPS**

In `backend/config/settings/base.py`, add `"apps.billing"` to `INSTALLED_APPS`:

```python
INSTALLED_APPS = [
    # ... existing apps ...
    # Local apps
    "apps.core",
    "apps.users",
    "apps.billing",  # Add this
]
```

**Step 7: Add billing URLs to main urls.py**

In `backend/config/urls.py`, add:

```python
urlpatterns = [
    path("admin/", admin.site.urls),
    path("api/auth/", include("apps.users.urls")),
    path("api/billing/", include("apps.billing.urls")),  # Add this
]
```

**Step 8: Commit**

```bash
git add backend/apps/billing/ backend/config/settings/base.py backend/config/urls.py
git commit -m "feat(billing): create billing app structure"
```

---

## Task 7: Implement Billing Services

**Files:**
- Create: `backend/apps/billing/services.py`
- Create: `backend/apps/billing/tests/__init__.py`
- Create: `backend/apps/billing/tests/test_services.py`

**Step 1: Write tests for billing services**

Create `backend/apps/billing/tests/__init__.py` (empty)

Create `backend/apps/billing/tests/test_services.py`:

```python
"""Tests for billing services."""

import pytest
from unittest.mock import patch, MagicMock
from django.contrib.auth import get_user_model

from apps.billing.services import BillingService

User = get_user_model()


@pytest.mark.django_db
class TestBillingService:
    """Tests for BillingService."""

    def test_get_or_create_stripe_customer_creates_new(self):
        """Should create Stripe customer if not exists."""
        user = User.objects.create_user(email="test@example.com", password="test")

        with patch("stripe.Customer.create") as mock_create:
            mock_create.return_value = MagicMock(id="cus_test123")
            customer_id = BillingService.get_or_create_stripe_customer(user)

        assert customer_id == "cus_test123"
        user.refresh_from_db()
        assert user.stripe_customer_id == "cus_test123"
        mock_create.assert_called_once_with(email="test@example.com")

    def test_get_or_create_stripe_customer_returns_existing(self):
        """Should return existing Stripe customer ID."""
        user = User.objects.create_user(
            email="test@example.com",
            password="test",
            stripe_customer_id="cus_existing",
        )

        with patch("stripe.Customer.create") as mock_create:
            customer_id = BillingService.get_or_create_stripe_customer(user)

        assert customer_id == "cus_existing"
        mock_create.assert_not_called()

    def test_create_checkout_session(self, settings):
        """Should create Stripe checkout session."""
        settings.STRIPE_PRICES = {"premium_monthly": "price_test123"}
        user = User.objects.create_user(
            email="test@example.com",
            password="test",
            stripe_customer_id="cus_test",
        )

        with patch("stripe.checkout.Session.create") as mock_create:
            mock_create.return_value = MagicMock(url="https://checkout.stripe.com/test")
            url = BillingService.create_checkout_session(
                user, "premium_monthly", "https://app.test/success", "https://app.test/cancel"
            )

        assert url == "https://checkout.stripe.com/test"
        mock_create.assert_called_once()

    def test_create_checkout_session_invalid_plan(self):
        """Should raise error for invalid plan."""
        user = User.objects.create_user(email="test@example.com", password="test")

        with pytest.raises(ValueError, match="Invalid plan"):
            BillingService.create_checkout_session(
                user, "invalid_plan", "https://test/success", "https://test/cancel"
            )

    def test_create_customer_portal_session(self):
        """Should create Stripe customer portal session."""
        user = User.objects.create_user(
            email="test@example.com",
            password="test",
            stripe_customer_id="cus_test",
        )

        with patch("stripe.billing_portal.Session.create") as mock_create:
            mock_create.return_value = MagicMock(url="https://billing.stripe.com/test")
            url = BillingService.create_customer_portal_session(
                user, "https://app.test/settings"
            )

        assert url == "https://billing.stripe.com/test"

    def test_update_subscription_from_webhook(self):
        """Should update user subscription from webhook data."""
        user = User.objects.create_user(
            email="test@example.com",
            password="test",
            stripe_customer_id="cus_test",
        )

        BillingService.update_subscription(
            customer_id="cus_test",
            tier="premium",
            status="active",
            current_period_end=1735689600,  # 2025-01-01
        )

        user.refresh_from_db()
        assert user.subscription_tier == "premium"
        assert user.subscription_status == "active"
        assert user.subscription_ends_at is not None
```

**Step 2: Run tests to verify they fail**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/billing/tests/test_services.py -v`

Expected: FAIL - services module doesn't exist

**Step 3: Implement BillingService**

Create `backend/apps/billing/services.py`:

```python
"""Billing services for Stripe integration."""

from datetime import datetime
from typing import Optional

import stripe
from django.conf import settings
from django.contrib.auth import get_user_model
from django.utils import timezone

from .constants import VALID_PLANS, get_price_id, get_tier_for_plan

User = get_user_model()

# Configure Stripe
stripe.api_key = settings.STRIPE_SECRET_KEY


class BillingService:
    """Service for Stripe billing operations."""

    @staticmethod
    def get_or_create_stripe_customer(user) -> str:
        """Get or create a Stripe customer for the user."""
        if user.stripe_customer_id:
            return user.stripe_customer_id

        customer = stripe.Customer.create(email=user.email)
        user.stripe_customer_id = customer.id
        user.save(update_fields=["stripe_customer_id"])
        return customer.id

    @staticmethod
    def create_checkout_session(
        user,
        plan: str,
        success_url: str,
        cancel_url: str,
    ) -> str:
        """Create a Stripe Checkout session and return the URL."""
        if plan not in VALID_PLANS:
            raise ValueError(f"Invalid plan: {plan}")

        price_id = get_price_id(plan)
        if not price_id:
            raise ValueError(f"Price ID not configured for plan: {plan}")

        customer_id = BillingService.get_or_create_stripe_customer(user)

        session = stripe.checkout.Session.create(
            customer=customer_id,
            mode="subscription",
            line_items=[{"price": price_id, "quantity": 1}],
            success_url=success_url,
            cancel_url=cancel_url,
            metadata={
                "user_id": str(user.id),
                "plan": plan,
            },
        )

        return session.url

    @staticmethod
    def create_customer_portal_session(user, return_url: str) -> str:
        """Create a Stripe Customer Portal session and return the URL."""
        if not user.stripe_customer_id:
            raise ValueError("User has no Stripe customer ID")

        session = stripe.billing_portal.Session.create(
            customer=user.stripe_customer_id,
            return_url=return_url,
        )

        return session.url

    @staticmethod
    def update_subscription(
        customer_id: str,
        tier: str,
        status: str,
        current_period_end: Optional[int] = None,
    ) -> None:
        """Update user subscription from Stripe webhook data."""
        try:
            user = User.objects.get(stripe_customer_id=customer_id)
        except User.DoesNotExist:
            return

        user.subscription_tier = tier
        user.subscription_status = status

        if current_period_end:
            user.subscription_ends_at = timezone.make_aware(
                datetime.fromtimestamp(current_period_end)
            )

        user.save(
            update_fields=[
                "subscription_tier",
                "subscription_status",
                "subscription_ends_at",
            ]
        )

    @staticmethod
    def cancel_subscription(customer_id: str) -> None:
        """Mark subscription as cancelled."""
        BillingService.update_subscription(
            customer_id=customer_id,
            tier="free",
            status="cancelled",
        )
```

**Step 4: Run tests to verify they pass**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/billing/tests/test_services.py -v`

Expected: All 6 tests PASS

**Step 5: Commit**

```bash
git add backend/apps/billing/services.py backend/apps/billing/tests/
git commit -m "feat(billing): implement BillingService with Stripe integration"
```

---

## Task 8: Implement Billing Views

**Files:**
- Create: `backend/apps/billing/serializers.py`
- Create: `backend/apps/billing/views.py`
- Create: `backend/apps/billing/tests/test_views.py`

**Step 1: Write tests for views**

Create `backend/apps/billing/tests/test_views.py`:

```python
"""Tests for billing views."""

import json
import pytest
from unittest.mock import patch, MagicMock
from django.urls import reverse
from rest_framework.test import APIClient
from django.contrib.auth import get_user_model

User = get_user_model()


@pytest.fixture
def api_client():
    return APIClient()


@pytest.fixture
def authenticated_user(api_client):
    user = User.objects.create_user(email="test@example.com", password="test")
    api_client.force_authenticate(user=user)
    return user


@pytest.mark.django_db
class TestCheckoutView:
    """Tests for checkout endpoint."""

    def test_checkout_requires_auth(self, api_client):
        """Should require authentication."""
        response = api_client.post(reverse("billing-checkout"), {"plan": "premium_monthly"})
        assert response.status_code == 401

    def test_checkout_creates_session(self, api_client, authenticated_user, settings):
        """Should create checkout session."""
        settings.STRIPE_PRICES = {"premium_monthly": "price_test"}

        with patch("apps.billing.services.BillingService.create_checkout_session") as mock:
            mock.return_value = "https://checkout.stripe.com/test"
            response = api_client.post(
                reverse("billing-checkout"),
                {"plan": "premium_monthly"},
                format="json",
            )

        assert response.status_code == 200
        assert response.data["checkout_url"] == "https://checkout.stripe.com/test"

    def test_checkout_rejects_invalid_plan(self, api_client, authenticated_user):
        """Should reject invalid plan."""
        response = api_client.post(
            reverse("billing-checkout"),
            {"plan": "invalid"},
            format="json",
        )
        assert response.status_code == 400


@pytest.mark.django_db
class TestSubscriptionView:
    """Tests for subscription status endpoint."""

    def test_returns_subscription_status(self, api_client, authenticated_user):
        """Should return current subscription status."""
        authenticated_user.subscription_tier = "premium"
        authenticated_user.subscription_status = "active"
        authenticated_user.save()

        response = api_client.get(reverse("billing-subscription"))

        assert response.status_code == 200
        assert response.data["tier"] == "premium"
        assert response.data["status"] == "active"


@pytest.mark.django_db
class TestCustomerPortalView:
    """Tests for customer portal endpoint."""

    def test_requires_stripe_customer(self, api_client, authenticated_user):
        """Should require Stripe customer ID."""
        response = api_client.post(reverse("billing-portal"))
        assert response.status_code == 400

    def test_creates_portal_session(self, api_client, authenticated_user):
        """Should create portal session."""
        authenticated_user.stripe_customer_id = "cus_test"
        authenticated_user.save()

        with patch("apps.billing.services.BillingService.create_customer_portal_session") as mock:
            mock.return_value = "https://billing.stripe.com/test"
            response = api_client.post(reverse("billing-portal"))

        assert response.status_code == 200
        assert response.data["portal_url"] == "https://billing.stripe.com/test"
```

**Step 2: Run tests to verify they fail**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/billing/tests/test_views.py -v`

Expected: FAIL - views don't exist

**Step 3: Create serializers**

Create `backend/apps/billing/serializers.py`:

```python
"""Billing serializers."""

from rest_framework import serializers

from .constants import VALID_PLANS


class CheckoutSerializer(serializers.Serializer):
    """Serializer for checkout request."""

    plan = serializers.ChoiceField(choices=[(p, p) for p in VALID_PLANS])


class CheckoutResponseSerializer(serializers.Serializer):
    """Serializer for checkout response."""

    checkout_url = serializers.URLField()


class PortalResponseSerializer(serializers.Serializer):
    """Serializer for portal response."""

    portal_url = serializers.URLField()


class SubscriptionSerializer(serializers.Serializer):
    """Serializer for subscription status."""

    tier = serializers.CharField()
    status = serializers.CharField()
    ends_at = serializers.DateTimeField(allow_null=True)
```

**Step 4: Create views**

Create `backend/apps/billing/views.py`:

```python
"""Billing views for REST API."""

import json
import logging

import stripe
from django.conf import settings
from django.http import HttpResponse
from django.views.decorators.csrf import csrf_exempt
from django.views.decorators.http import require_POST
from rest_framework import status
from rest_framework.permissions import IsAuthenticated
from rest_framework.response import Response
from rest_framework.views import APIView

from .constants import get_tier_for_plan
from .serializers import (
    CheckoutSerializer,
    CheckoutResponseSerializer,
    PortalResponseSerializer,
    SubscriptionSerializer,
)
from .services import BillingService

logger = logging.getLogger(__name__)


class CheckoutView(APIView):
    """Create Stripe Checkout session."""

    permission_classes = [IsAuthenticated]

    def post(self, request):
        serializer = CheckoutSerializer(data=request.data)
        if not serializer.is_valid():
            return Response(serializer.errors, status=status.HTTP_400_BAD_REQUEST)

        plan = serializer.validated_data["plan"]

        # Build URLs - in production, these come from frontend
        success_url = request.build_absolute_uri("/billing/success")
        cancel_url = request.build_absolute_uri("/billing/cancel")

        try:
            checkout_url = BillingService.create_checkout_session(
                user=request.user,
                plan=plan,
                success_url=success_url,
                cancel_url=cancel_url,
            )
        except ValueError as e:
            return Response({"error": str(e)}, status=status.HTTP_400_BAD_REQUEST)

        return Response(CheckoutResponseSerializer({"checkout_url": checkout_url}).data)


class CustomerPortalView(APIView):
    """Create Stripe Customer Portal session."""

    permission_classes = [IsAuthenticated]

    def post(self, request):
        if not request.user.stripe_customer_id:
            return Response(
                {"error": "No billing account found"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        return_url = request.build_absolute_uri("/settings")

        try:
            portal_url = BillingService.create_customer_portal_session(
                user=request.user,
                return_url=return_url,
            )
        except ValueError as e:
            return Response({"error": str(e)}, status=status.HTTP_400_BAD_REQUEST)

        return Response(PortalResponseSerializer({"portal_url": portal_url}).data)


class SubscriptionView(APIView):
    """Get current subscription status."""

    permission_classes = [IsAuthenticated]

    def get(self, request):
        user = request.user
        data = {
            "tier": user.subscription_tier,
            "status": user.subscription_status,
            "ends_at": user.subscription_ends_at,
        }
        return Response(SubscriptionSerializer(data).data)


@csrf_exempt
@require_POST
def stripe_webhook(request):
    """Handle Stripe webhook events."""
    payload = request.body
    sig_header = request.headers.get("Stripe-Signature")

    try:
        event = stripe.Webhook.construct_event(
            payload, sig_header, settings.STRIPE_WEBHOOK_SECRET
        )
    except ValueError:
        logger.error("Invalid webhook payload")
        return HttpResponse(status=400)
    except stripe.error.SignatureVerificationError:
        logger.error("Invalid webhook signature")
        return HttpResponse(status=400)

    event_type = event["type"]
    data = event["data"]["object"]

    logger.info(f"Received Stripe webhook: {event_type}")

    if event_type == "checkout.session.completed":
        _handle_checkout_completed(data)
    elif event_type == "customer.subscription.updated":
        _handle_subscription_updated(data)
    elif event_type == "customer.subscription.deleted":
        _handle_subscription_deleted(data)
    elif event_type == "invoice.payment_failed":
        _handle_payment_failed(data)
    elif event_type == "invoice.paid":
        _handle_invoice_paid(data)

    return HttpResponse(status=200)


def _handle_checkout_completed(session):
    """Handle successful checkout."""
    customer_id = session.get("customer")
    metadata = session.get("metadata", {})
    plan = metadata.get("plan", "")
    tier = get_tier_for_plan(plan)

    BillingService.update_subscription(
        customer_id=customer_id,
        tier=tier,
        status="active",
    )
    logger.info(f"Checkout completed for customer {customer_id}, tier: {tier}")


def _handle_subscription_updated(subscription):
    """Handle subscription update."""
    customer_id = subscription.get("customer")
    status_value = subscription.get("status")
    current_period_end = subscription.get("current_period_end")

    # Map Stripe status to our status
    status_map = {
        "active": "active",
        "past_due": "past_due",
        "canceled": "cancelled",
        "unpaid": "past_due",
    }
    our_status = status_map.get(status_value, "none")

    # Get tier from price
    items = subscription.get("items", {}).get("data", [])
    tier = "free"
    if items:
        price_id = items[0].get("price", {}).get("id", "")
        # Reverse lookup tier from price ID
        for plan, pid in settings.STRIPE_PRICES.items():
            if pid == price_id:
                tier = get_tier_for_plan(plan)
                break

    BillingService.update_subscription(
        customer_id=customer_id,
        tier=tier,
        status=our_status,
        current_period_end=current_period_end,
    )
    logger.info(f"Subscription updated for customer {customer_id}: {tier}/{our_status}")


def _handle_subscription_deleted(subscription):
    """Handle subscription cancellation."""
    customer_id = subscription.get("customer")
    BillingService.cancel_subscription(customer_id)
    logger.info(f"Subscription deleted for customer {customer_id}")


def _handle_payment_failed(invoice):
    """Handle failed payment."""
    customer_id = invoice.get("customer")
    BillingService.update_subscription(
        customer_id=customer_id,
        tier="premium",  # Keep current tier
        status="past_due",
    )
    logger.warning(f"Payment failed for customer {customer_id}")


def _handle_invoice_paid(invoice):
    """Handle successful payment."""
    customer_id = invoice.get("customer")
    # Just update status to active, tier should already be set
    from django.contrib.auth import get_user_model
    User = get_user_model()
    try:
        user = User.objects.get(stripe_customer_id=customer_id)
        user.subscription_status = "active"
        user.save(update_fields=["subscription_status"])
        logger.info(f"Invoice paid for customer {customer_id}")
    except User.DoesNotExist:
        pass
```

**Step 5: Run tests to verify they pass**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/billing/tests/test_views.py -v`

Expected: All 6 tests PASS

**Step 6: Commit**

```bash
git add backend/apps/billing/serializers.py backend/apps/billing/views.py backend/apps/billing/tests/
git commit -m "feat(billing): implement checkout, portal, and webhook views"
```

---

## Task 9: Create Teams App Structure

**Files:**
- Create: `backend/apps/teams/__init__.py`
- Create: `backend/apps/teams/apps.py`
- Create: `backend/apps/teams/admin.py`
- Create: `backend/apps/teams/models.py`
- Create: `backend/apps/teams/urls.py`

**Step 1: Create teams app directory**

Run:
```bash
mkdir -p /Users/didouye/Workspace/voiceboard/backend/apps/teams
touch /Users/didouye/Workspace/voiceboard/backend/apps/teams/__init__.py
```

**Step 2: Create apps.py**

Create `backend/apps/teams/apps.py`:

```python
"""Teams app configuration."""

from django.apps import AppConfig


class TeamsConfig(AppConfig):
    """Configuration for teams app."""

    default_auto_field = "django.db.models.BigAutoField"
    name = "apps.teams"
```

**Step 3: Write tests for Team model**

Create `backend/apps/teams/tests/__init__.py` (empty)

Create `backend/apps/teams/tests/test_models.py`:

```python
"""Tests for Team models."""

import pytest
from django.contrib.auth import get_user_model
from django.db import IntegrityError

from apps.teams.models import Team, TeamMembership, TeamInvitation

User = get_user_model()


@pytest.mark.django_db
class TestTeamModel:
    """Tests for Team model."""

    def test_create_team(self):
        """Should create a team with owner."""
        owner = User.objects.create_user(email="owner@test.com", password="test")
        team = Team.objects.create(name="Test Team", owner=owner)

        assert team.name == "Test Team"
        assert team.owner == owner
        assert team.max_members == 8
        assert team.member_count == 1  # Owner counts

    def test_max_members_with_extra_seats(self):
        """Should include extra seats in max members."""
        owner = User.objects.create_user(email="owner@test.com", password="test")
        team = Team.objects.create(name="Test Team", owner=owner, extra_seats=5)

        assert team.max_members == 13  # 8 + 5

    def test_member_count_includes_members(self):
        """Should count owner + members."""
        owner = User.objects.create_user(email="owner@test.com", password="test")
        member = User.objects.create_user(email="member@test.com", password="test")
        team = Team.objects.create(name="Test Team", owner=owner)
        TeamMembership.objects.create(team=team, user=member)

        assert team.member_count == 2  # Owner + 1 member


@pytest.mark.django_db
class TestTeamMembership:
    """Tests for TeamMembership model."""

    def test_create_membership(self):
        """Should create membership."""
        owner = User.objects.create_user(email="owner@test.com", password="test")
        member = User.objects.create_user(email="member@test.com", password="test")
        team = Team.objects.create(name="Test Team", owner=owner)

        membership = TeamMembership.objects.create(team=team, user=member)

        assert membership.role == "member"
        assert member in team.members.all()

    def test_unique_membership(self):
        """Should not allow duplicate memberships."""
        owner = User.objects.create_user(email="owner@test.com", password="test")
        member = User.objects.create_user(email="member@test.com", password="test")
        team = Team.objects.create(name="Test Team", owner=owner)
        TeamMembership.objects.create(team=team, user=member)

        with pytest.raises(IntegrityError):
            TeamMembership.objects.create(team=team, user=member)


@pytest.mark.django_db
class TestTeamInvitation:
    """Tests for TeamInvitation model."""

    def test_create_invitation(self):
        """Should create invitation."""
        owner = User.objects.create_user(email="owner@test.com", password="test")
        team = Team.objects.create(name="Test Team", owner=owner)

        invitation = TeamInvitation.objects.create(
            team=team,
            email="invited@test.com",
            invited_by=owner,
            token="test-token-123",
        )

        assert invitation.email == "invited@test.com"
        assert invitation.team == team
```

**Step 4: Run tests to verify they fail**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/teams/tests/test_models.py -v`

Expected: FAIL - models don't exist

**Step 5: Create models**

Create `backend/apps/teams/models.py`:

```python
"""Team models."""

from django.conf import settings
from django.db import models


class Team(models.Model):
    """Team model for collaborative soundboards."""

    name = models.CharField(max_length=100)
    owner = models.ForeignKey(
        settings.AUTH_USER_MODEL,
        on_delete=models.CASCADE,
        related_name="owned_teams",
    )
    members = models.ManyToManyField(
        settings.AUTH_USER_MODEL,
        through="TeamMembership",
        related_name="teams",
    )
    created_at = models.DateTimeField(auto_now_add=True)

    # Stripe
    stripe_subscription_id = models.CharField(max_length=255, blank=True, default="")
    extra_seats = models.PositiveIntegerField(default=0)

    class Meta:
        db_table = "teams"

    def __str__(self):
        return self.name

    @property
    def max_members(self):
        """Maximum members including owner (8 base + extra seats)."""
        return 8 + self.extra_seats

    @property
    def member_count(self):
        """Current member count including owner."""
        return self.members.count() + 1


class TeamMembership(models.Model):
    """Through model for team membership."""

    ROLE_MEMBER = "member"
    ROLE_CHOICES = [
        (ROLE_MEMBER, "Member"),
    ]

    team = models.ForeignKey(Team, on_delete=models.CASCADE)
    user = models.ForeignKey(settings.AUTH_USER_MODEL, on_delete=models.CASCADE)
    role = models.CharField(max_length=20, choices=ROLE_CHOICES, default=ROLE_MEMBER)
    joined_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "team_memberships"
        unique_together = ["team", "user"]


class TeamInvitation(models.Model):
    """Pending team invitation."""

    team = models.ForeignKey(Team, on_delete=models.CASCADE, related_name="invitations")
    email = models.EmailField()
    invited_by = models.ForeignKey(
        settings.AUTH_USER_MODEL,
        on_delete=models.CASCADE,
        related_name="sent_invitations",
    )
    token = models.CharField(max_length=64, unique=True)
    created_at = models.DateTimeField(auto_now_add=True)

    class Meta:
        db_table = "team_invitations"
        unique_together = ["team", "email"]
```

**Step 6: Create admin**

Create `backend/apps/teams/admin.py`:

```python
"""Teams admin configuration."""

from django.contrib import admin

from .models import Team, TeamMembership, TeamInvitation


class TeamMembershipInline(admin.TabularInline):
    model = TeamMembership
    extra = 0


@admin.register(Team)
class TeamAdmin(admin.ModelAdmin):
    list_display = ["name", "owner", "member_count", "created_at"]
    search_fields = ["name", "owner__email"]
    inlines = [TeamMembershipInline]


@admin.register(TeamInvitation)
class TeamInvitationAdmin(admin.ModelAdmin):
    list_display = ["email", "team", "invited_by", "created_at"]
    search_fields = ["email", "team__name"]
```

**Step 7: Add teams app to INSTALLED_APPS**

In `backend/config/settings/base.py`, add `"apps.teams"`:

```python
INSTALLED_APPS = [
    # ... existing apps ...
    # Local apps
    "apps.core",
    "apps.users",
    "apps.billing",
    "apps.teams",  # Add this
]
```

**Step 8: Create and apply migration**

Run:
```bash
cd /Users/didouye/Workspace/voiceboard/backend
python manage.py makemigrations teams
python manage.py migrate
```

**Step 9: Run tests to verify they pass**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/teams/tests/test_models.py -v`

Expected: All 6 tests PASS

**Step 10: Commit**

```bash
git add backend/apps/teams/ backend/config/settings/base.py
git commit -m "feat(teams): create Team, TeamMembership, TeamInvitation models"
```

---

## Task 10: Implement Teams Views

**Files:**
- Create: `backend/apps/teams/serializers.py`
- Create: `backend/apps/teams/permissions.py`
- Create: `backend/apps/teams/views.py`
- Create: `backend/apps/teams/urls.py`
- Create: `backend/apps/teams/tests/test_views.py`

**Step 1: Write tests for team views**

Create `backend/apps/teams/tests/test_views.py`:

```python
"""Tests for teams views."""

import pytest
from django.urls import reverse
from rest_framework.test import APIClient
from django.contrib.auth import get_user_model

from apps.teams.models import Team, TeamMembership

User = get_user_model()


@pytest.fixture
def api_client():
    return APIClient()


@pytest.fixture
def premium_user(api_client):
    user = User.objects.create_user(
        email="premium@test.com",
        password="test",
        subscription_tier="teams",
        subscription_status="active",
    )
    api_client.force_authenticate(user=user)
    return user


@pytest.fixture
def free_user(api_client):
    user = User.objects.create_user(email="free@test.com", password="test")
    api_client.force_authenticate(user=user)
    return user


@pytest.mark.django_db
class TestTeamListCreate:
    """Tests for team list/create endpoint."""

    def test_list_teams(self, api_client, premium_user):
        """Should list user's teams."""
        team = Team.objects.create(name="My Team", owner=premium_user)
        response = api_client.get(reverse("team-list"))

        assert response.status_code == 200
        assert len(response.data) == 1
        assert response.data[0]["name"] == "My Team"

    def test_create_team_requires_subscription(self, api_client, free_user):
        """Should require Teams subscription to create team."""
        response = api_client.post(
            reverse("team-list"),
            {"name": "New Team"},
            format="json",
        )
        assert response.status_code == 403

    def test_create_team_with_subscription(self, api_client, premium_user):
        """Should allow creating team with subscription."""
        response = api_client.post(
            reverse("team-list"),
            {"name": "New Team"},
            format="json",
        )
        assert response.status_code == 201
        assert Team.objects.filter(name="New Team", owner=premium_user).exists()


@pytest.mark.django_db
class TestTeamDetail:
    """Tests for team detail endpoint."""

    def test_get_team_detail(self, api_client, premium_user):
        """Should get team details."""
        team = Team.objects.create(name="My Team", owner=premium_user)
        response = api_client.get(reverse("team-detail", args=[team.id]))

        assert response.status_code == 200
        assert response.data["name"] == "My Team"

    def test_update_team_owner_only(self, api_client, premium_user):
        """Only owner can update team."""
        other_user = User.objects.create_user(email="other@test.com", password="test")
        team = Team.objects.create(name="Other Team", owner=other_user)
        TeamMembership.objects.create(team=team, user=premium_user)

        response = api_client.put(
            reverse("team-detail", args=[team.id]),
            {"name": "Renamed"},
            format="json",
        )
        assert response.status_code == 403

    def test_delete_team_owner_only(self, api_client, premium_user):
        """Only owner can delete team."""
        team = Team.objects.create(name="My Team", owner=premium_user)
        response = api_client.delete(reverse("team-detail", args=[team.id]))

        assert response.status_code == 204
        assert not Team.objects.filter(id=team.id).exists()


@pytest.mark.django_db
class TestTeamInvite:
    """Tests for team invite endpoint."""

    def test_invite_member(self, api_client, premium_user):
        """Should invite member by email."""
        team = Team.objects.create(name="My Team", owner=premium_user)
        response = api_client.post(
            reverse("team-invite", args=[team.id]),
            {"email": "newmember@test.com"},
            format="json",
        )

        assert response.status_code == 201
        assert team.invitations.filter(email="newmember@test.com").exists()

    def test_invite_existing_user_adds_directly(self, api_client, premium_user):
        """Should add existing user directly."""
        existing = User.objects.create_user(email="existing@test.com", password="test")
        team = Team.objects.create(name="My Team", owner=premium_user)

        response = api_client.post(
            reverse("team-invite", args=[team.id]),
            {"email": "existing@test.com"},
            format="json",
        )

        assert response.status_code == 201
        assert existing in team.members.all()


@pytest.mark.django_db
class TestTeamLeave:
    """Tests for team leave endpoint."""

    def test_member_can_leave(self, api_client, premium_user):
        """Member should be able to leave team."""
        owner = User.objects.create_user(
            email="owner@test.com",
            password="test",
            subscription_tier="teams",
            subscription_status="active",
        )
        team = Team.objects.create(name="Owner Team", owner=owner)
        TeamMembership.objects.create(team=team, user=premium_user)

        response = api_client.post(reverse("team-leave", args=[team.id]))

        assert response.status_code == 204
        assert premium_user not in team.members.all()

    def test_owner_cannot_leave(self, api_client, premium_user):
        """Owner cannot leave their own team."""
        team = Team.objects.create(name="My Team", owner=premium_user)

        response = api_client.post(reverse("team-leave", args=[team.id]))

        assert response.status_code == 400
```

**Step 2: Run tests to verify they fail**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/teams/tests/test_views.py -v`

Expected: FAIL - views don't exist

**Step 3: Create permissions**

Create `backend/apps/teams/permissions.py`:

```python
"""Team permissions."""

from rest_framework import permissions


class HasTeamsSubscription(permissions.BasePermission):
    """Requires active Teams subscription."""

    message = "Teams subscription required."

    def has_permission(self, request, view):
        user = request.user
        return (
            user.subscription_tier == "teams"
            and user.subscription_status == "active"
        )


class IsTeamOwner(permissions.BasePermission):
    """Requires user to be team owner."""

    message = "Only team owner can perform this action."

    def has_object_permission(self, request, view, obj):
        return obj.owner == request.user


class IsTeamMember(permissions.BasePermission):
    """Requires user to be team member or owner."""

    def has_object_permission(self, request, view, obj):
        return obj.owner == request.user or request.user in obj.members.all()
```

**Step 4: Create serializers**

Create `backend/apps/teams/serializers.py`:

```python
"""Team serializers."""

from rest_framework import serializers

from apps.users.serializers import UserPublicSerializer

from .models import Team, TeamMembership, TeamInvitation


class TeamMembershipSerializer(serializers.ModelSerializer):
    """Serializer for team membership."""

    user = UserPublicSerializer(read_only=True)

    class Meta:
        model = TeamMembership
        fields = ["user", "role", "joined_at"]


class TeamSerializer(serializers.ModelSerializer):
    """Serializer for Team."""

    owner = UserPublicSerializer(read_only=True)
    members = TeamMembershipSerializer(source="teammembership_set", many=True, read_only=True)
    member_count = serializers.IntegerField(read_only=True)
    max_members = serializers.IntegerField(read_only=True)

    class Meta:
        model = Team
        fields = [
            "id",
            "name",
            "owner",
            "members",
            "member_count",
            "max_members",
            "created_at",
        ]
        read_only_fields = ["id", "owner", "created_at"]


class TeamCreateSerializer(serializers.ModelSerializer):
    """Serializer for creating a team."""

    class Meta:
        model = Team
        fields = ["name"]


class InviteSerializer(serializers.Serializer):
    """Serializer for team invitation."""

    email = serializers.EmailField()
```

**Step 5: Create views**

Create `backend/apps/teams/views.py`:

```python
"""Team views."""

import secrets

from django.contrib.auth import get_user_model
from rest_framework import generics, status
from rest_framework.permissions import IsAuthenticated
from rest_framework.response import Response
from rest_framework.views import APIView

from .models import Team, TeamMembership, TeamInvitation
from .permissions import HasTeamsSubscription, IsTeamOwner, IsTeamMember
from .serializers import (
    TeamSerializer,
    TeamCreateSerializer,
    InviteSerializer,
)

User = get_user_model()


class TeamListCreateView(generics.ListCreateAPIView):
    """List user's teams or create a new team."""

    permission_classes = [IsAuthenticated]

    def get_serializer_class(self):
        if self.request.method == "POST":
            return TeamCreateSerializer
        return TeamSerializer

    def get_queryset(self):
        user = self.request.user
        # Teams where user is owner or member
        owned = Team.objects.filter(owner=user)
        member_of = Team.objects.filter(members=user)
        return (owned | member_of).distinct()

    def get_permissions(self):
        if self.request.method == "POST":
            return [IsAuthenticated(), HasTeamsSubscription()]
        return [IsAuthenticated()]

    def perform_create(self, serializer):
        serializer.save(owner=self.request.user)


class TeamDetailView(generics.RetrieveUpdateDestroyAPIView):
    """Get, update, or delete a team."""

    serializer_class = TeamSerializer
    permission_classes = [IsAuthenticated, IsTeamMember]

    def get_queryset(self):
        user = self.request.user
        owned = Team.objects.filter(owner=user)
        member_of = Team.objects.filter(members=user)
        return (owned | member_of).distinct()

    def get_permissions(self):
        if self.request.method in ["PUT", "PATCH", "DELETE"]:
            return [IsAuthenticated(), IsTeamOwner()]
        return [IsAuthenticated(), IsTeamMember()]


class TeamInviteView(APIView):
    """Invite a member to a team."""

    permission_classes = [IsAuthenticated, IsTeamOwner]

    def post(self, request, pk):
        try:
            team = Team.objects.get(pk=pk)
        except Team.DoesNotExist:
            return Response(
                {"error": "Team not found"},
                status=status.HTTP_404_NOT_FOUND,
            )

        self.check_object_permissions(request, team)

        serializer = InviteSerializer(data=request.data)
        if not serializer.is_valid():
            return Response(serializer.errors, status=status.HTTP_400_BAD_REQUEST)

        email = serializer.validated_data["email"]

        # Check team capacity
        if team.member_count >= team.max_members:
            return Response(
                {"error": "Team is at maximum capacity"},
                status=status.HTTP_400_BAD_REQUEST,
            )

        # Check if user exists
        try:
            user = User.objects.get(email=email)
            # Check if already a member
            if user == team.owner or user in team.members.all():
                return Response(
                    {"error": "User is already a team member"},
                    status=status.HTTP_400_BAD_REQUEST,
                )
            # Add directly
            TeamMembership.objects.create(team=team, user=user)
            return Response(
                {"message": f"{email} added to team"},
                status=status.HTTP_201_CREATED,
            )
        except User.DoesNotExist:
            # Create invitation
            if team.invitations.filter(email=email).exists():
                return Response(
                    {"error": "Invitation already sent"},
                    status=status.HTTP_400_BAD_REQUEST,
                )

            TeamInvitation.objects.create(
                team=team,
                email=email,
                invited_by=request.user,
                token=secrets.token_urlsafe(32),
            )
            # TODO: Send invitation email
            return Response(
                {"message": f"Invitation sent to {email}"},
                status=status.HTTP_201_CREATED,
            )


class TeamRemoveMemberView(APIView):
    """Remove a member from a team."""

    permission_classes = [IsAuthenticated, IsTeamOwner]

    def delete(self, request, pk, user_id):
        try:
            team = Team.objects.get(pk=pk)
        except Team.DoesNotExist:
            return Response(
                {"error": "Team not found"},
                status=status.HTTP_404_NOT_FOUND,
            )

        self.check_object_permissions(request, team)

        try:
            membership = TeamMembership.objects.get(team=team, user_id=user_id)
            membership.delete()
            return Response(status=status.HTTP_204_NO_CONTENT)
        except TeamMembership.DoesNotExist:
            return Response(
                {"error": "User is not a team member"},
                status=status.HTTP_404_NOT_FOUND,
            )


class TeamLeaveView(APIView):
    """Leave a team."""

    permission_classes = [IsAuthenticated]

    def post(self, request, pk):
        try:
            team = Team.objects.get(pk=pk)
        except Team.DoesNotExist:
            return Response(
                {"error": "Team not found"},
                status=status.HTTP_404_NOT_FOUND,
            )

        if team.owner == request.user:
            return Response(
                {"error": "Owner cannot leave the team. Delete it instead."},
                status=status.HTTP_400_BAD_REQUEST,
            )

        try:
            membership = TeamMembership.objects.get(team=team, user=request.user)
            membership.delete()
            return Response(status=status.HTTP_204_NO_CONTENT)
        except TeamMembership.DoesNotExist:
            return Response(
                {"error": "You are not a member of this team"},
                status=status.HTTP_400_BAD_REQUEST,
            )
```

**Step 6: Create URLs**

Create `backend/apps/teams/urls.py`:

```python
"""Team URL patterns."""

from django.urls import path

from .views import (
    TeamListCreateView,
    TeamDetailView,
    TeamInviteView,
    TeamRemoveMemberView,
    TeamLeaveView,
)

urlpatterns = [
    path("", TeamListCreateView.as_view(), name="team-list"),
    path("<int:pk>/", TeamDetailView.as_view(), name="team-detail"),
    path("<int:pk>/invite/", TeamInviteView.as_view(), name="team-invite"),
    path("<int:pk>/members/<int:user_id>/", TeamRemoveMemberView.as_view(), name="team-remove-member"),
    path("<int:pk>/leave/", TeamLeaveView.as_view(), name="team-leave"),
]
```

**Step 7: Add teams URLs to main urls.py**

In `backend/config/urls.py`:

```python
urlpatterns = [
    path("admin/", admin.site.urls),
    path("api/auth/", include("apps.users.urls")),
    path("api/billing/", include("apps.billing.urls")),
    path("api/teams/", include("apps.teams.urls")),  # Add this
]
```

**Step 8: Run tests to verify they pass**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/teams/tests/test_views.py -v`

Expected: All 9 tests PASS

**Step 9: Commit**

```bash
git add backend/apps/teams/ backend/config/urls.py
git commit -m "feat(teams): implement team CRUD, invite, and leave endpoints"
```

---

## Task 11: Create Feature Permissions Helper

**Files:**
- Create: `backend/apps/core/permissions.py`
- Create: `backend/apps/core/tests/test_permissions.py`

**Step 1: Write tests**

Create `backend/apps/core/tests/test_permissions.py`:

```python
"""Tests for core permissions."""

import pytest
from django.contrib.auth import get_user_model

from apps.core.permissions import user_has_feature, FEATURE_TIERS

User = get_user_model()


@pytest.mark.django_db
class TestUserHasFeature:
    """Tests for user_has_feature helper."""

    def test_free_user_has_no_premium_features(self):
        """Free user should not have premium features."""
        user = User.objects.create_user(email="free@test.com", password="test")

        assert user_has_feature(user, "cloud_sync") is False
        assert user_has_feature(user, "sound_search") is False
        assert user_has_feature(user, "ai_generation") is False

    def test_premium_user_has_premium_features(self):
        """Premium user should have premium features."""
        user = User.objects.create_user(
            email="premium@test.com",
            password="test",
            subscription_tier="premium",
            subscription_status="active",
        )

        assert user_has_feature(user, "cloud_sync") is True
        assert user_has_feature(user, "sound_search") is True
        assert user_has_feature(user, "remote_control") is True

    def test_premium_user_no_team_features(self):
        """Premium user should not have team features."""
        user = User.objects.create_user(
            email="premium@test.com",
            password="test",
            subscription_tier="premium",
            subscription_status="active",
        )

        assert user_has_feature(user, "shared_soundboards") is False
        assert user_has_feature(user, "team_management") is False

    def test_teams_user_has_all_features(self):
        """Teams user should have all features."""
        user = User.objects.create_user(
            email="teams@test.com",
            password="test",
            subscription_tier="teams",
            subscription_status="active",
        )

        assert user_has_feature(user, "cloud_sync") is True
        assert user_has_feature(user, "shared_soundboards") is True
        assert user_has_feature(user, "team_management") is True

    def test_unknown_feature_returns_true(self):
        """Unknown feature should return True (not restricted)."""
        user = User.objects.create_user(email="test@test.com", password="test")

        assert user_has_feature(user, "unknown_feature") is True
```

**Step 2: Run tests to verify they fail**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/core/tests/test_permissions.py -v`

Expected: FAIL - permissions module doesn't exist

**Step 3: Create permissions helper**

Create `backend/apps/core/permissions.py`:

```python
"""Core permissions and feature gates."""

from rest_framework import permissions

# Feature to tier mapping
FEATURE_TIERS = {
    "cloud_sync": ["premium", "teams"],
    "sound_search": ["premium", "teams"],
    "ai_generation": ["premium", "teams"],
    "remote_control": ["premium", "teams"],
    "shared_soundboards": ["teams"],
    "team_management": ["teams"],
    "discord_bot": ["teams"],
}


def user_has_feature(user, feature: str) -> bool:
    """Check if user has access to a feature based on subscription tier."""
    required_tiers = FEATURE_TIERS.get(feature)

    if required_tiers is None:
        # Feature not restricted
        return True

    return user.subscription_tier in required_tiers


class CanModifyCloudData(permissions.BasePermission):
    """Allow modification only with active subscription."""

    message = "Your subscription has expired. Your data is read-only."

    def has_permission(self, request, view):
        if request.method in permissions.SAFE_METHODS:
            return True  # GET, HEAD, OPTIONS always allowed

        user = request.user
        if user.subscription_tier == "free":
            return False

        if user.subscription_status in ["cancelled", "past_due"]:
            return False

        return True


class RequiresFeature(permissions.BasePermission):
    """Base permission class for feature requirements."""

    feature = None

    def has_permission(self, request, view):
        if not self.feature:
            return True
        return user_has_feature(request.user, self.feature)


class RequiresCloudSync(RequiresFeature):
    """Requires cloud_sync feature."""

    feature = "cloud_sync"
    message = "Premium subscription required for cloud sync."


class RequiresSoundSearch(RequiresFeature):
    """Requires sound_search feature."""

    feature = "sound_search"
    message = "Premium subscription required for sound search."
```

**Step 4: Run tests to verify they pass**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest apps/core/tests/test_permissions.py -v`

Expected: All 5 tests PASS

**Step 5: Commit**

```bash
git add backend/apps/core/permissions.py backend/apps/core/tests/test_permissions.py
git commit -m "feat(core): add feature permission helpers"
```

---

## Task 12: Run Full Test Suite and Final Verification

**Step 1: Run all tests**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && pytest -v`

Expected: All tests PASS

**Step 2: Run linting**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && ruff check .`

Expected: No errors (or fix any issues)

**Step 3: Run formatting**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && ruff format .`

**Step 4: Final commit if needed**

```bash
git add -A
git commit -m "chore: fix linting and formatting issues"
```

**Step 5: Push to remote**

```bash
git push
```

---

## Summary

This implementation plan covers:

1. **Task 1-2**: Dependencies and configuration
2. **Task 3-4**: User model extensions
3. **Task 5**: Super admin setup command
4. **Task 6-8**: Billing app (Stripe integration)
5. **Task 9-10**: Teams app (CRUD, invitations)
6. **Task 11**: Feature permissions
7. **Task 12**: Final verification

Total estimated tasks: 12
