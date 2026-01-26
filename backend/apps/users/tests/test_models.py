"""Tests for User model."""

import pytest
from django.contrib.auth import get_user_model

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
