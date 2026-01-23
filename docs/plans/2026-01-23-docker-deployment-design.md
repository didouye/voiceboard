# Docker Deployment Design

> **Date:** 2026-01-23
> **Status:** Approved

## Overview

Production deployment setup using Docker Compose with:
- Traefik reverse proxy with optional Let's Encrypt
- Embedded or external PostgreSQL, Redis, MinIO (S3)
- Images published to GitHub Container Registry (ghcr.io)
- Host volumes for data persistence

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Internet                              │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│  Traefik (reverse proxy)                                    │
│  - Let's Encrypt auto if LETSENCRYPT_EMAIL defined          │
│  - Otherwise HTTP only (for Cloudflare proxy)               │
└─────────────┬───────────────────────┬───────────────────────┘
              │                       │
              ▼                       ▼
┌─────────────────────┐   ┌─────────────────────┐
│  web (Gunicorn)     │   │  channels (Daphne)  │
│  REST API Django    │   │  WebSocket Django   │
└─────────┬───────────┘   └─────────┬───────────┘
          │                         │
          └───────────┬─────────────┘
                      │
        ┌─────────────┼─────────────┐
        ▼             ▼             ▼
┌───────────┐  ┌───────────┐  ┌───────────┐
│ PostgreSQL│  │   Redis   │  │   MinIO   │
│ (profile  │  │ (profile  │  │ (profile  │
│   "db")   │  │  "redis") │  │   "s3")   │
└───────────┘  └───────────┘  └───────────┘
```

Each infrastructure service can be:
- **Embedded** (profile activated) → local container
- **External** (profile not activated) → configured via environment URL

## File Structure

```
backend/
├── Dockerfile                    # Existing, unchanged
├── docker-compose.yml            # Production with all services
├── docker-compose.letsencrypt.yml # Let's Encrypt overlay
├── docker-compose.dev.yml        # Local dev (unchanged)
├── .env.example                  # Configuration template
└── data/                         # Git-ignored
    ├── postgres/
    ├── redis/
    ├── minio/
    └── traefik/                  # Let's Encrypt certificates
```

## Docker Compose Configuration

### Main Services (docker-compose.yml)

```yaml
services:
  traefik:
    image: traefik:v3.0
    command:
      - "--providers.docker=true"
      - "--providers.docker.exposedbydefault=false"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
      - "--entrypoints.web.http.redirections.entrypoint.to=websecure"
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ${DATA_PATH:-./data}/traefik:/letsencrypt
    restart: unless-stopped

  web:
    image: ghcr.io/OWNER/voiceboard-backend:latest
    command: uv run gunicorn config.wsgi:application --bind 0.0.0.0:8000
    env_file: .env
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.web.rule=Host(`${DOMAIN}`)"
      - "traefik.http.routers.web.entrypoints=websecure"
      - "traefik.http.services.web.loadbalancer.server.port=8000"
    depends_on:
      db:
        condition: service_healthy
        required: false
    restart: unless-stopped

  channels:
    image: ghcr.io/OWNER/voiceboard-backend:latest
    command: uv run daphne config.asgi:application --bind 0.0.0.0:8001
    env_file: .env
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.channels.rule=Host(`${DOMAIN}`) && PathPrefix(`/ws/`)"
      - "traefik.http.services.channels.loadbalancer.server.port=8001"
    restart: unless-stopped
```

### Embedded Services (with profiles)

```yaml
  db:
    image: postgres:16-alpine
    profiles: ["db"]
    volumes:
      - ${DATA_PATH:-./data}/postgres:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: ${POSTGRES_DB:-voiceboard}
      POSTGRES_USER: ${POSTGRES_USER:-voiceboard}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:?POSTGRES_PASSWORD required}
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER:-voiceboard}"]
      interval: 5s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    profiles: ["redis"]
    volumes:
      - ${DATA_PATH:-./data}/redis:/data
    command: redis-server --appendonly yes
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  minio:
    image: minio/minio
    profiles: ["s3"]
    command: server /data --console-address ":9001"
    volumes:
      - ${DATA_PATH:-./data}/minio:/data
    environment:
      MINIO_ROOT_USER: ${AWS_ACCESS_KEY_ID:?AWS_ACCESS_KEY_ID required}
      MINIO_ROOT_PASSWORD: ${AWS_SECRET_ACCESS_KEY:?AWS_SECRET_ACCESS_KEY required}
    healthcheck:
      test: ["CMD", "mc", "ready", "local"]
      interval: 5s
      timeout: 5s
      retries: 5
    restart: unless-stopped
```

### Let's Encrypt Overlay (docker-compose.letsencrypt.yml)

```yaml
services:
  traefik:
    command:
      - "--providers.docker=true"
      - "--providers.docker.exposedbydefault=false"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
      - "--entrypoints.web.http.redirections.entrypoint.to=websecure"
      - "--certificatesresolvers.letsencrypt.acme.email=${LETSENCRYPT_EMAIL}"
      - "--certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json"
      - "--certificatesresolvers.letsencrypt.acme.httpchallenge.entrypoint=web"

  web:
    labels:
      - "traefik.http.routers.web.tls.certresolver=letsencrypt"

  channels:
    labels:
      - "traefik.http.routers.channels.tls.certresolver=letsencrypt"
```

## Environment Variables (.env.example)

```bash
# === Domain ===
DOMAIN=voiceboard.example.com

# === Django ===
SECRET_KEY=your-secret-key-here
DEBUG=false
ALLOWED_HOSTS=${DOMAIN}

# === Let's Encrypt (leave empty if behind Cloudflare) ===
LETSENCRYPT_EMAIL=admin@example.com

# === Data path on host ===
DATA_PATH=./data

# === Embedded services profiles (comma-separated) ===
# Options: db,redis,s3
# Set in shell: COMPOSE_PROFILES=db,redis,s3 docker compose up -d
# Or uncomment below to set as default:
# COMPOSE_PROFILES=db,redis,s3

# === PostgreSQL ===
# Embedded (when "db" profile active):
POSTGRES_DB=voiceboard
POSTGRES_USER=voiceboard
POSTGRES_PASSWORD=change-me-in-production

# External (when "db" profile not active):
# DATABASE_URL=postgres://user:pass@host:5432/voiceboard

# === Redis ===
# Embedded uses default settings
# External (when "redis" profile not active):
# REDIS_URL=redis://host:6379

# === S3 Storage ===
AWS_ACCESS_KEY_ID=minioadmin
AWS_SECRET_ACCESS_KEY=minioadmin
AWS_STORAGE_BUCKET_NAME=voiceboard

# Embedded MinIO (when "s3" profile active):
AWS_S3_ENDPOINT_URL=http://minio:9000

# External S3 (when "s3" profile not active):
# AWS_S3_ENDPOINT_URL=https://s3.amazonaws.com
```

## CI/CD: GitHub Actions

`.github/workflows/backend-release.yml`:

```yaml
name: Backend Release

on:
  push:
    branches: [main]
    paths:
      - 'backend/**'
      - '.github/workflows/backend-release.yml'

jobs:
  build-and-push:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - uses: actions/checkout@v4

      - name: Generate CalVer tag
        id: version
        run: echo "tag=$(date +'%Y%m%d.%H%M')" >> $GITHUB_OUTPUT

      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: ./backend
          push: true
          tags: |
            ghcr.io/${{ github.repository_owner }}/voiceboard-backend:latest
            ghcr.io/${{ github.repository_owner }}/voiceboard-backend:${{ steps.version.outputs.tag }}
```

## Usage

### Quick Start (all embedded, behind Cloudflare)

```bash
cd backend
cp .env.example .env
nano .env  # Configure DOMAIN, POSTGRES_PASSWORD, SECRET_KEY, etc.

COMPOSE_PROFILES=db,redis,s3 docker compose up -d
```

### With Let's Encrypt

```bash
COMPOSE_PROFILES=db,redis,s3 docker compose -f docker-compose.yml -f docker-compose.letsencrypt.yml up -d
```

### Mixed: External PostgreSQL + Embedded Redis/S3

```bash
# In .env, set DATABASE_URL to external PostgreSQL
COMPOSE_PROFILES=redis,s3 docker compose up -d
```

### All External Services

```bash
# In .env, set DATABASE_URL, REDIS_URL, AWS_S3_ENDPOINT_URL
docker compose up -d
```

## Image Registry

- **Registry:** GitHub Container Registry (ghcr.io)
- **Image:** `ghcr.io/OWNER/voiceboard-backend`
- **Tags:**
  - `latest` - always points to most recent build
  - `YYYYMMDD.HHMM` - CalVer timestamp (e.g., `20260123.1542`)

## Data Persistence

All data stored in `${DATA_PATH:-./data}/` on host:

| Service    | Host Path                | Container Path               |
|------------|--------------------------|------------------------------|
| PostgreSQL | `./data/postgres/`       | `/var/lib/postgresql/data`   |
| Redis      | `./data/redis/`          | `/data`                      |
| MinIO      | `./data/minio/`          | `/data`                      |
| Traefik    | `./data/traefik/`        | `/letsencrypt`               |

## Health Checks

All embedded services include health checks:
- **PostgreSQL:** `pg_isready`
- **Redis:** `redis-cli ping`
- **MinIO:** `mc ready local`

The `web` service waits for `db` health check (when profile active) before starting.
