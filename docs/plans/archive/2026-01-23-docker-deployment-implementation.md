# Docker Deployment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Production-ready Docker Compose setup with Traefik, optional Let's Encrypt, and configurable embedded/external services.

**Architecture:** Traefik reverse proxy fronts Django (Gunicorn + Daphne). PostgreSQL, Redis, and MinIO are optional embedded services controlled via Docker Compose profiles. Images published to ghcr.io with CalVer tags.

**Tech Stack:** Docker Compose v2, Traefik v3, PostgreSQL 16, Redis 7, MinIO, GitHub Actions

**Design:** See `docs/plans/2026-01-23-docker-deployment-design.md`

---

## Task 1: Update .gitignore for data directory

**Files:**
- Modify: `backend/.gitignore`

**Step 1: Add data directory to gitignore**

Add to `backend/.gitignore`:
```
# Docker data volumes
data/
```

**Step 2: Verify**

Run: `cat backend/.gitignore | grep -E "^data/"`
Expected: `data/`

**Step 3: Commit**

```bash
git add backend/.gitignore
git commit -m "chore: add data/ to gitignore for Docker volumes"
```

---

## Task 2: Create docker-compose.yml with Traefik and profiles

**Files:**
- Replace: `backend/docker-compose.yml`

**Step 1: Write the complete docker-compose.yml**

Replace `backend/docker-compose.yml` with:

```yaml
services:
  # === Reverse Proxy ===
  traefik:
    image: traefik:v3.0
    command:
      - "--providers.docker=true"
      - "--providers.docker.exposedbydefault=false"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
      - "--entrypoints.web.http.redirections.entrypoint.to=websecure"
      - "--entrypoints.web.http.redirections.entrypoint.scheme=https"
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ${DATA_PATH:-./data}/traefik:/letsencrypt
    restart: unless-stopped

  # === Django REST API ===
  web:
    image: ghcr.io/${GITHUB_REPOSITORY_OWNER:-voiceboard}/voiceboard-backend:${IMAGE_TAG:-latest}
    command: uv run gunicorn config.wsgi:application --bind 0.0.0.0:8000 --workers 3
    env_file: .env
    environment:
      - DATABASE_URL=${DATABASE_URL:-postgres://${POSTGRES_USER:-voiceboard}:${POSTGRES_PASSWORD}@db:5432/${POSTGRES_DB:-voiceboard}}
      - REDIS_URL=${REDIS_URL:-redis://redis:6379}
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.web.rule=Host(`${DOMAIN}`)"
      - "traefik.http.routers.web.entrypoints=websecure"
      - "traefik.http.routers.web.tls=true"
      - "traefik.http.services.web.loadbalancer.server.port=8000"
    depends_on:
      db:
        condition: service_healthy
        required: false
      redis:
        condition: service_healthy
        required: false
      minio:
        condition: service_healthy
        required: false
    restart: unless-stopped

  # === Django WebSocket (Channels) ===
  channels:
    image: ghcr.io/${GITHUB_REPOSITORY_OWNER:-voiceboard}/voiceboard-backend:${IMAGE_TAG:-latest}
    command: uv run daphne config.asgi:application --bind 0.0.0.0:8001
    env_file: .env
    environment:
      - DATABASE_URL=${DATABASE_URL:-postgres://${POSTGRES_USER:-voiceboard}:${POSTGRES_PASSWORD}@db:5432/${POSTGRES_DB:-voiceboard}}
      - REDIS_URL=${REDIS_URL:-redis://redis:6379}
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.channels.rule=Host(`${DOMAIN}`) && PathPrefix(`/ws/`)"
      - "traefik.http.routers.channels.entrypoints=websecure"
      - "traefik.http.routers.channels.tls=true"
      - "traefik.http.services.channels.loadbalancer.server.port=8001"
    depends_on:
      db:
        condition: service_healthy
        required: false
      redis:
        condition: service_healthy
        required: false
    restart: unless-stopped

  # === PostgreSQL (embedded, optional) ===
  db:
    image: postgres:16-alpine
    profiles: ["db"]
    volumes:
      - ${DATA_PATH:-./data}/postgres:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: ${POSTGRES_DB:-voiceboard}
      POSTGRES_USER: ${POSTGRES_USER:-voiceboard}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD:?POSTGRES_PASSWORD is required}
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER:-voiceboard} -d ${POSTGRES_DB:-voiceboard}"]
      interval: 5s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  # === Redis (embedded, optional) ===
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

  # === MinIO S3 (embedded, optional) ===
  minio:
    image: minio/minio
    profiles: ["s3"]
    command: server /data --console-address ":9001"
    volumes:
      - ${DATA_PATH:-./data}/minio:/data
    environment:
      MINIO_ROOT_USER: ${AWS_ACCESS_KEY_ID:?AWS_ACCESS_KEY_ID is required}
      MINIO_ROOT_PASSWORD: ${AWS_SECRET_ACCESS_KEY:?AWS_SECRET_ACCESS_KEY is required}
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.minio-console.rule=Host(`minio.${DOMAIN}`)"
      - "traefik.http.routers.minio-console.entrypoints=websecure"
      - "traefik.http.routers.minio-console.tls=true"
      - "traefik.http.routers.minio-console.service=minio-console"
      - "traefik.http.services.minio-console.loadbalancer.server.port=9001"
    healthcheck:
      test: ["CMD", "mc", "ready", "local"]
      interval: 5s
      timeout: 5s
      retries: 5
    restart: unless-stopped
```

**Step 2: Validate syntax**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && docker compose config --quiet`
Expected: No output (success)

**Step 3: Commit**

```bash
git add backend/docker-compose.yml
git commit -m "feat(docker): rewrite docker-compose with Traefik and profiles

- Traefik v3 reverse proxy with HTTP→HTTPS redirect
- Profiles: db, redis, s3 for optional embedded services
- Health checks on all services
- Host volumes via DATA_PATH variable
- Image from ghcr.io with configurable tag"
```

---

## Task 3: Create Let's Encrypt overlay file

**Files:**
- Create: `backend/docker-compose.letsencrypt.yml`

**Step 1: Write the overlay file**

Create `backend/docker-compose.letsencrypt.yml`:

```yaml
# Let's Encrypt SSL certificates overlay
# Usage: docker compose -f docker-compose.yml -f docker-compose.letsencrypt.yml up -d

services:
  traefik:
    command:
      - "--providers.docker=true"
      - "--providers.docker.exposedbydefault=false"
      - "--entrypoints.web.address=:80"
      - "--entrypoints.websecure.address=:443"
      - "--entrypoints.web.http.redirections.entrypoint.to=websecure"
      - "--entrypoints.web.http.redirections.entrypoint.scheme=https"
      # Let's Encrypt ACME
      - "--certificatesresolvers.letsencrypt.acme.email=${LETSENCRYPT_EMAIL:?LETSENCRYPT_EMAIL is required}"
      - "--certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json"
      - "--certificatesresolvers.letsencrypt.acme.httpchallenge.entrypoint=web"

  web:
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.web.rule=Host(`${DOMAIN}`)"
      - "traefik.http.routers.web.entrypoints=websecure"
      - "traefik.http.routers.web.tls=true"
      - "traefik.http.routers.web.tls.certresolver=letsencrypt"
      - "traefik.http.services.web.loadbalancer.server.port=8000"

  channels:
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.channels.rule=Host(`${DOMAIN}`) && PathPrefix(`/ws/`)"
      - "traefik.http.routers.channels.entrypoints=websecure"
      - "traefik.http.routers.channels.tls=true"
      - "traefik.http.routers.channels.tls.certresolver=letsencrypt"
      - "traefik.http.services.channels.loadbalancer.server.port=8001"

  minio:
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.minio-console.rule=Host(`minio.${DOMAIN}`)"
      - "traefik.http.routers.minio-console.entrypoints=websecure"
      - "traefik.http.routers.minio-console.tls=true"
      - "traefik.http.routers.minio-console.tls.certresolver=letsencrypt"
      - "traefik.http.routers.minio-console.service=minio-console"
      - "traefik.http.services.minio-console.loadbalancer.server.port=9001"
```

**Step 2: Validate combined config**

Run: `cd /Users/didouye/Workspace/voiceboard/backend && docker compose -f docker-compose.yml -f docker-compose.letsencrypt.yml config --quiet`
Expected: No output (success)

**Step 3: Commit**

```bash
git add backend/docker-compose.letsencrypt.yml
git commit -m "feat(docker): add Let's Encrypt overlay

Usage: docker compose -f docker-compose.yml -f docker-compose.letsencrypt.yml up -d
Requires LETSENCRYPT_EMAIL environment variable"
```

---

## Task 4: Update .env.example

**Files:**
- Replace: `backend/.env.example`

**Step 1: Write comprehensive .env.example**

Replace `backend/.env.example` with:

```bash
# Voiceboard Backend Configuration
# Copy to .env and customize values

# =============================================================================
# DOMAIN & SSL
# =============================================================================

# Your domain (required)
DOMAIN=voiceboard.example.com

# Let's Encrypt email (only if using docker-compose.letsencrypt.yml)
# Leave empty if behind Cloudflare or other SSL termination
LETSENCRYPT_EMAIL=admin@example.com

# =============================================================================
# DJANGO
# =============================================================================

# Generate with: python -c "from django.core.management.utils import get_random_secret_key; print(get_random_secret_key())"
SECRET_KEY=change-me-to-a-random-secret-key

DEBUG=false
ALLOWED_HOSTS=${DOMAIN}

# =============================================================================
# DOCKER SETTINGS
# =============================================================================

# Data directory on host (for all persistent data)
DATA_PATH=./data

# Image settings (usually no need to change)
# GITHUB_REPOSITORY_OWNER=your-org
# IMAGE_TAG=latest

# =============================================================================
# EMBEDDED SERVICES (via Docker Compose profiles)
# =============================================================================
# Activate embedded services by setting COMPOSE_PROFILES before docker compose:
#   COMPOSE_PROFILES=db,redis,s3 docker compose up -d
#
# Or export it:
#   export COMPOSE_PROFILES=db,redis,s3
#
# Options: db, redis, s3 (comma-separated, any combination)

# --- PostgreSQL (embedded) ---
POSTGRES_DB=voiceboard
POSTGRES_USER=voiceboard
POSTGRES_PASSWORD=change-me-in-production

# --- S3/MinIO credentials (used by both embedded MinIO and external S3) ---
AWS_ACCESS_KEY_ID=minioadmin
AWS_SECRET_ACCESS_KEY=change-me-in-production
AWS_STORAGE_BUCKET_NAME=voiceboard

# =============================================================================
# EXTERNAL SERVICES (when profile not activated)
# =============================================================================
# Uncomment and configure these when NOT using embedded services

# --- External PostgreSQL (when "db" profile not in COMPOSE_PROFILES) ---
# DATABASE_URL=postgres://user:password@hostname:5432/voiceboard

# --- External Redis (when "redis" profile not in COMPOSE_PROFILES) ---
# REDIS_URL=redis://hostname:6379

# --- External S3 (when "s3" profile not in COMPOSE_PROFILES) ---
# AWS_S3_ENDPOINT_URL=https://s3.amazonaws.com
# AWS_S3_REGION_NAME=us-east-1

# =============================================================================
# EMBEDDED MINIO S3 ENDPOINT
# =============================================================================
# When using embedded MinIO (s3 profile), this is the internal endpoint
AWS_S3_ENDPOINT_URL=http://minio:9000
```

**Step 2: Commit**

```bash
git add backend/.env.example
git commit -m "docs: comprehensive .env.example with all configuration options

- Domain and SSL settings
- Django settings
- Docker data path and image settings
- Embedded services (PostgreSQL, Redis, MinIO) credentials
- External services URLs (DATABASE_URL, REDIS_URL, S3)"
```

---

## Task 5: Create GitHub Actions workflow for backend release

**Files:**
- Create: `.github/workflows/backend-release.yml`

**Step 1: Write the workflow**

Create `.github/workflows/backend-release.yml`:

```yaml
name: Backend Release

on:
  push:
    branches: [main]
    paths:
      - 'backend/**'
      - '.github/workflows/backend-release.yml'

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository_owner }}/voiceboard-backend

jobs:
  build-and-push:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Generate CalVer tag
        id: version
        run: echo "tag=$(date -u +'%Y%m%d.%H%M')" >> $GITHUB_OUTPUT

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata for Docker
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=raw,value=latest
            type=raw,value=${{ steps.version.outputs.tag }}

      - name: Build and push Docker image
        uses: docker/build-push-action@v6
        with:
          context: ./backend
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

**Step 2: Commit**

```bash
git add .github/workflows/backend-release.yml
git commit -m "ci: add backend Docker image release workflow

- Triggers on push to main when backend/ changes
- Builds and pushes to ghcr.io
- Tags: latest + CalVer (YYYYMMDD.HHMM)
- Uses GitHub Actions cache for faster builds"
```

---

## Task 6: Add deployment documentation to backend README

**Files:**
- Modify: `backend/README.md` (or create if doesn't exist)

**Step 1: Check if README exists**

Run: `ls -la /Users/didouye/Workspace/voiceboard/backend/README.md`

**Step 2: Add/update deployment section**

Add to `backend/README.md`:

```markdown
## Deployment

### Quick Start (All Embedded Services)

```bash
# Copy and configure environment
cp .env.example .env
nano .env  # Set DOMAIN, POSTGRES_PASSWORD, SECRET_KEY, etc.

# Start with all embedded services (PostgreSQL, Redis, MinIO)
COMPOSE_PROFILES=db,redis,s3 docker compose up -d

# Run migrations
docker compose exec web uv run python manage.py migrate

# Create superuser
docker compose exec web uv run python manage.py createsuperuser
```

### With Let's Encrypt SSL

```bash
# Requires LETSENCRYPT_EMAIL in .env
COMPOSE_PROFILES=db,redis,s3 docker compose -f docker-compose.yml -f docker-compose.letsencrypt.yml up -d
```

### Using External Services

Configure external service URLs in `.env`, then omit their profiles:

```bash
# Example: External PostgreSQL, embedded Redis and MinIO
# In .env: DATABASE_URL=postgres://user:pass@external-host:5432/db
COMPOSE_PROFILES=redis,s3 docker compose up -d

# Example: All external services
# In .env: DATABASE_URL=..., REDIS_URL=..., AWS_S3_ENDPOINT_URL=...
docker compose up -d
```

### Service Profiles

| Profile | Service    | When to use                          |
|---------|------------|--------------------------------------|
| `db`    | PostgreSQL | No external PostgreSQL available     |
| `redis` | Redis      | No external Redis available          |
| `s3`    | MinIO      | No external S3-compatible storage    |

### Data Persistence

All data is stored in `${DATA_PATH:-./data}/` on the host:

- `data/postgres/` - PostgreSQL database files
- `data/redis/` - Redis AOF persistence
- `data/minio/` - MinIO object storage
- `data/traefik/` - Let's Encrypt certificates

### Updating

```bash
# Pull latest image
docker compose pull

# Restart with new image
docker compose up -d

# Run migrations if needed
docker compose exec web uv run python manage.py migrate
```
```

**Step 3: Commit**

```bash
git add backend/README.md
git commit -m "docs: add deployment instructions to backend README

- Quick start with embedded services
- Let's Encrypt SSL setup
- External services configuration
- Service profiles explanation
- Data persistence locations
- Update procedure"
```

---

## Task 7: Update ROADMAP.md

**Files:**
- Modify: `ROADMAP.md`

**Step 1: Mark infrastructure task as done**

In `ROADMAP.md`, under Phase 4, change:

```markdown
- [ ] **Infrastructure**
```

to:

```markdown
- [x] **Infrastructure**
```

**Step 2: Commit**

```bash
git add ROADMAP.md
git commit -m "docs: mark Phase 4 Infrastructure as complete"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Add data/ to gitignore | `backend/.gitignore` |
| 2 | Rewrite docker-compose.yml | `backend/docker-compose.yml` |
| 3 | Create Let's Encrypt overlay | `backend/docker-compose.letsencrypt.yml` |
| 4 | Update .env.example | `backend/.env.example` |
| 5 | Create CI/CD workflow | `.github/workflows/backend-release.yml` |
| 6 | Add deployment docs | `backend/README.md` |
| 7 | Update roadmap | `ROADMAP.md` |
