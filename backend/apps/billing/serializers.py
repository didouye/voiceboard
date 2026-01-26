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
