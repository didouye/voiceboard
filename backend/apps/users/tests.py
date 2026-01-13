"""Tests for users app."""

from django.contrib.auth import get_user_model
from rest_framework import status

User = get_user_model()


class TestUserModel:
    """Tests for User model."""

    def test_create_user(self, db):
        """Test creating a user with email."""
        user = User.objects.create_user(
            email="test@example.com",
            password="testpass123",
        )
        assert user.email == "test@example.com"
        assert user.check_password("testpass123")
        assert not user.is_staff
        assert not user.is_superuser

    def test_create_superuser(self, db):
        """Test creating a superuser."""
        user = User.objects.create_superuser(
            email="admin@example.com",
            password="adminpass123",
        )
        assert user.email == "admin@example.com"
        assert user.is_staff
        assert user.is_superuser

    def test_user_str(self, user):
        """Test user string representation."""
        assert str(user) == user.email


class TestMeView:
    """Tests for /api/auth/me/ endpoint."""

    def test_me_unauthenticated(self, api_client):
        """Test that unauthenticated requests are rejected."""
        response = api_client.get("/api/auth/me/")
        assert response.status_code == status.HTTP_401_UNAUTHORIZED

    def test_me_authenticated(self, authenticated_client, user):
        """Test getting current user profile."""
        response = authenticated_client.get("/api/auth/me/")
        assert response.status_code == status.HTTP_200_OK
        assert response.data["email"] == user.email
        assert response.data["first_name"] == user.first_name

    def test_me_update(self, authenticated_client, user):
        """Test updating user profile."""
        response = authenticated_client.patch(
            "/api/auth/me/",
            {"first_name": "Updated"},
        )
        assert response.status_code == status.HTTP_200_OK
        assert response.data["first_name"] == "Updated"


class TestGoogleAuthURL:
    """Tests for /api/auth/google/url/ endpoint."""

    def test_get_google_auth_url(self, api_client):
        """Test getting Google OAuth URL."""
        response = api_client.get(
            "/api/auth/google/url/",
            {"redirect_uri": "http://localhost"},
        )
        assert response.status_code == status.HTTP_200_OK
        assert "auth_url" in response.data
        assert "state" in response.data
        assert "accounts.google.com" in response.data["auth_url"]


class TestDiscordAuthURL:
    """Tests for /api/auth/discord/url/ endpoint."""

    def test_get_discord_auth_url(self, api_client):
        """Test getting Discord OAuth URL."""
        response = api_client.get(
            "/api/auth/discord/url/",
            {"redirect_uri": "http://localhost"},
        )
        assert response.status_code == status.HTTP_200_OK
        assert "auth_url" in response.data
        assert "state" in response.data
        assert "discord.com" in response.data["auth_url"]
