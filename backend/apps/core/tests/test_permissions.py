"""Tests for core permissions."""

import pytest
from django.contrib.auth import get_user_model

from apps.core.permissions import user_has_feature

User = get_user_model()


@pytest.mark.django_db
class TestUserHasFeature:
    """Tests for user_has_feature helper."""

    def test_free_user_has_no_premium_features(self):
        """Free user should not have premium features."""
        user = User.objects.create_user(email="free@test.com", password="test")

        assert user_has_feature(user, "cloud_sync") is False
        assert user_has_feature(user, "sound_search") is False
        assert user_has_feature(user, "ai_generation") is False

    def test_premium_user_has_premium_features(self):
        """Premium user should have premium features."""
        user = User.objects.create_user(
            email="premium@test.com",
            password="test",
            subscription_tier="premium",
            subscription_status="active",
        )

        assert user_has_feature(user, "cloud_sync") is True
        assert user_has_feature(user, "sound_search") is True
        assert user_has_feature(user, "remote_control") is True

    def test_premium_user_no_team_features(self):
        """Premium user should not have team features."""
        user = User.objects.create_user(
            email="premium@test.com",
            password="test",
            subscription_tier="premium",
            subscription_status="active",
        )

        assert user_has_feature(user, "shared_soundboards") is False
        assert user_has_feature(user, "team_management") is False

    def test_teams_user_has_all_features(self):
        """Teams user should have all features."""
        user = User.objects.create_user(
            email="teams@test.com",
            password="test",
            subscription_tier="teams",
            subscription_status="active",
        )

        assert user_has_feature(user, "cloud_sync") is True
        assert user_has_feature(user, "shared_soundboards") is True
        assert user_has_feature(user, "team_management") is True

    def test_unknown_feature_returns_true(self):
        """Unknown feature should return True (not restricted)."""
        user = User.objects.create_user(email="test@test.com", password="test")

        assert user_has_feature(user, "unknown_feature") is True
