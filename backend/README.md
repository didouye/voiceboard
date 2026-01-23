# Voiceboard Backend

Django backend for Voiceboard application.

## Server Deployment

### Prerequisites

- Linux server (Ubuntu 22.04+ recommended)
- Docker Engine 24+ and Docker Compose v2+
- Domain name pointing to your server
- Ports 80 and 443 open

### Quick Deploy

**1. Create a directory and download the compose files:**

```bash
mkdir -p /opt/voiceboard && cd /opt/voiceboard

# Download compose files
curl -O https://raw.githubusercontent.com/didouye/voiceboard/main/backend/docker-compose.yml
curl -O https://raw.githubusercontent.com/didouye/voiceboard/main/backend/docker-compose.letsencrypt.yml
curl -O https://raw.githubusercontent.com/didouye/voiceboard/main/backend/.env.example

# Create your .env
cp .env.example .env
nano .env
```

**2. Configure `.env`:**

```bash
# Domain
DOMAIN=voiceboard.example.com

# Django
SECRET_KEY=your-secure-random-key    # Generate with: openssl rand -hex 32
DEBUG=false
ALLOWED_HOSTS=${DOMAIN}

# PostgreSQL (required if using embedded db)
POSTGRES_PASSWORD=your-secure-db-password

# MinIO/S3 (required if using embedded s3)
AWS_ACCESS_KEY_ID=minioadmin
AWS_SECRET_ACCESS_KEY=your-secure-minio-password

# Let's Encrypt (only for docker-compose.letsencrypt.yml)
LETSENCRYPT_EMAIL=admin@example.com
```

**3. Start the stack:**

```bash
# All embedded services (PostgreSQL, Redis, MinIO) + Let's Encrypt SSL
COMPOSE_PROFILES=db,redis,s3 docker compose -f docker-compose.yml -f docker-compose.letsencrypt.yml up -d

# Or behind Cloudflare (no Let's Encrypt needed)
COMPOSE_PROFILES=db,redis,s3 docker compose up -d
```

**4. Initialize database:**

```bash
docker compose exec web uv run python manage.py migrate
docker compose exec web uv run python manage.py createsuperuser
```

**5. Access your app at `https://your-domain.com`**

---

## Service Profiles

The embedded services (PostgreSQL, Redis, MinIO) are controlled via **Docker Compose profiles**. Only services whose profiles are activated will start.

| Profile | Service    | Description                     |
|---------|------------|---------------------------------|
| `db`    | PostgreSQL | Database                        |
| `redis` | Redis      | Cache and Channels backend      |
| `s3`    | MinIO      | S3-compatible file storage      |

### Usage Examples

```bash
# All embedded services
COMPOSE_PROFILES=db,redis,s3 docker compose up -d

# External PostgreSQL, embedded Redis and MinIO
COMPOSE_PROFILES=redis,s3 docker compose up -d

# Only embedded PostgreSQL
COMPOSE_PROFILES=db docker compose up -d

# All external services (no profiles needed)
docker compose up -d
```

You can also set `COMPOSE_PROFILES` in your `.env` file:

```bash
# .env
COMPOSE_PROFILES=db,redis,s3
```

Then simply run:

```bash
docker compose up -d
```

---

## External Services Configuration

When **not** activating a profile, configure the external service URL in `.env`:

### External PostgreSQL (no `db` profile)

```bash
DATABASE_URL=postgres://user:password@your-postgres-host:5432/voiceboard
```

### External Redis (no `redis` profile)

```bash
REDIS_URL=redis://your-redis-host:6379
```

### External S3 (no `s3` profile)

```bash
AWS_S3_ENDPOINT_URL=https://s3.amazonaws.com
AWS_S3_REGION_NAME=us-east-1
AWS_ACCESS_KEY_ID=your-access-key
AWS_SECRET_ACCESS_KEY=your-secret-key
AWS_STORAGE_BUCKET_NAME=your-bucket
```

---

## SSL Configuration

### With Let's Encrypt

Use the overlay file for automatic SSL certificates:

```bash
COMPOSE_PROFILES=db,redis,s3 docker compose -f docker-compose.yml -f docker-compose.letsencrypt.yml up -d
```

Requires `LETSENCRYPT_EMAIL` in `.env`.

### Behind Cloudflare

If Cloudflare handles SSL (orange cloud enabled), use the base compose file only:

```bash
COMPOSE_PROFILES=db,redis,s3 docker compose up -d
```

Cloudflare terminates SSL and forwards HTTP to your server.

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
docker compose logs -f traefik  # Reverse proxy
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

All data stored in `${DATA_PATH:-./data}/`:

| Directory | Content | Profile |
|-----------|---------|---------|
| `data/postgres/` | PostgreSQL database | `db` |
| `data/redis/` | Redis persistence | `redis` |
| `data/minio/` | File storage | `s3` |
| `data/traefik/` | SSL certificates | always |

---

## Portainer Deployment

1. Go to **Stacks** → **Add stack**
2. Name: `voiceboard`
3. **Web editor**: paste content from `docker-compose.yml` (and merge `docker-compose.letsencrypt.yml` if needed)
4. **Environment variables**: add your configuration
5. **Advanced**: set `COMPOSE_PROFILES` to `db,redis,s3` (or as needed)
6. Click **Deploy the stack**

---

## Development

For local development:

```bash
git clone https://github.com/didouye/voiceboard.git
cd voiceboard/backend

# Start dependencies only
docker compose -f docker-compose.dev.yml up -d

# Run Django locally
uv run python manage.py runserver
```
