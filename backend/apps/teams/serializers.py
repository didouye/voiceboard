"""Team serializers."""

from rest_framework import serializers

from apps.users.serializers import UserPublicSerializer

from .models import Team, TeamMembership


class TeamMembershipSerializer(serializers.ModelSerializer):
    """Serializer for team membership."""

    user = UserPublicSerializer(read_only=True)

    class Meta:
        model = TeamMembership
        fields = ["user", "role", "joined_at"]


class TeamSerializer(serializers.ModelSerializer):
    """Serializer for Team."""

    owner = UserPublicSerializer(read_only=True)
    members = TeamMembershipSerializer(source="teammembership_set", many=True, read_only=True)
    member_count = serializers.IntegerField(read_only=True)
    max_members = serializers.IntegerField(read_only=True)

    class Meta:
        model = Team
        fields = [
            "id",
            "name",
            "owner",
            "members",
            "member_count",
            "max_members",
            "created_at",
        ]
        read_only_fields = ["id", "owner", "created_at"]


class TeamCreateSerializer(serializers.ModelSerializer):
    """Serializer for creating a team."""

    class Meta:
        model = Team
        fields = ["name"]


class InviteSerializer(serializers.Serializer):
    """Serializer for team invitation."""

    email = serializers.EmailField()
