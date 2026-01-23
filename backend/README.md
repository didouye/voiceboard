# Voiceboard Backend

Django backend for Voiceboard application.

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
