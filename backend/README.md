# Voiceboard Backend

Django backend for Voiceboard application.

## Server Deployment

### Prerequisites

- Linux server (Ubuntu 22.04+ recommended)
- Docker Engine 24+ and Docker Compose v2+
- Domain name pointing to your server
- Ports 80 and 443 open

### 1. Install Docker

```bash
# Install Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER

# Log out and back in, then verify
docker --version
docker compose version
```

### 2. Clone and Configure

```bash
# Create app directory
sudo mkdir -p /opt/voiceboard
sudo chown $USER:$USER /opt/voiceboard
cd /opt/voiceboard

# Download docker-compose files (or clone repo)
curl -O https://raw.githubusercontent.com/didouye/voiceboard/main/backend/docker-compose.yml
curl -O https://raw.githubusercontent.com/didouye/voiceboard/main/backend/docker-compose.letsencrypt.yml
curl -O https://raw.githubusercontent.com/didouye/voiceboard/main/backend/.env.example

# Configure environment
cp .env.example .env
nano .env
```

**Required `.env` settings:**

```bash
DOMAIN=voiceboard.example.com
SECRET_KEY=your-secure-random-key
POSTGRES_PASSWORD=your-secure-password
AWS_ACCESS_KEY_ID=minio-access-key
AWS_SECRET_ACCESS_KEY=minio-secret-key
LETSENCRYPT_EMAIL=admin@example.com  # For SSL certificates
```

### 3. Start Services

```bash
# With Let's Encrypt SSL (recommended for production)
COMPOSE_PROFILES=db,redis,s3 docker compose -f docker-compose.yml -f docker-compose.letsencrypt.yml up -d

# Or behind Cloudflare (Cloudflare handles SSL)
COMPOSE_PROFILES=db,redis,s3 docker compose up -d
```

### 4. Initialize Database

```bash
# Run migrations
docker compose exec web uv run python manage.py migrate

# Create admin user
docker compose exec web uv run python manage.py createsuperuser
```

### 5. Verify Deployment

```bash
# Check all containers are running
docker compose ps

# Check logs
docker compose logs -f

# Test API
curl https://your-domain.com/api/health/
```

---

## Configuration Options

### Service Profiles

Activate embedded services with `COMPOSE_PROFILES`:

| Profile | Service    | When to use                          |
|---------|------------|--------------------------------------|
| `db`    | PostgreSQL | No external PostgreSQL available     |
| `redis` | Redis      | No external Redis available          |
| `s3`    | MinIO      | No external S3-compatible storage    |

```bash
# All embedded (typical self-hosted setup)
COMPOSE_PROFILES=db,redis,s3 docker compose up -d

# External PostgreSQL only
COMPOSE_PROFILES=redis,s3 docker compose up -d

# All external services
docker compose up -d
```

### External Services

Configure in `.env` when not using embedded services:

```bash
# External PostgreSQL (omit "db" from COMPOSE_PROFILES)
DATABASE_URL=postgres://user:password@hostname:5432/voiceboard

# External Redis (omit "redis" from COMPOSE_PROFILES)
REDIS_URL=redis://hostname:6379

# External S3 (omit "s3" from COMPOSE_PROFILES)
AWS_S3_ENDPOINT_URL=https://s3.amazonaws.com
AWS_S3_REGION_NAME=us-east-1
```

### Data Persistence

All data stored in `${DATA_PATH:-./data}/` on host:

| Directory | Content |
|-----------|---------|
| `data/postgres/` | PostgreSQL database files |
| `data/redis/` | Redis AOF persistence |
| `data/minio/` | MinIO object storage |
| `data/traefik/` | Let's Encrypt certificates |

---

## Operations

### Updating

```bash
cd /opt/voiceboard

# Pull latest images
docker compose pull

# Restart with new images
docker compose up -d

# Run migrations if needed
docker compose exec web uv run python manage.py migrate

# Clean old images
docker image prune -f
```

### Viewing Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f web
docker compose logs -f channels
docker compose logs -f traefik
```

### Backup

```bash
# Stop services (optional, for consistent backup)
docker compose stop

# Backup data directory
tar -czvf voiceboard-backup-$(date +%Y%m%d).tar.gz data/

# Restart services
docker compose start
```

### Restore

```bash
# Stop services
docker compose down

# Restore data
tar -xzvf voiceboard-backup-YYYYMMDD.tar.gz

# Start services
COMPOSE_PROFILES=db,redis,s3 docker compose up -d
```

---

## Troubleshooting

### Container won't start

```bash
# Check logs
docker compose logs web

# Common issues:
# - Missing required env vars → check .env
# - Port already in use → stop conflicting service
# - Permission denied on data/ → chown -R $USER:$USER data/
```

### SSL certificate not working

```bash
# Check Traefik logs
docker compose logs traefik

# Verify domain DNS points to server
dig +short your-domain.com

# Ensure ports 80/443 are open
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
```

### Database connection failed

```bash
# Check if db container is healthy
docker compose ps db

# Check db logs
docker compose logs db

# Verify POSTGRES_PASSWORD matches in .env
```

---

## Development

For local development, use `docker-compose.dev.yml`:

```bash
# Start only PostgreSQL and Redis
docker compose -f docker-compose.dev.yml up -d

# Run Django locally
uv run python manage.py runserver
```
