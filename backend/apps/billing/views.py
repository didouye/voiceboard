"""Billing views for REST API."""

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
    CheckoutResponseSerializer,
    CheckoutSerializer,
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
        event = stripe.Webhook.construct_event(payload, sig_header, settings.STRIPE_WEBHOOK_SECRET)
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
    # Get the user's current tier to preserve it
    from django.contrib.auth import get_user_model

    User = get_user_model()
    try:
        user = User.objects.get(stripe_customer_id=customer_id)
        current_tier = user.subscription_tier
    except User.DoesNotExist:
        current_tier = "premium"  # Fallback

    BillingService.update_subscription(
        customer_id=customer_id,
        tier=current_tier,  # Keep current tier
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
