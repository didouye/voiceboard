"""Billing services for Stripe integration."""

from datetime import datetime

import stripe
from django.conf import settings
from django.contrib.auth import get_user_model
from django.utils import timezone

from .constants import VALID_PLANS, get_price_id

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
        current_period_end: int | None = None,
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
