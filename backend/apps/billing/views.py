"""Billing views for REST API.

Note: Views will be implemented in Task 8.
This file contains placeholder imports for urls.py to reference.
"""

from rest_framework.views import APIView


class CheckoutView(APIView):
    """Create Stripe Checkout session."""

    pass


class CustomerPortalView(APIView):
    """Create Stripe Customer Portal session."""

    pass


class SubscriptionView(APIView):
    """Get current subscription status."""

    pass


def stripe_webhook(request):
    """Handle Stripe webhook events."""
    pass
