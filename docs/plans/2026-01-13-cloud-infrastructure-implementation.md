# Cloud Infrastructure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create the Django backend infrastructure with REST API, JWT authentication, social OAuth, and Docker deployment.

**Architecture:** Django REST Framework backend with Gunicorn (HTTP) and Daphne (WebSocket), PostgreSQL database, Redis cache, running in Docker Compose behind Nginx reverse proxy. Authentication via Google/Discord OAuth with JWT tokens.

**Tech Stack:** Django 5.1, DRF, SimpleJWT, django-allauth, Channels, PostgreSQL, Redis, Docker, Nginx, uv

**Design Reference:** `docs/plans/2026-01-12-cloud-infrastructure-design.md`

---

## Part 1: Project Setup

### Task 1: Create backend directory structure

**Files:**
- Create: `backend/`

**Step 1: Create directory**

```bash
mkdir -p backend
```

**Step 2: Verify**

```bash
ls -la backend/
```

Expected: Empty directory exists

**Step 3: Commit**

```bash
git add backend/
git commit --allow-empty -m "chore: create backend directory for Django project"
```

---

### Task 2: Initialize Python project with uv

**Files:**
- Create: `backend/pyproject.toml`
- Create: `backend/.python-version`

**Step 1: Initialize uv project**

```bash
cd backend && uv init --name voiceboard-backend --no-readme
```

**Step 2: Set Python version**

```bash
cd backend && echo "3.12" > .python-version
```

**Step 3: Verify pyproject.toml exists**

```bash
cat backend/pyproject.toml
```

Expected: Shows `[project]` section with name "voiceboard-backend"

**Step 4: Remove generated hello.py**

```bash
rm -f backend/hello.py
```

**Step 5: Commit**

```bash
git add backend/pyproject.toml backend/.python-version
git commit -m "chore: initialize Python project with uv"
```

---

### Task 3: Add Django dependencies

**Files:**
- Modify: `backend/pyproject.toml`

**Step 1: Add dependencies to pyproject.toml**

Replace the `dependencies` section in `backend/pyproject.toml`:

```toml
[project]
name = "voiceboard-backend"
version = "0.1.0"
requires-python = ">=3.12"
dependencies = [
    "django>=5.1",
    "djangorestframework>=3.15",
    "djangorestframework-simplejwt>=5.3",
    "django-allauth[socialaccount]>=65.0",
    "channels>=4.0",
    "channels-redis>=4.2",
    "psycopg[binary]>=3.2",
    "boto3>=1.35",
    "python-dotenv>=1.0",
    "django-cors-headers>=4.0",
]

[project.optional-dependencies]
dev = ["django-debug-toolbar", "ruff", "pytest", "pytest-django"]
prod = ["gunicorn", "daphne", "sentry-sdk[django]"]
```

**Step 2: Sync dependencies**

```bash
cd backend && uv sync
```

Expected: Creates `uv.lock` and installs packages

**Step 3: Verify Django is installed**

```bash
cd backend && uv run django-admin --version
```

Expected: Shows Django version (5.1.x)

**Step 4: Commit**

```bash
git add backend/pyproject.toml backend/uv.lock
git commit -m "chore: add Django and DRF dependencies"
```

---

### Task 4: Create Django project structure

**Files:**
- Create: `backend/config/__init__.py`
- Create: `backend/config/urls.py`
- Create: `backend/config/wsgi.py`
- Create: `backend/config/asgi.py`
- Create: `backend/manage.py`

**Step 1: Create Django project**

```bash
cd backend && uv run django-admin startproject config .
```

**Step 2: Verify structure**

```bash
ls -la backend/config/
```

Expected: Shows `__init__.py`, `settings.py`, `urls.py`, `wsgi.py`, `asgi.py`

**Step 3: Verify manage.py works**

```bash
cd backend && uv run python manage.py --help
```

Expected: Shows Django management commands

**Step 4: Commit**

```bash
git add backend/config/ backend/manage.py
git commit -m "chore: create Django project structure"
```

---

### Task 5: Create apps directory

**Files:**
- Create: `backend/apps/__init__.py`
- Create: `backend/apps/core/__init__.py`
- Create: `backend/apps/users/__init__.py`

**Step 1: Create directories**

```bash
mkdir -p backend/apps/core backend/apps/users
touch backend/apps/__init__.py backend/apps/core/__init__.py backend/apps/users/__init__.py
```

**Step 2: Verify**

```bash
ls -la backend/apps/
```

Expected: Shows `__init__.py`, `core/`, `users/`

**Step 3: Commit**

```bash
git add backend/apps/
git commit -m "chore: create apps directory structure"
```

---

## Part 2: Django Configuration

### Task 6: Split settings into base/development/production

**Files:**
- Create: `backend/config/settings/__init__.py`
- Create: `backend/config/settings/base.py`
- Create: `backend/config/settings/development.py`
- Create: `backend/config/settings/production.py`
- Delete: `backend/config/settings.py`

**Step 1: Create settings directory**

```bash
mkdir -p backend/config/settings
```

**Step 2: Create base.py**

Create `backend/config/settings/base.py`:

```python
"""Base settings shared across all environments."""

from pathlib import Path
from datetime import timedelta
import os

from dotenv import load_dotenv

# Build paths inside the project like this: BASE_DIR / 'subdir'.
BASE_DIR = Path(__file__).resolve().parent.parent.parent

# Load environment variables
load_dotenv(BASE_DIR / ".env")

# SECURITY WARNING: keep the secret key used in production secret!
SECRET_KEY = os.environ.get("DJANGO_SECRET_KEY", "dev-secret-key-change-in-production")

# Application definition
INSTALLED_APPS = [
    "django.contrib.admin",
    "django.contrib.auth",
    "django.contrib.contenttypes",
    "django.contrib.sessions",
    "django.contrib.messages",
    "django.contrib.staticfiles",
    "django.contrib.sites",
    # Third-party
    "rest_framework",
    "rest_framework_simplejwt",
    "rest_framework_simplejwt.token_blacklist",
    "corsheaders",
    "allauth",
    "allauth.account",
    "allauth.socialaccount",
    "allauth.socialaccount.providers.google",
    "allauth.socialaccount.providers.discord",
    # Local apps
    "apps.core",
    "apps.users",
]

MIDDLEWARE = [
    "django.middleware.security.SecurityMiddleware",
    "corsheaders.middleware.CorsMiddleware",
    "django.contrib.sessions.middleware.SessionMiddleware",
    "django.middleware.common.CommonMiddleware",
    "django.middleware.csrf.CsrfViewMiddleware",
    "django.contrib.auth.middleware.AuthenticationMiddleware",
    "django.contrib.messages.middleware.MessageMiddleware",
    "django.middleware.clickjacking.XFrameOptionsMiddleware",
    "allauth.account.middleware.AccountMiddleware",
]

ROOT_URLCONF = "config.urls"

TEMPLATES = [
    {
        "BACKEND": "django.template.backends.django.DjangoTemplates",
        "DIRS": [],
        "APP_DIRS": True,
        "OPTIONS": {
            "context_processors": [
                "django.template.context_processors.debug",
                "django.template.context_processors.request",
                "django.contrib.auth.context_processors.auth",
                "django.contrib.messages.context_processors.messages",
            ],
        },
    },
]

WSGI_APPLICATION = "config.wsgi.application"
ASGI_APPLICATION = "config.asgi.application"

# Database
DATABASES = {
    "default": {
        "ENGINE": "django.db.backends.postgresql",
        "NAME": os.environ.get("POSTGRES_DB", "voiceboard"),
        "USER": os.environ.get("POSTGRES_USER", "voiceboard"),
        "PASSWORD": os.environ.get("POSTGRES_PASSWORD", "voiceboard"),
        "HOST": os.environ.get("POSTGRES_HOST", "localhost"),
        "PORT": os.environ.get("POSTGRES_PORT", "5432"),
    }
}

# Password validation
AUTH_PASSWORD_VALIDATORS = [
    {"NAME": "django.contrib.auth.password_validation.UserAttributeSimilarityValidator"},
    {"NAME": "django.contrib.auth.password_validation.MinimumLengthValidator"},
    {"NAME": "django.contrib.auth.password_validation.CommonPasswordValidator"},
    {"NAME": "django.contrib.auth.password_validation.NumericPasswordValidator"},
]

# Internationalization
LANGUAGE_CODE = "en-us"
TIME_ZONE = "UTC"
USE_I18N = True
USE_TZ = True

# Static files (CSS, JavaScript, Images)
STATIC_URL = "static/"
STATIC_ROOT = BASE_DIR / "static"

# Default primary key field type
DEFAULT_AUTO_FIELD = "django.db.models.BigAutoField"

# Custom user model
AUTH_USER_MODEL = "users.User"

# Django REST Framework
REST_FRAMEWORK = {
    "DEFAULT_AUTHENTICATION_CLASSES": [
        "rest_framework_simplejwt.authentication.JWTAuthentication",
    ],
    "DEFAULT_PERMISSION_CLASSES": [
        "rest_framework.permissions.IsAuthenticated",
    ],
}

# Simple JWT
SIMPLE_JWT = {
    "ACCESS_TOKEN_LIFETIME": timedelta(minutes=int(os.environ.get("JWT_ACCESS_TOKEN_LIFETIME", "15"))),
    "REFRESH_TOKEN_LIFETIME": timedelta(minutes=int(os.environ.get("JWT_REFRESH_TOKEN_LIFETIME", "43200"))),
    "ROTATE_REFRESH_TOKENS": True,
    "BLACKLIST_AFTER_ROTATION": True,
    "AUTH_HEADER_TYPES": ("Bearer",),
}

# Django Sites Framework
SITE_ID = 1

# Django Allauth
ACCOUNT_USER_MODEL_USERNAME_FIELD = None
ACCOUNT_EMAIL_REQUIRED = True
ACCOUNT_USERNAME_REQUIRED = False
ACCOUNT_AUTHENTICATION_METHOD = "email"

# Social auth providers
SOCIALACCOUNT_PROVIDERS = {
    "google": {
        "APP": {
            "client_id": os.environ.get("GOOGLE_CLIENT_ID", ""),
            "secret": os.environ.get("GOOGLE_CLIENT_SECRET", ""),
        },
        "SCOPE": ["profile", "email"],
        "AUTH_PARAMS": {"access_type": "online"},
    },
    "discord": {
        "APP": {
            "client_id": os.environ.get("DISCORD_CLIENT_ID", ""),
            "secret": os.environ.get("DISCORD_CLIENT_SECRET", ""),
        },
        "SCOPE": ["identify", "email"],
    },
}

# Redis (for Channels and cache)
REDIS_URL = os.environ.get("REDIS_URL", "redis://localhost:6379/0")

# Channels
CHANNEL_LAYERS = {
    "default": {
        "BACKEND": "channels_redis.core.RedisChannelLayer",
        "CONFIG": {
            "hosts": [REDIS_URL],
        },
    },
}

# CORS
CORS_ALLOWED_ORIGINS = os.environ.get("CORS_ALLOWED_ORIGINS", "http://localhost:4200").split(",")
CORS_ALLOW_CREDENTIALS = True
```

**Step 3: Create development.py**

Create `backend/config/settings/development.py`:

```python
"""Development settings."""

from .base import *  # noqa: F401, F403

DEBUG = True
ALLOWED_HOSTS = ["localhost", "127.0.0.1"]

# Use SQLite for local development without Docker
DATABASES = {
    "default": {
        "ENGINE": "django.db.backends.sqlite3",
        "NAME": BASE_DIR / "db.sqlite3",  # noqa: F405
    }
}

# Disable Redis in development (use in-memory channel layer)
CHANNEL_LAYERS = {
    "default": {
        "BACKEND": "channels.layers.InMemoryChannelLayer",
    },
}

# Debug toolbar
INSTALLED_APPS += ["debug_toolbar"]  # noqa: F405
MIDDLEWARE.insert(0, "debug_toolbar.middleware.DebugToolbarMiddleware")  # noqa: F405
INTERNAL_IPS = ["127.0.0.1"]

# CORS - allow all in development
CORS_ALLOW_ALL_ORIGINS = True
```

**Step 4: Create production.py**

Create `backend/config/settings/production.py`:

```python
"""Production settings."""

from .base import *  # noqa: F401, F403
import sentry_sdk

DEBUG = False
ALLOWED_HOSTS = os.environ.get("DJANGO_ALLOWED_HOSTS", "").split(",")  # noqa: F405

# Security settings
SECURE_BROWSER_XSS_FILTER = True
SECURE_CONTENT_TYPE_NOSNIFF = True
X_FRAME_OPTIONS = "DENY"
SECURE_PROXY_SSL_HEADER = ("HTTP_X_FORWARDED_PROTO", "https")

# Sentry
SENTRY_DSN = os.environ.get("SENTRY_DSN")  # noqa: F405
if SENTRY_DSN:
    sentry_sdk.init(
        dsn=SENTRY_DSN,
        traces_sample_rate=0.1,
        profiles_sample_rate=0.1,
    )
```

**Step 5: Create __init__.py**

Create `backend/config/settings/__init__.py`:

```python
"""Settings package - import based on DJANGO_SETTINGS_MODULE."""
```

**Step 6: Remove old settings.py**

```bash
rm backend/config/settings.py
```

**Step 7: Update manage.py to use development by default**

Update `backend/manage.py`:

```python
#!/usr/bin/env python
"""Django's command-line utility for administrative tasks."""
import os
import sys


def main():
    """Run administrative tasks."""
    os.environ.setdefault("DJANGO_SETTINGS_MODULE", "config.settings.development")
    try:
        from django.core.management import execute_from_command_line
    except ImportError as exc:
        raise ImportError(
            "Couldn't import Django. Are you sure it's installed and "
            "available on your PYTHONPATH environment variable? Did you "
            "forget to activate a virtual environment?"
        ) from exc
    execute_from_command_line(sys.argv)


if __name__ == "__main__":
    main()
```

**Step 8: Commit**

```bash
git add backend/config/settings/ backend/manage.py
git rm backend/config/settings.py 2>/dev/null || true
git commit -m "feat: split Django settings into base/development/production"
```

---

### Task 7: Update wsgi.py and asgi.py

**Files:**
- Modify: `backend/config/wsgi.py`
- Modify: `backend/config/asgi.py`

**Step 1: Update wsgi.py**

Replace `backend/config/wsgi.py`:

```python
"""WSGI config for Voiceboard backend."""

import os

from django.core.wsgi import get_wsgi_application

os.environ.setdefault("DJANGO_SETTINGS_MODULE", "config.settings.production")

application = get_wsgi_application()
```

**Step 2: Update asgi.py**

Replace `backend/config/asgi.py`:

```python
"""ASGI config for Voiceboard backend with Channels support."""

import os

from channels.routing import ProtocolTypeRouter, URLRouter
from channels.security.websocket import AllowedHostsOriginValidator
from django.core.asgi import get_asgi_application

os.environ.setdefault("DJANGO_SETTINGS_MODULE", "config.settings.production")

# Initialize Django ASGI application early to ensure the AppRegistry
# is populated before importing code that may import ORM models.
django_asgi_app = get_asgi_application()

# Import after Django setup
from apps.core.routing import websocket_urlpatterns  # noqa: E402

application = ProtocolTypeRouter(
    {
        "http": django_asgi_app,
        "websocket": AllowedHostsOriginValidator(
            URLRouter(websocket_urlpatterns)
        ),
    }
)
```

**Step 3: Commit**

```bash
git add backend/config/wsgi.py backend/config/asgi.py
git commit -m "feat: update wsgi.py and asgi.py for production"
```

---

### Task 8: Create core app routing

**Files:**
- Create: `backend/apps/core/routing.py`

**Step 1: Create routing.py**

Create `backend/apps/core/routing.py`:

```python
"""WebSocket URL routing for Channels."""

from django.urls import path

websocket_urlpatterns = [
    # WebSocket routes will be added here
    # Example: path("ws/remote/", RemoteConsumer.as_asgi()),
]
```

**Step 2: Commit**

```bash
git add backend/apps/core/routing.py
git commit -m "feat: add WebSocket routing placeholder"
```

---

### Task 9: Create .env.example for backend

**Files:**
- Create: `backend/.env.example`

**Step 1: Create .env.example**

Create `backend/.env.example`:

```bash
# Django
DJANGO_SECRET_KEY=your-secret-key-here
DJANGO_DEBUG=true
DJANGO_ALLOWED_HOSTS=localhost,127.0.0.1

# Database (not needed for development - uses SQLite)
POSTGRES_DB=voiceboard
POSTGRES_USER=voiceboard
POSTGRES_PASSWORD=secure-password
POSTGRES_HOST=db
POSTGRES_PORT=5432

# Redis (not needed for development - uses in-memory)
REDIS_URL=redis://redis:6379/0

# JWT
JWT_ACCESS_TOKEN_LIFETIME=15
JWT_REFRESH_TOKEN_LIFETIME=43200

# OAuth - Google (https://console.cloud.google.com/)
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=

# OAuth - Discord (https://discord.com/developers/applications)
DISCORD_CLIENT_ID=
DISCORD_CLIENT_SECRET=

# CORS
CORS_ALLOWED_ORIGINS=http://localhost:4200

# Sentry (optional)
SENTRY_DSN=

# Scaleway Object Storage (not implemented yet)
# SCW_ACCESS_KEY=
# SCW_SECRET_KEY=
# SCW_BUCKET_NAME=voiceboard-media
# SCW_REGION=fr-par
# SCW_ENDPOINT_URL=https://s3.fr-par.scw.cloud
# MEDIA_URL=https://media.voiceboard.cloud/
```

**Step 2: Update .gitignore**

Add to root `.gitignore`:

```
# Backend
backend/.env
backend/db.sqlite3
backend/static/
backend/__pycache__/
backend/**/__pycache__/
backend/*.pyc
```

**Step 3: Commit**

```bash
git add backend/.env.example .gitignore
git commit -m "chore: add backend .env.example and update .gitignore"
```

---

### Task 10: Update root urls.py

**Files:**
- Modify: `backend/config/urls.py`

**Step 1: Update urls.py**

Replace `backend/config/urls.py`:

```python
"""URL configuration for Voiceboard backend."""

from django.contrib import admin
from django.urls import path, include
from django.conf import settings

urlpatterns = [
    path("admin/", admin.site.urls),
    path("api/auth/", include("apps.users.urls")),
]

# Debug toolbar (development only)
if settings.DEBUG:
    import debug_toolbar

    urlpatterns = [
        path("__debug__/", include(debug_toolbar.urls)),
    ] + urlpatterns
```

**Step 2: Commit**

```bash
git add backend/config/urls.py
git commit -m "feat: configure root URL routing"
```

---

## Part 3: User App

### Task 11: Create custom User model

**Files:**
- Create: `backend/apps/users/models.py`

**Step 1: Create models.py**

Create `backend/apps/users/models.py`:

```python
"""Custom User model - email-based authentication."""

from django.contrib.auth.models import AbstractUser, BaseUserManager
from django.db import models


class UserManager(BaseUserManager):
    """Custom user manager for email-based authentication."""

    def create_user(self, email, password=None, **extra_fields):
        """Create and return a regular user."""
        if not email:
            raise ValueError("Email is required")
        email = self.normalize_email(email)
        user = self.model(email=email, **extra_fields)
        user.set_password(password)
        user.save(using=self._db)
        return user

    def create_superuser(self, email, password=None, **extra_fields):
        """Create and return a superuser."""
        extra_fields.setdefault("is_staff", True)
        extra_fields.setdefault("is_superuser", True)

        if extra_fields.get("is_staff") is not True:
            raise ValueError("Superuser must have is_staff=True")
        if extra_fields.get("is_superuser") is not True:
            raise ValueError("Superuser must have is_superuser=True")

        return self.create_user(email, password, **extra_fields)


class User(AbstractUser):
    """Custom user model - email-based, no username."""

    username = None
    email = models.EmailField("email address", unique=True)
    avatar_url = models.URLField(blank=True, default="")

    # OAuth provider IDs
    google_id = models.CharField(max_length=255, blank=True, default="")
    discord_id = models.CharField(max_length=255, blank=True, default="")

    objects = UserManager()

    USERNAME_FIELD = "email"
    REQUIRED_FIELDS = []

    class Meta:
        db_table = "users"

    def __str__(self):
        return self.email
```

**Step 2: Commit**

```bash
git add backend/apps/users/models.py
git commit -m "feat: add custom User model with email authentication"
```

---

### Task 12: Create user serializers

**Files:**
- Create: `backend/apps/users/serializers.py`

**Step 1: Create serializers.py**

Create `backend/apps/users/serializers.py`:

```python
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
```

**Step 2: Commit**

```bash
git add backend/apps/users/serializers.py
git commit -m "feat: add user serializers"
```

---

### Task 13: Create user views

**Files:**
- Create: `backend/apps/users/views.py`

**Step 1: Create views.py**

Create `backend/apps/users/views.py`:

```python
"""User views for REST API."""

from rest_framework import generics, permissions, status
from rest_framework.response import Response
from rest_framework.views import APIView
from rest_framework_simplejwt.tokens import RefreshToken
from django.contrib.auth import get_user_model

from .serializers import UserSerializer

User = get_user_model()


class MeView(generics.RetrieveUpdateAPIView):
    """Get or update current user profile."""

    serializer_class = UserSerializer
    permission_classes = [permissions.IsAuthenticated]

    def get_object(self):
        return self.request.user


class LogoutView(APIView):
    """Logout user by blacklisting refresh token."""

    permission_classes = [permissions.IsAuthenticated]

    def post(self, request):
        try:
            refresh_token = request.data.get("refresh")
            if refresh_token:
                token = RefreshToken(refresh_token)
                token.blacklist()
            return Response(status=status.HTTP_204_NO_CONTENT)
        except Exception:
            return Response(
                {"detail": "Invalid token"},
                status=status.HTTP_400_BAD_REQUEST
            )
```

**Step 2: Commit**

```bash
git add backend/apps/users/views.py
git commit -m "feat: add user views (me, logout)"
```

---

### Task 14: Create user URLs

**Files:**
- Create: `backend/apps/users/urls.py`

**Step 1: Create urls.py**

Create `backend/apps/users/urls.py`:

```python
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
```

**Step 2: Commit**

```bash
git add backend/apps/users/urls.py
git commit -m "feat: add user URL routing"
```

---

### Task 15: Create admin registration

**Files:**
- Create: `backend/apps/users/admin.py`

**Step 1: Create admin.py**

Create `backend/apps/users/admin.py`:

```python
"""User admin configuration."""

from django.contrib import admin
from django.contrib.auth.admin import UserAdmin as BaseUserAdmin
from django.contrib.auth import get_user_model

User = get_user_model()


@admin.register(User)
class UserAdmin(BaseUserAdmin):
    """Admin for custom User model."""

    list_display = ["email", "first_name", "last_name", "is_staff", "date_joined"]
    list_filter = ["is_staff", "is_superuser", "is_active"]
    search_fields = ["email", "first_name", "last_name"]
    ordering = ["-date_joined"]

    fieldsets = (
        (None, {"fields": ("email", "password")}),
        ("Personal info", {"fields": ("first_name", "last_name", "avatar_url")}),
        ("OAuth", {"fields": ("google_id", "discord_id")}),
        ("Permissions", {"fields": ("is_active", "is_staff", "is_superuser", "groups", "user_permissions")}),
        ("Important dates", {"fields": ("last_login", "date_joined")}),
    )

    add_fieldsets = (
        (None, {
            "classes": ("wide",),
            "fields": ("email", "password1", "password2"),
        }),
    )
```

**Step 2: Commit**

```bash
git add backend/apps/users/admin.py
git commit -m "feat: add user admin configuration"
```

---

## Part 4: Social Authentication

### Task 16: Create social auth views

**Files:**
- Create: `backend/apps/users/social_auth.py`

**Step 1: Create social_auth.py**

Create `backend/apps/users/social_auth.py`:

```python
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

        return Response({
            "auth_url": auth_url,
            "state": state,
        })


class GoogleCallbackView(APIView):
    """Exchange Google auth code for JWT tokens."""

    permission_classes = [AllowAny]

    def post(self, request):
        code = request.data.get("code")
        redirect_uri = request.data.get("redirect_uri")

        if not code or not redirect_uri:
            return Response(
                {"detail": "code and redirect_uri are required"},
                status=status.HTTP_400_BAD_REQUEST
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
                {"detail": "Failed to exchange code for tokens"},
                status=status.HTTP_400_BAD_REQUEST
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
                {"detail": "Failed to get user info"},
                status=status.HTTP_400_BAD_REQUEST
            )

        user_info = user_response.json()
        email = user_info.get("email")
        google_id = user_info.get("id")

        if not email:
            return Response(
                {"detail": "Email not provided by Google"},
                status=status.HTTP_400_BAD_REQUEST
            )

        # Get or create user
        user, created = User.objects.get_or_create(
            email=email,
            defaults={
                "google_id": google_id,
                "first_name": user_info.get("given_name", ""),
                "last_name": user_info.get("family_name", ""),
                "avatar_url": user_info.get("picture", ""),
            }
        )

        # Update Google ID if not set
        if not user.google_id:
            user.google_id = google_id
            user.save(update_fields=["google_id"])

        # Generate JWT tokens
        jwt_tokens = get_tokens_for_user(user)

        return Response({
            **jwt_tokens,
            "user": {
                "id": user.id,
                "email": user.email,
                "first_name": user.first_name,
                "last_name": user.last_name,
                "avatar_url": user.avatar_url,
            },
            "created": created,
        })


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

        return Response({
            "auth_url": auth_url,
            "state": state,
        })


class DiscordCallbackView(APIView):
    """Exchange Discord auth code for JWT tokens."""

    permission_classes = [AllowAny]

    def post(self, request):
        code = request.data.get("code")
        redirect_uri = request.data.get("redirect_uri")

        if not code or not redirect_uri:
            return Response(
                {"detail": "code and redirect_uri are required"},
                status=status.HTTP_400_BAD_REQUEST
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
                {"detail": "Failed to exchange code for tokens"},
                status=status.HTTP_400_BAD_REQUEST
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
                {"detail": "Failed to get user info"},
                status=status.HTTP_400_BAD_REQUEST
            )

        user_info = user_response.json()
        email = user_info.get("email")
        discord_id = user_info.get("id")
        username = user_info.get("username", "")
        avatar_hash = user_info.get("avatar")

        if not email:
            return Response(
                {"detail": "Email not provided by Discord. Make sure to grant email permission."},
                status=status.HTTP_400_BAD_REQUEST
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
            }
        )

        # Update Discord ID if not set
        if not user.discord_id:
            user.discord_id = discord_id
            user.save(update_fields=["discord_id"])

        # Generate JWT tokens
        jwt_tokens = get_tokens_for_user(user)

        return Response({
            **jwt_tokens,
            "user": {
                "id": user.id,
                "email": user.email,
                "first_name": user.first_name,
                "last_name": user.last_name,
                "avatar_url": user.avatar_url,
            },
            "created": created,
        })
```

**Step 2: Commit**

```bash
git add backend/apps/users/social_auth.py
git commit -m "feat: add Google and Discord OAuth views"
```

---

### Task 17: Add requests dependency

**Files:**
- Modify: `backend/pyproject.toml`

**Step 1: Add requests to dependencies**

Add `requests>=2.32` to the dependencies list in `backend/pyproject.toml`:

```toml
dependencies = [
    "django>=5.1",
    "djangorestframework>=3.15",
    "djangorestframework-simplejwt>=5.3",
    "django-allauth[socialaccount]>=65.0",
    "channels>=4.0",
    "channels-redis>=4.2",
    "psycopg[binary]>=3.2",
    "boto3>=1.35",
    "python-dotenv>=1.0",
    "django-cors-headers>=4.0",
    "requests>=2.32",
]
```

**Step 2: Sync dependencies**

```bash
cd backend && uv sync
```

**Step 3: Commit**

```bash
git add backend/pyproject.toml backend/uv.lock
git commit -m "chore: add requests dependency for OAuth"
```

---

### Task 18: Create and run migrations

**Files:**
- Create: `backend/apps/users/migrations/`

**Step 1: Create migrations**

```bash
cd backend && uv run python manage.py makemigrations users
```

Expected: Creates `0001_initial.py` migration

**Step 2: Run migrations**

```bash
cd backend && uv run python manage.py migrate
```

Expected: Applies all migrations successfully

**Step 3: Verify**

```bash
cd backend && uv run python manage.py showmigrations users
```

Expected: Shows `[X] 0001_initial`

**Step 4: Commit**

```bash
git add backend/apps/users/migrations/
git commit -m "feat: add user migrations"
```

---

### Task 19: Test Django setup

**Files:**
- None (verification only)

**Step 1: Check for issues**

```bash
cd backend && uv run python manage.py check
```

Expected: "System check identified no issues"

**Step 2: Create superuser (interactive, optional)**

```bash
cd backend && uv run python manage.py createsuperuser --email admin@example.com
```

**Step 3: Run development server**

```bash
cd backend && uv run python manage.py runserver
```

Expected: Server starts at http://127.0.0.1:8000/

**Step 4: Verify API endpoint**

Visit http://127.0.0.1:8000/api/auth/google/url/?redirect_uri=http://localhost

Expected: JSON response with `auth_url` and `state`

---

## Part 5: Docker Configuration

### Task 20: Create Dockerfile

**Files:**
- Create: `backend/Dockerfile`

**Step 1: Create Dockerfile**

Create `backend/Dockerfile`:

```dockerfile
FROM python:3.12-slim

# Install uv
COPY --from=ghcr.io/astral-sh/uv:latest /uv /bin/uv

# Set working directory
WORKDIR /app

# Copy dependency files
COPY pyproject.toml uv.lock ./

# Install dependencies (production only)
RUN uv sync --frozen --no-dev --extra prod

# Copy application code
COPY . .

# Collect static files
RUN uv run python manage.py collectstatic --noinput

# Expose port
EXPOSE 8000

# Default command (overridden in docker-compose)
CMD ["uv", "run", "gunicorn", "config.wsgi:application", "--bind", "0.0.0.0:8000"]
```

**Step 2: Create .dockerignore**

Create `backend/.dockerignore`:

```
.git
.gitignore
.env
.env.*
!.env.example
__pycache__
*.pyc
*.pyo
.pytest_cache
.coverage
htmlcov
.ruff_cache
db.sqlite3
static/
*.md
tests/
.venv/
```

**Step 3: Commit**

```bash
git add backend/Dockerfile backend/.dockerignore
git commit -m "feat: add Dockerfile for Django backend"
```

---

### Task 21: Create docker-compose.yml

**Files:**
- Create: `backend/docker-compose.yml`

**Step 1: Create docker-compose.yml**

Create `backend/docker-compose.yml`:

```yaml
services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/ssl:/etc/nginx/ssl:ro
      - ./static:/app/static:ro
    depends_on:
      - web
      - channels
    restart: unless-stopped

  web:
    build: .
    command: uv run gunicorn config.wsgi:application --bind 0.0.0.0:8000 --workers 3
    env_file: .env
    volumes:
      - ./static:/app/static
    depends_on:
      - db
      - redis
    restart: unless-stopped

  channels:
    build: .
    command: uv run daphne config.asgi:application --bind 0.0.0.0:8001
    env_file: .env
    depends_on:
      - db
      - redis
    restart: unless-stopped

  db:
    image: postgres:16-alpine
    volumes:
      - postgres_data:/var/lib/postgresql/data
    env_file: .env
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
```

**Step 2: Commit**

```bash
git add backend/docker-compose.yml
git commit -m "feat: add docker-compose.yml"
```

---

### Task 22: Create docker-compose.dev.yml

**Files:**
- Create: `backend/docker-compose.dev.yml`

**Step 1: Create docker-compose.dev.yml**

Create `backend/docker-compose.dev.yml`:

```yaml
# Development compose - just DB and Redis
# Run Django locally with: uv run python manage.py runserver

services:
  db:
    image: postgres:16-alpine
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: voiceboard
      POSTGRES_USER: voiceboard
      POSTGRES_PASSWORD: voiceboard

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

**Step 2: Commit**

```bash
git add backend/docker-compose.dev.yml
git commit -m "feat: add docker-compose.dev.yml for local development"
```

---

### Task 23: Create nginx configuration

**Files:**
- Create: `backend/nginx/nginx.conf`

**Step 1: Create nginx directory**

```bash
mkdir -p backend/nginx/ssl
```

**Step 2: Create nginx.conf**

Create `backend/nginx/nginx.conf`:

```nginx
events {
    worker_connections 1024;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    # Logging
    access_log /var/log/nginx/access.log;
    error_log /var/log/nginx/error.log;

    # Gzip
    gzip on;
    gzip_types text/plain text/css application/json application/javascript;

    # Upstreams
    upstream web {
        server web:8000;
    }

    upstream channels {
        server channels:8001;
    }

    # HTTP -> HTTPS redirect
    server {
        listen 80;
        server_name voiceboard.cloud;
        return 301 https://$server_name$request_uri;
    }

    # HTTPS server
    server {
        listen 443 ssl;
        server_name voiceboard.cloud;

        # Cloudflare origin certificates
        ssl_certificate /etc/nginx/ssl/origin.pem;
        ssl_certificate_key /etc/nginx/ssl/origin-key.pem;

        # SSL settings
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
        ssl_prefer_server_ciphers off;

        # Static files
        location /static/ {
            alias /app/static/;
            expires 30d;
            add_header Cache-Control "public, immutable";
        }

        # WebSocket
        location /ws/ {
            proxy_pass http://channels;
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            proxy_read_timeout 86400;
        }

        # API
        location / {
            proxy_pass http://web;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
```

**Step 3: Add .gitkeep for ssl directory**

```bash
touch backend/nginx/ssl/.gitkeep
```

**Step 4: Commit**

```bash
git add backend/nginx/
git commit -m "feat: add nginx configuration"
```

---

## Part 6: CI/CD Pipeline

### Task 24: Create GitHub Actions workflow

**Files:**
- Create: `.github/workflows/backend.yml`

**Step 1: Create workflow file**

Create `.github/workflows/backend.yml`:

```yaml
name: Backend CI/CD

on:
  push:
    branches: [main]
    paths:
      - 'backend/**'
      - '.github/workflows/backend.yml'
  pull_request:
    branches: [main]
    paths:
      - 'backend/**'
      - '.github/workflows/backend.yml'

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}-backend

defaults:
  run:
    working-directory: backend

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install uv
        uses: astral-sh/setup-uv@v4

      - name: Install dependencies
        run: uv sync --frozen --extra dev

      - name: Run ruff check
        run: uv run ruff check .

      - name: Run ruff format check
        run: uv run ruff format --check .

  test:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:16-alpine
        env:
          POSTGRES_DB: test_voiceboard
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v4

      - name: Install uv
        uses: astral-sh/setup-uv@v4

      - name: Install dependencies
        run: uv sync --frozen --extra dev

      - name: Run tests
        run: uv run pytest -v
        env:
          DJANGO_SETTINGS_MODULE: config.settings.development
          POSTGRES_DB: test_voiceboard
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_HOST: localhost

  build:
    needs: [lint, test]
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - uses: actions/checkout@v4

      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=sha
            type=raw,value=latest

      - name: Build and push Docker image
        uses: docker/build-push-action@v5
        with:
          context: ./backend
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}

  deploy:
    needs: build
    runs-on: ubuntu-latest
    environment: production

    steps:
      - name: Deploy to server
        uses: appleboy/ssh-action@v1
        with:
          host: ${{ secrets.SERVER_HOST }}
          username: ${{ secrets.SERVER_USER }}
          key: ${{ secrets.SERVER_SSH_KEY }}
          script: |
            cd /opt/voiceboard
            docker compose pull
            docker compose up -d
            docker image prune -f
```

**Step 2: Commit**

```bash
git add .github/workflows/backend.yml
git commit -m "feat: add backend CI/CD workflow"
```

---

### Task 25: Create pytest configuration

**Files:**
- Create: `backend/pytest.ini`
- Create: `backend/conftest.py`

**Step 1: Create pytest.ini**

Create `backend/pytest.ini`:

```ini
[pytest]
DJANGO_SETTINGS_MODULE = config.settings.development
python_files = tests.py test_*.py *_test.py
addopts = -v --tb=short
```

**Step 2: Create conftest.py**

Create `backend/conftest.py`:

```python
"""Pytest configuration and fixtures."""

import pytest
from django.contrib.auth import get_user_model

User = get_user_model()


@pytest.fixture
def user(db):
    """Create a test user."""
    return User.objects.create_user(
        email="test@example.com",
        password="testpass123",
        first_name="Test",
        last_name="User",
    )


@pytest.fixture
def api_client():
    """Create a DRF API client."""
    from rest_framework.test import APIClient
    return APIClient()


@pytest.fixture
def authenticated_client(api_client, user):
    """Create an authenticated API client."""
    api_client.force_authenticate(user=user)
    return api_client
```

**Step 3: Commit**

```bash
git add backend/pytest.ini backend/conftest.py
git commit -m "feat: add pytest configuration"
```

---

### Task 26: Create basic tests

**Files:**
- Create: `backend/apps/users/tests.py`

**Step 1: Create tests.py**

Create `backend/apps/users/tests.py`:

```python
"""Tests for users app."""

import pytest
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
```

**Step 2: Run tests**

```bash
cd backend && uv run pytest -v
```

Expected: All tests pass

**Step 3: Commit**

```bash
git add backend/apps/users/tests.py
git commit -m "test: add user model and API tests"
```

---

### Task 27: Create ruff configuration

**Files:**
- Create: `backend/ruff.toml`

**Step 1: Create ruff.toml**

Create `backend/ruff.toml`:

```toml
line-length = 100
target-version = "py312"

[lint]
select = [
    "E",    # pycodestyle errors
    "W",    # pycodestyle warnings
    "F",    # pyflakes
    "I",    # isort
    "B",    # flake8-bugbear
    "C4",   # flake8-comprehensions
    "UP",   # pyupgrade
]
ignore = [
    "E501",  # line too long (handled by formatter)
]

[lint.isort]
known-first-party = ["apps", "config"]
```

**Step 2: Run ruff**

```bash
cd backend && uv run ruff check .
```

**Step 3: Fix any issues**

```bash
cd backend && uv run ruff check --fix .
```

**Step 4: Commit**

```bash
git add backend/ruff.toml
git commit -m "chore: add ruff configuration"
```

---

### Task 28: Final verification and cleanup

**Files:**
- None (verification only)

**Step 1: Verify all tests pass**

```bash
cd backend && uv run pytest -v
```

**Step 2: Verify ruff passes**

```bash
cd backend && uv run ruff check . && uv run ruff format --check .
```

**Step 3: Verify Django checks pass**

```bash
cd backend && uv run python manage.py check
```

**Step 4: Verify migrations are up to date**

```bash
cd backend && uv run python manage.py makemigrations --check --dry-run
```

Expected: "No changes detected"

**Step 5: Create final commit**

```bash
git add -A
git commit -m "feat: complete Django backend infrastructure setup"
```

---

## Summary

This plan creates:

1. **Django project** with uv package manager
2. **Split settings** (base/development/production)
3. **Custom User model** with email authentication
4. **Social OAuth** (Google + Discord) with JWT tokens
5. **Docker configuration** for production deployment
6. **CI/CD pipeline** with lint, test, build, deploy stages
7. **Pytest setup** with fixtures and initial tests

**Next steps after this plan:**
- Configure Scaleway VPS
- Set up Cloudflare DNS
- Deploy with Docker Compose
- Configure OAuth apps (Google Console, Discord Developer Portal)
