"""Tests for billing views."""

import json
from unittest.mock import patch

import pytest
from django.contrib.auth import get_user_model
from django.urls import reverse
from rest_framework.test import APIClient

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

    def test_subscription_requires_auth(self, api_client):
        """Should require authentication."""
        response = api_client.get(reverse("billing-subscription"))
        assert response.status_code == 401

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

    def test_portal_requires_auth(self, api_client):
        """Should require authentication."""
        response = api_client.post(reverse("billing-portal"))
        assert response.status_code == 401

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


@pytest.mark.django_db
class TestStripeWebhook:
    """Tests for Stripe webhook endpoint."""

    def test_webhook_rejects_invalid_signature(self, api_client, settings):
        """Should reject invalid webhook signature."""
        settings.STRIPE_WEBHOOK_SECRET = "whsec_test"
        response = api_client.post(
            reverse("billing-webhook"),
            data=json.dumps({"type": "test"}),
            content_type="application/json",
            HTTP_STRIPE_SIGNATURE="invalid",
        )
        assert response.status_code == 400

    def test_webhook_handles_checkout_completed(self, api_client, settings):
        """Should handle checkout.session.completed event."""
        settings.STRIPE_WEBHOOK_SECRET = "whsec_test"
        user = User.objects.create_user(
            email="webhook@test.com",
            password="test",
            stripe_customer_id="cus_webhook",
        )

        event_data = {
            "type": "checkout.session.completed",
            "data": {
                "object": {
                    "customer": "cus_webhook",
                    "metadata": {"plan": "premium_monthly"},
                }
            },
        }

        with patch("stripe.Webhook.construct_event") as mock_construct:
            mock_construct.return_value = event_data
            response = api_client.post(
                reverse("billing-webhook"),
                data=json.dumps(event_data),
                content_type="application/json",
                HTTP_STRIPE_SIGNATURE="valid_sig",
            )

        assert response.status_code == 200
        user.refresh_from_db()
        assert user.subscription_tier == "premium"
        assert user.subscription_status == "active"

    def test_webhook_handles_subscription_deleted(self, api_client, settings):
        """Should handle customer.subscription.deleted event."""
        settings.STRIPE_WEBHOOK_SECRET = "whsec_test"
        user = User.objects.create_user(
            email="cancel@test.com",
            password="test",
            stripe_customer_id="cus_cancel",
            subscription_tier="premium",
            subscription_status="active",
        )

        event_data = {
            "type": "customer.subscription.deleted",
            "data": {
                "object": {
                    "customer": "cus_cancel",
                }
            },
        }

        with patch("stripe.Webhook.construct_event") as mock_construct:
            mock_construct.return_value = event_data
            response = api_client.post(
                reverse("billing-webhook"),
                data=json.dumps(event_data),
                content_type="application/json",
                HTTP_STRIPE_SIGNATURE="valid_sig",
            )

        assert response.status_code == 200
        user.refresh_from_db()
        assert user.subscription_tier == "free"
        assert user.subscription_status == "cancelled"

    def test_webhook_handles_payment_failed(self, api_client, settings):
        """Should handle invoice.payment_failed event."""
        settings.STRIPE_WEBHOOK_SECRET = "whsec_test"
        user = User.objects.create_user(
            email="failed@test.com",
            password="test",
            stripe_customer_id="cus_failed",
            subscription_tier="premium",
            subscription_status="active",
        )

        event_data = {
            "type": "invoice.payment_failed",
            "data": {
                "object": {
                    "customer": "cus_failed",
                }
            },
        }

        with patch("stripe.Webhook.construct_event") as mock_construct:
            mock_construct.return_value = event_data
            response = api_client.post(
                reverse("billing-webhook"),
                data=json.dumps(event_data),
                content_type="application/json",
                HTTP_STRIPE_SIGNATURE="valid_sig",
            )

        assert response.status_code == 200
        user.refresh_from_db()
        assert user.subscription_status == "past_due"

    def test_webhook_handles_subscription_updated(self, api_client, settings):
        """Should handle customer.subscription.updated event."""
        settings.STRIPE_WEBHOOK_SECRET = "whsec_test"
        settings.STRIPE_PRICES = {"premium_monthly": "price_premium"}
        user = User.objects.create_user(
            email="updated@test.com",
            password="test",
            stripe_customer_id="cus_updated",
            subscription_tier="free",
            subscription_status="none",
        )

        event_data = {
            "type": "customer.subscription.updated",
            "data": {
                "object": {
                    "customer": "cus_updated",
                    "status": "active",
                    "current_period_end": 1735689600,
                    "items": {"data": [{"price": {"id": "price_premium"}}]},
                }
            },
        }

        with patch("stripe.Webhook.construct_event") as mock_construct:
            mock_construct.return_value = event_data
            response = api_client.post(
                reverse("billing-webhook"),
                data=json.dumps(event_data),
                content_type="application/json",
                HTTP_STRIPE_SIGNATURE="valid_sig",
            )

        assert response.status_code == 200
        user.refresh_from_db()
        assert user.subscription_tier == "premium"
        assert user.subscription_status == "active"
