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
