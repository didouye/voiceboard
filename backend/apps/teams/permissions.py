"""Team permissions."""

from rest_framework import permissions


class HasTeamsSubscription(permissions.BasePermission):
    """Requires active Teams subscription."""

    message = "Teams subscription required."

    def has_permission(self, request, view):
        user = request.user
        return user.subscription_tier == "teams" and user.subscription_status == "active"


class IsTeamOwner(permissions.BasePermission):
    """Requires user to be team owner."""

    message = "Only team owner can perform this action."

    def has_object_permission(self, request, view, obj):
        return obj.owner == request.user


class IsTeamMember(permissions.BasePermission):
    """Requires user to be team member or owner."""

    def has_object_permission(self, request, view, obj):
        return obj.owner == request.user or request.user in obj.members.all()
