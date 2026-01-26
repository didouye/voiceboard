"""Core permissions and feature gates."""

from rest_framework import permissions

# Feature to tier mapping
FEATURE_TIERS = {
    "cloud_sync": ["premium", "teams"],
    "sound_search": ["premium", "teams"],
    "ai_generation": ["premium", "teams"],
    "remote_control": ["premium", "teams"],
    "shared_soundboards": ["teams"],
    "team_management": ["teams"],
    "discord_bot": ["teams"],
}


def user_has_feature(user, feature: str) -> bool:
    """Check if user has access to a feature based on subscription tier."""
    required_tiers = FEATURE_TIERS.get(feature)

    if required_tiers is None:
        # Feature not restricted
        return True

    return user.subscription_tier in required_tiers


class CanModifyCloudData(permissions.BasePermission):
    """Allow modification only with active subscription."""

    message = "Your subscription has expired. Your data is read-only."

    def has_permission(self, request, view):
        if request.method in permissions.SAFE_METHODS:
            return True  # GET, HEAD, OPTIONS always allowed

        user = request.user
        if user.subscription_tier == "free":
            return False

        if user.subscription_status in ["cancelled", "past_due"]:
            return False

        return True


class RequiresFeature(permissions.BasePermission):
    """Base permission class for feature requirements."""

    feature = None

    def has_permission(self, request, view):
        if not self.feature:
            return True
        return user_has_feature(request.user, self.feature)


class RequiresCloudSync(RequiresFeature):
    """Requires cloud_sync feature."""

    feature = "cloud_sync"
    message = "Premium subscription required for cloud sync."


class RequiresSoundSearch(RequiresFeature):
    """Requires sound_search feature."""

    feature = "sound_search"
    message = "Premium subscription required for sound search."
