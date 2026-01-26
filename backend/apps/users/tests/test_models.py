"""Tests for User model."""

import pytest
from django.contrib.auth import get_user_model

from apps.users.serializers import UserSerializer

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
