"""Tests for billing services."""

from unittest.mock import MagicMock, patch

import pytest
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
            url = BillingService.create_customer_portal_session(user, "https://app.test/settings")

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
