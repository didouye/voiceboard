# Cloud Infrastructure Design

> **Date**: 2026-01-12
> **Status**: Approved
> **Phase**: 4 - Cloud & Collaboration

## Overview

Infrastructure foundation for Voiceboard cloud features: Django backend with REST API, WebSocket support, social authentication, and S3 storage.

## Architecture

```
                              +---------------------+
                              |     Cloudflare      |
                              |  (SSL + proxy)      |
                              |  voiceboard.cloud   |
                              +---------+-----------+
                                        | HTTPS (origin cert)
                                        v
+-----------------------------------------------------------------------+
|                     Scaleway VPS (Docker Compose)                     |
|  +-----------+  +-------------+  +-------------+                      |
|  |   Nginx   |  |   Django    |  |   Django    |                      |
|  | (reverse  |--|   (web)     |  |  (channels) |                      |
|  |  proxy)   |  |  Gunicorn   |  |   Daphne    |                      |
|  +-----------+  +------+------+  +------+------+                      |
|                        |                |                             |
|                 +------+----------------+------+                      |
|                 |                              |                      |
|                 |  +-------------+  +---------+-------+               |
|                 |  | PostgreSQL  |  |     Redis       |               |
|                 |  |   (db)      |  | (cache+pubsub)  |               |
|                 |  +-------------+  +-----------------+               |
|                 |                                                     |
+-----------------------------------------------------------------------+
                  | API calls (upload, metadata)
                  v
       +---------------------+      +---------------------+
       | Scaleway Object     |------| Scaleway Edge       |
       | Storage (S3)        |      | Services (CDN)      |
       +---------------------+      +---------+-----------+
                                              |
                                   media.voiceboard.cloud
                                              v
                                       Desktop/Mobile
```

**Components:**
- **Cloudflare**: SSL termination, DDoS protection, proxy
- **Nginx**: Reverse proxy, static files, WebSocket routing
- **Django (web)**: REST API via Gunicorn (sync requests)
- **Django (channels)**: WebSocket via Daphne (async)
- **PostgreSQL**: Primary database
- **Redis**: Cache, session store, Channels layer
- **Scaleway Object Storage**: Audio file storage
- **Scaleway Edge Services**: CDN for direct media downloads

**Domains:**
- `voiceboard.cloud` - API (Django)
- `media.voiceboard.cloud` - Edge CDN (Scaleway Edge Services)

## Django Project Structure

```
voiceboard-backend/
├── config/                     # Project configuration
│   ├── settings/
│   │   ├── base.py            # Shared settings
│   │   ├── development.py     # Local dev (DEBUG=True)
│   │   └── production.py      # Production settings
│   ├── urls.py                # Root URL routing
│   ├── asgi.py                # ASGI entry (WebSocket)
│   └── wsgi.py                # WSGI entry (HTTP)
│
├── apps/
│   ├── users/                 # User management
│   │   ├── models.py          # User model
│   │   ├── serializers.py     # DRF serializers
│   │   ├── views.py           # Registration, profile
│   │   └── urls.py
│   │
│   ├── soundboards/           # Soundboard sync (future)
│   │   ├── models.py          # Soundboard, Sound, Folder
│   │   ├── serializers.py
│   │   ├── views.py           # CRUD + sync
│   │   └── urls.py
│   │
│   └── core/                  # Shared utilities
│       ├── storage.py         # Scaleway S3 backend
│       └── permissions.py     # Custom DRF permissions
│
├── pyproject.toml             # Dependencies + project metadata
├── uv.lock                    # Locked dependencies
├── manage.py
└── Dockerfile
```

## Dependencies (pyproject.toml)

```toml
[project]
name = "voiceboard-backend"
version = "0.1.0"
requires-python = ">=3.12"
dependencies = [
    "django>=5.1",
    "djangorestframework>=3.15",
    "djangorestframework-simplejwt>=5.3",
    "django-allauth>=65.0",
    "channels>=4.0",
    "channels-redis>=4.2",
    "psycopg[binary]>=3.2",
    "boto3>=1.35",
    "python-dotenv>=1.0",
]

[project.optional-dependencies]
dev = ["django-debug-toolbar", "ruff"]
prod = ["gunicorn", "daphne", "sentry-sdk"]
```

## Docker Configuration

### Dockerfile

```dockerfile
FROM python:3.12-slim
COPY --from=ghcr.io/astral-sh/uv:latest /uv /bin/uv
WORKDIR /app
COPY pyproject.toml uv.lock ./
RUN uv sync --frozen --no-dev
COPY . .
```

### docker-compose.yml

```yaml
services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./nginx/origin.pem:/etc/nginx/ssl/origin.pem:ro
      - ./nginx/origin-key.pem:/etc/nginx/ssl/origin-key.pem:ro
      - ./static:/app/static:ro
    depends_on:
      - web
      - channels

  web:
    image: ghcr.io/your-org/voiceboard-backend:latest
    command: gunicorn config.wsgi:application --bind 0.0.0.0:8000 --workers 3
    env_file: .env
    volumes:
      - ./static:/app/static
    depends_on:
      - db
      - redis

  channels:
    image: ghcr.io/your-org/voiceboard-backend:latest
    command: daphne config.asgi:application --bind 0.0.0.0:8001
    env_file: .env
    depends_on:
      - db
      - redis

  db:
    image: postgres:16-alpine
    volumes:
      - ./data/postgres:/var/lib/postgresql/data
    env_file: .env

  redis:
    image: redis:7-alpine
    volumes:
      - ./data/redis:/data
```

### Host Directory Structure

```
/opt/voiceboard/
├── docker-compose.yml
├── .env                        # Secrets (not in git)
├── nginx/
│   ├── nginx.conf
│   ├── origin.pem              # Cloudflare origin cert
│   └── origin-key.pem
├── static/                     # Django collectstatic output
├── data/
│   ├── postgres/               # Database files
│   └── redis/                  # Redis persistence
```

## Nginx Configuration

```nginx
upstream web {
    server web:8000;
}

upstream channels {
    server channels:8001;
}

server {
    listen 80;
    server_name voiceboard.cloud;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl;
    server_name voiceboard.cloud;

    ssl_certificate /etc/nginx/ssl/origin.pem;
    ssl_certificate_key /etc/nginx/ssl/origin-key.pem;

    location /static/ {
        alias /app/static/;
        expires 30d;
        add_header Cache-Control "public, immutable";
    }

    location /ws/ {
        proxy_pass http://channels;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_read_timeout 86400;
    }

    location / {
        proxy_pass http://web;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## CI/CD Pipeline

```yaml
# .github/workflows/deploy.yml
name: Build & Deploy

on:
  push:
    branches: [main]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}-backend

jobs:
  build:
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

      - name: Build and push Docker image
        uses: docker/build-push-action@v5
        with:
          context: ./backend
          push: true
          tags: |
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:latest
            ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }}

  deploy:
    needs: build
    runs-on: ubuntu-latest

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

**GitHub Secrets required:**
- `SERVER_HOST` - Scaleway server IP/hostname
- `SERVER_USER` - SSH user
- `SERVER_SSH_KEY` - Private SSH key

## Authentication

### Flow (Social Auth + JWT)

```
Desktop (Tauri)                              Django Backend
      |                                             |
      |  1. GET /api/auth/google/url/               |
      | ------------------------------------------> |
      |     {auth_url}                              |
      | <------------------------------------------ |
      |                                             |
      |  2. Open browser -> Google OAuth            |
      | -------------------------------------------> Google/Discord
      |                                             |
      |  3. User authorizes, redirect with code     |
      | <------------------------------------------ |
      |                                             |
      |  4. POST /api/auth/google/callback/         |
      |     {code}                                  |
      | ------------------------------------------> |
      |                                             | Django exchanges
      |                                             | code for user info
      |     {access_token, refresh_token, user}     |
      | <------------------------------------------ |
      |                                             |
      |  5. API calls with JWT                      |
      |     Authorization: Bearer <access_token>    |
      | ------------------------------------------> |
```

### Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/auth/google/url/` | Get Google OAuth URL |
| POST | `/api/auth/google/callback/` | Exchange code for JWT |
| GET | `/api/auth/discord/url/` | Get Discord OAuth URL |
| POST | `/api/auth/discord/callback/` | Exchange code for JWT |
| POST | `/api/auth/refresh/` | Refresh access token |
| POST | `/api/auth/logout/` | Revoke token (blacklist) |
| GET | `/api/auth/me/` | Current user profile |

### Token Strategy

- **Access token**: 15 minutes lifetime
- **Refresh token**: 30 days lifetime
- **Revocation**: `rest_framework_simplejwt.token_blacklist`
- **Storage (desktop)**: OS keychain via Tauri secure storage

## Models

### User Model

```python
# apps/users/models.py
from django.contrib.auth.models import AbstractUser
from django.db import models

class User(AbstractUser):
    """Custom user - email-based, no username."""
    username = None
    email = models.EmailField(unique=True)
    avatar_url = models.URLField(blank=True)

    # OAuth info
    google_id = models.CharField(max_length=255, blank=True)
    discord_id = models.CharField(max_length=255, blank=True)

    USERNAME_FIELD = "email"
    REQUIRED_FIELDS = []

    class Meta:
        db_table = "users"
```

## Environment Variables

```bash
# .env

# Django
DJANGO_SECRET_KEY=your-secret-key-here
DJANGO_DEBUG=false
DJANGO_ALLOWED_HOSTS=voiceboard.cloud

# Database
POSTGRES_DB=voiceboard
POSTGRES_USER=voiceboard
POSTGRES_PASSWORD=secure-db-password

# Redis
REDIS_URL=redis://redis:6379/0

# Scaleway Object Storage
SCW_ACCESS_KEY=your-access-key
SCW_SECRET_KEY=your-secret-key
SCW_BUCKET_NAME=voiceboard-media
SCW_REGION=fr-par
SCW_ENDPOINT_URL=https://s3.fr-par.scw.cloud
MEDIA_URL=https://media.voiceboard.cloud/

# OAuth - Google
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret

# OAuth - Discord
DISCORD_CLIENT_ID=your-discord-client-id
DISCORD_CLIENT_SECRET=your-discord-client-secret

# JWT
JWT_ACCESS_TOKEN_LIFETIME=15
JWT_REFRESH_TOKEN_LIFETIME=43200

# Sentry (optional)
SENTRY_DSN=https://xxx@sentry.io/xxx
```

### Loading with python-dotenv

```python
# config/settings/base.py
from pathlib import Path
from dotenv import load_dotenv
import os

BASE_DIR = Path(__file__).resolve().parent.parent.parent
load_dotenv(BASE_DIR / ".env")

SECRET_KEY = os.environ["DJANGO_SECRET_KEY"]
DEBUG = os.getenv("DJANGO_DEBUG", "false").lower() == "true"
ALLOWED_HOSTS = os.getenv("DJANGO_ALLOWED_HOSTS", "").split(",")
```

## Implementation Notes

### Phase 4 Roadmap

This infrastructure enables all Phase 4 features:

1. **Infrastructure** (this design) - Foundation
2. **User Management** - Accounts, profiles, billing (Stripe)
3. **Teams** - Team creation, invitations, shared soundboards
4. **Synchronization** - Soundboard sync between devices
5. **Remote Live Logging** - See `2026-01-04-remote-live-logging-design.md`
6. **Remote Control** - See `2025-12-26-remote-control-design.md`

### Next Steps

1. Set up Django project with uv
2. Configure Docker Compose locally
3. Implement social auth endpoints
4. Deploy to Scaleway VPS
5. Configure Cloudflare DNS and SSL
