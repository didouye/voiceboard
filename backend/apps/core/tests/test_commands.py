"""Tests for management commands."""

from io import StringIO

import pytest
from django.contrib.auth import get_user_model
from django.core.management import call_command

User = get_user_model()


@pytest.mark.django_db
class TestSetupSuperadminCommand:
    """Tests for setup_superadmin command."""

    def test_creates_superadmin_when_env_set(self, settings):
        """Should create superadmin from SUPER_ADMIN_EMAIL."""
        settings.SUPER_ADMIN_EMAIL = "admin@test.com"
        out = StringIO()

        call_command("setup_superadmin", stdout=out)

        user = User.objects.get(email="admin@test.com")
        assert user.is_superuser is True
        assert user.is_staff is True
        assert user.subscription_tier == "teams"
        assert user.subscription_status == "active"
        assert "Super admin created" in out.getvalue()

    def test_skips_when_env_not_set(self, settings):
        """Should skip when SUPER_ADMIN_EMAIL not set."""
        settings.SUPER_ADMIN_EMAIL = ""
        out = StringIO()

        call_command("setup_superadmin", stdout=out)

        assert User.objects.filter(is_superuser=True).count() == 0
        assert "not set" in out.getvalue()

    def test_skips_when_superadmin_exists(self, settings):
        """Should skip when a superadmin already exists."""
        User.objects.create_superuser(email="existing@test.com", password="test")
        settings.SUPER_ADMIN_EMAIL = "new@test.com"
        out = StringIO()

        call_command("setup_superadmin", stdout=out)

        assert not User.objects.filter(email="new@test.com").exists()
        assert "already exists" in out.getvalue()

    def test_promotes_existing_user(self, settings):
        """Should promote existing user to superadmin."""
        user = User.objects.create_user(email="admin@test.com", password="test")
        assert user.is_superuser is False

        settings.SUPER_ADMIN_EMAIL = "admin@test.com"
        out = StringIO()

        call_command("setup_superadmin", stdout=out)

        user.refresh_from_db()
        assert user.is_superuser is True
        assert user.is_staff is True
        assert "Super admin promoted" in out.getvalue()
