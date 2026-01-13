"""User serializers for REST API."""

from rest_framework import serializers
from django.contrib.auth import get_user_model

User = get_user_model()


class UserSerializer(serializers.ModelSerializer):
    """Serializer for user profile."""

    class Meta:
        model = User
        fields = ["id", "email", "first_name", "last_name", "avatar_url", "date_joined"]
        read_only_fields = ["id", "email", "date_joined"]


class UserPublicSerializer(serializers.ModelSerializer):
    """Public serializer for user (limited fields)."""

    class Meta:
        model = User
        fields = ["id", "first_name", "last_name", "avatar_url"]
        read_only_fields = fields
