"""Social authentication views for Google and Discord OAuth."""

import secrets
from urllib.parse import urlencode

import requests
from django.conf import settings
from django.contrib.auth import get_user_model
from rest_framework import status
from rest_framework.permissions import AllowAny
from rest_framework.response import Response
from rest_framework.views import APIView
from rest_framework_simplejwt.tokens import RefreshToken

User = get_user_model()


def get_tokens_for_user(user):
    """Generate JWT tokens for a user."""
    refresh = RefreshToken.for_user(user)
    return {
        "access": str(refresh.access_token),
        "refresh": str(refresh),
    }


class GoogleAuthURLView(APIView):
    """Get Google OAuth authorization URL."""

    permission_classes = [AllowAny]

    def get(self, request):
        redirect_uri = request.query_params.get("redirect_uri", "")
        state = secrets.token_urlsafe(32)

        params = {
            "client_id": settings.SOCIALACCOUNT_PROVIDERS["google"]["APP"]["client_id"],
            "redirect_uri": redirect_uri,
            "response_type": "code",
            "scope": "openid email profile",
            "state": state,
            "access_type": "online",
        }

        auth_url = f"https://accounts.google.com/o/oauth2/v2/auth?{urlencode(params)}"

        return Response(
            {
                "auth_url": auth_url,
                "state": state,
            }
        )


class GoogleCallbackView(APIView):
    """Exchange Google auth code for JWT tokens."""

    permission_classes = [AllowAny]

    def post(self, request):
        code = request.data.get("code")
        redirect_uri = request.data.get("redirect_uri")

        if not code or not redirect_uri:
            return Response(
                {"detail": "code and redirect_uri are required"}, status=status.HTTP_400_BAD_REQUEST
            )

        # Exchange code for tokens
        token_response = requests.post(
            "https://oauth2.googleapis.com/token",
            data={
                "client_id": settings.SOCIALACCOUNT_PROVIDERS["google"]["APP"]["client_id"],
                "client_secret": settings.SOCIALACCOUNT_PROVIDERS["google"]["APP"]["secret"],
                "code": code,
                "grant_type": "authorization_code",
                "redirect_uri": redirect_uri,
            },
            timeout=10,
        )

        if not token_response.ok:
            return Response(
                {"detail": "Failed to exchange code for tokens"}, status=status.HTTP_400_BAD_REQUEST
            )

        tokens = token_response.json()
        access_token = tokens.get("access_token")

        # Get user info
        user_response = requests.get(
            "https://www.googleapis.com/oauth2/v2/userinfo",
            headers={"Authorization": f"Bearer {access_token}"},
            timeout=10,
        )

        if not user_response.ok:
            return Response(
                {"detail": "Failed to get user info"}, status=status.HTTP_400_BAD_REQUEST
            )

        user_info = user_response.json()
        email = user_info.get("email")
        google_id = user_info.get("id")

        if not email:
            return Response(
                {"detail": "Email not provided by Google"}, status=status.HTTP_400_BAD_REQUEST
            )

        # Get or create user
        user, created = User.objects.get_or_create(
            email=email,
            defaults={
                "google_id": google_id,
                "first_name": user_info.get("given_name", ""),
                "last_name": user_info.get("family_name", ""),
                "avatar_url": user_info.get("picture", ""),
            },
        )

        # Update Google ID if not set
        if not user.google_id:
            user.google_id = google_id
            user.save(update_fields=["google_id"])

        # Generate JWT tokens
        jwt_tokens = get_tokens_for_user(user)

        return Response(
            {
                **jwt_tokens,
                "user": {
                    "id": user.id,
                    "email": user.email,
                    "first_name": user.first_name,
                    "last_name": user.last_name,
                    "avatar_url": user.avatar_url,
                },
                "created": created,
            }
        )


class DiscordAuthURLView(APIView):
    """Get Discord OAuth authorization URL."""

    permission_classes = [AllowAny]

    def get(self, request):
        redirect_uri = request.query_params.get("redirect_uri", "")
        state = secrets.token_urlsafe(32)

        params = {
            "client_id": settings.SOCIALACCOUNT_PROVIDERS["discord"]["APP"]["client_id"],
            "redirect_uri": redirect_uri,
            "response_type": "code",
            "scope": "identify email",
            "state": state,
        }

        auth_url = f"https://discord.com/api/oauth2/authorize?{urlencode(params)}"

        return Response(
            {
                "auth_url": auth_url,
                "state": state,
            }
        )


class DiscordCallbackView(APIView):
    """Exchange Discord auth code for JWT tokens."""

    permission_classes = [AllowAny]

    def post(self, request):
        code = request.data.get("code")
        redirect_uri = request.data.get("redirect_uri")

        if not code or not redirect_uri:
            return Response(
                {"detail": "code and redirect_uri are required"}, status=status.HTTP_400_BAD_REQUEST
            )

        # Exchange code for tokens
        token_response = requests.post(
            "https://discord.com/api/oauth2/token",
            data={
                "client_id": settings.SOCIALACCOUNT_PROVIDERS["discord"]["APP"]["client_id"],
                "client_secret": settings.SOCIALACCOUNT_PROVIDERS["discord"]["APP"]["secret"],
                "code": code,
                "grant_type": "authorization_code",
                "redirect_uri": redirect_uri,
            },
            headers={"Content-Type": "application/x-www-form-urlencoded"},
            timeout=10,
        )

        if not token_response.ok:
            return Response(
                {"detail": "Failed to exchange code for tokens"}, status=status.HTTP_400_BAD_REQUEST
            )

        tokens = token_response.json()
        access_token = tokens.get("access_token")

        # Get user info
        user_response = requests.get(
            "https://discord.com/api/users/@me",
            headers={"Authorization": f"Bearer {access_token}"},
            timeout=10,
        )

        if not user_response.ok:
            return Response(
                {"detail": "Failed to get user info"}, status=status.HTTP_400_BAD_REQUEST
            )

        user_info = user_response.json()
        email = user_info.get("email")
        discord_id = user_info.get("id")
        username = user_info.get("username", "")
        avatar_hash = user_info.get("avatar")

        if not email:
            return Response(
                {"detail": "Email not provided by Discord. Make sure to grant email permission."},
                status=status.HTTP_400_BAD_REQUEST,
            )

        # Build avatar URL
        avatar_url = ""
        if avatar_hash:
            avatar_url = f"https://cdn.discordapp.com/avatars/{discord_id}/{avatar_hash}.png"

        # Get or create user
        user, created = User.objects.get_or_create(
            email=email,
            defaults={
                "discord_id": discord_id,
                "first_name": username,
                "avatar_url": avatar_url,
            },
        )

        # Update Discord ID if not set
        if not user.discord_id:
            user.discord_id = discord_id
            user.save(update_fields=["discord_id"])

        # Generate JWT tokens
        jwt_tokens = get_tokens_for_user(user)

        return Response(
            {
                **jwt_tokens,
                "user": {
                    "id": user.id,
                    "email": user.email,
                    "first_name": user.first_name,
                    "last_name": user.last_name,
                    "avatar_url": user.avatar_url,
                },
                "created": created,
            }
        )
