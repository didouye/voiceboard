"""User URL patterns."""

from django.urls import path
from rest_framework_simplejwt.views import TokenRefreshView

from .views import MeView, LogoutView
from .social_auth import (
    GoogleAuthURLView,
    GoogleCallbackView,
    DiscordAuthURLView,
    DiscordCallbackView,
)

urlpatterns = [
    # User profile
    path("me/", MeView.as_view(), name="auth-me"),

    # JWT token management
    path("refresh/", TokenRefreshView.as_view(), name="auth-refresh"),
    path("logout/", LogoutView.as_view(), name="auth-logout"),

    # Social auth - Google
    path("google/url/", GoogleAuthURLView.as_view(), name="auth-google-url"),
    path("google/callback/", GoogleCallbackView.as_view(), name="auth-google-callback"),

    # Social auth - Discord
    path("discord/url/", DiscordAuthURLView.as_view(), name="auth-discord-url"),
    path("discord/callback/", DiscordCallbackView.as_view(), name="auth-discord-callback"),
]
