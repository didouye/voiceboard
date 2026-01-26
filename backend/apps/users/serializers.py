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
