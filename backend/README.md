# Voiceboard Backend

Django backend for Voiceboard application.

## Server Deployment

### Prerequisites

- Linux server (Ubuntu 22.04+ recommended)
- Docker Engine 24+ and Docker Compose v2+
- Domain name pointing to your server
- Ports 80 and 443 open

### Quick Deploy

**1. Create a directory and the docker-compose.yml file:**

```bash
mkdir -p /opt/voiceboard && cd /opt/voiceboard
```

**2. Create `docker-compose.yml` with this content:**

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
      - "--entrypoints.web.http.redirections.entrypoint.scheme=https"
      # Let's Encrypt (comment out if behind Cloudflare)
      - "--certificatesresolvers.letsencrypt.acme.email=${LETSENCRYPT_EMAIL}"
      - "--certificatesresolvers.letsencrypt.acme.storage=/letsencrypt/acme.json"
      - "--certificatesresolvers.letsencrypt.acme.httpchallenge.entrypoint=web"
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock:ro
      - ${DATA_PATH:-./data}/traefik:/letsencrypt
    restart: unless-stopped

  web:
    image: ghcr.io/didouye/voiceboard-backend:latest
    command: uv run gunicorn config.wsgi:application --bind 0.0.0.0:8000 --workers 3
    environment:
      - DJANGO_SETTINGS_MODULE=config.settings.production
      - DJANGO_SECRET_KEY=${SECRET_KEY}
      - DJANGO_ALLOWED_HOSTS=${DOMAIN}
      - DATABASE_URL=postgres://${POSTGRES_USER:-voiceboard}:${POSTGRES_PASSWORD}@db:5432/${POSTGRES_DB:-voiceboard}
      - REDIS_URL=redis://redis:6379
      - AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID}
      - AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY}
      - AWS_STORAGE_BUCKET_NAME=${AWS_STORAGE_BUCKET_NAME:-voiceboard}
      - AWS_S3_ENDPOINT_URL=http://minio:9000
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.web.rule=Host(`${DOMAIN}`)"
      - "traefik.http.routers.web.entrypoints=websecure"
      - "traefik.http.routers.web.tls=true"
      - "traefik.http.routers.web.tls.certresolver=letsencrypt"
      - "traefik.http.services.web.loadbalancer.server.port=8000"
    depends_on:
      db:
        condition: service_healthy
      redis:
        condition: service_healthy
    restart: unless-stopped

  channels:
    image: ghcr.io/didouye/voiceboard-backend:latest
    command: uv run daphne config.asgi:application --bind 0.0.0.0:8001
    environment:
      - DJANGO_SETTINGS_MODULE=config.settings.production
      - DJANGO_SECRET_KEY=${SECRET_KEY}
      - DJANGO_ALLOWED_HOSTS=${DOMAIN}
      - DATABASE_URL=postgres://${POSTGRES_USER:-voiceboard}:${POSTGRES_PASSWORD}@db:5432/${POSTGRES_DB:-voiceboard}
      - REDIS_URL=redis://redis:6379
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.channels.rule=Host(`${DOMAIN}`) && PathPrefix(`/ws/`)"
      - "traefik.http.routers.channels.entrypoints=websecure"
      - "traefik.http.routers.channels.tls=true"
      - "traefik.http.routers.channels.tls.certresolver=letsencrypt"
      - "traefik.http.services.channels.loadbalancer.server.port=8001"
    depends_on:
      db:
        condition: service_healthy
      redis:
        condition: service_healthy
    restart: unless-stopped

  db:
    image: postgres:16-alpine
    volumes:
      - ${DATA_PATH:-./data}/postgres:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: ${POSTGRES_DB:-voiceboard}
      POSTGRES_USER: ${POSTGRES_USER:-voiceboard}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER:-voiceboard}"]
      interval: 5s
      timeout: 5s
      retries: 5
    restart: unless-stopped

  redis:
    image: redis:7-alpine
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
    command: server /data --console-address ":9001"
    volumes:
      - ${DATA_PATH:-./data}/minio:/data
    environment:
      MINIO_ROOT_USER: ${AWS_ACCESS_KEY_ID}
      MINIO_ROOT_PASSWORD: ${AWS_SECRET_ACCESS_KEY}
    healthcheck:
      test: ["CMD", "mc", "ready", "local"]
      interval: 5s
      timeout: 5s
      retries: 5
    restart: unless-stopped
```

**3. Create `.env` file with your configuration:**

```bash
# Domain
DOMAIN=voiceboard.example.com

# Let's Encrypt email (comment traefik ACME lines if using Cloudflare)
LETSENCRYPT_EMAIL=admin@example.com

# Django secret key (generate with: openssl rand -hex 32)
SECRET_KEY=your-secure-random-key-here

# PostgreSQL
POSTGRES_PASSWORD=your-secure-db-password

# MinIO/S3
AWS_ACCESS_KEY_ID=minioadmin
AWS_SECRET_ACCESS_KEY=your-secure-minio-password

# Data path (optional, default: ./data)
# DATA_PATH=/var/lib/voiceboard
```

**4. Start the stack:**

```bash
docker compose up -d
```

**5. Initialize database:**

```bash
# Run migrations
docker compose exec web uv run python manage.py migrate

# Create admin user
docker compose exec web uv run python manage.py createsuperuser
```

**6. Access your app at `https://your-domain.com`**

---

## Configuration

### Behind Cloudflare (no Let's Encrypt)

If your server is behind Cloudflare proxy, remove the Let's Encrypt configuration from Traefik:

```yaml
traefik:
  command:
    - "--providers.docker=true"
    - "--providers.docker.exposedbydefault=false"
    - "--entrypoints.web.address=:80"
    - "--entrypoints.websecure.address=:443"
    - "--entrypoints.web.http.redirections.entrypoint.to=websecure"
    # Remove the certificatesresolvers lines
```

And remove `tls.certresolver=letsencrypt` from web/channels labels.

### Using External Services

**External PostgreSQL:**

Remove the `db` service and update web/channels environment:

```yaml
environment:
  - DATABASE_URL=postgres://user:password@your-postgres-host:5432/voiceboard
```

**External Redis:**

Remove the `redis` service and update:

```yaml
environment:
  - REDIS_URL=redis://your-redis-host:6379
```

**External S3 (AWS, Cloudflare R2, etc.):**

Remove the `minio` service and update:

```yaml
environment:
  - AWS_S3_ENDPOINT_URL=https://s3.amazonaws.com
  - AWS_S3_REGION_NAME=us-east-1
```

---

## Operations

### Updating

```bash
docker compose pull
docker compose up -d
docker compose exec web uv run python manage.py migrate
docker image prune -f
```

### Viewing Logs

```bash
docker compose logs -f          # All services
docker compose logs -f web      # Django API only
```

### Backup

```bash
tar -czvf voiceboard-backup-$(date +%Y%m%d).tar.gz data/
```

### Restore

```bash
docker compose down
tar -xzvf voiceboard-backup-YYYYMMDD.tar.gz
docker compose up -d
```

---

## Data Persistence

All data stored in `./data/` (or `${DATA_PATH}`):

| Directory | Content |
|-----------|---------|
| `data/postgres/` | PostgreSQL database |
| `data/redis/` | Redis persistence |
| `data/minio/` | File storage |
| `data/traefik/` | SSL certificates |

---

## Portainer Deployment

1. In Portainer, go to **Stacks** → **Add stack**
2. Name: `voiceboard`
3. Paste the docker-compose.yml content above
4. Add environment variables in the **Environment variables** section
5. Click **Deploy the stack**

---

## Development

For local development without Docker:

```bash
git clone https://github.com/didouye/voiceboard.git
cd voiceboard/backend

# Start dependencies only
docker compose -f docker-compose.dev.yml up -d

# Run Django locally
uv run python manage.py runserver
```
