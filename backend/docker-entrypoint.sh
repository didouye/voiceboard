#!/bin/sh
set -e

echo "=== Database Connection Debug ==="
echo "POSTGRES_HOST: ${POSTGRES_HOST:-db}"
echo "POSTGRES_PORT: ${POSTGRES_PORT:-5432}"
echo "POSTGRES_DB: ${POSTGRES_DB:-voiceboard}"
echo "POSTGRES_USER: ${POSTGRES_USER:-voiceboard}"
echo "================================="

echo "Waiting for database..."
MAX_RETRIES=30
RETRY_COUNT=0

while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
    RETRY_COUNT=$((RETRY_COUNT + 1))
    echo "Attempt $RETRY_COUNT/$MAX_RETRIES..."

    if uv run python -c "import django; django.setup(); from django.db import connection; connection.ensure_connection()" 2>&1; then
        echo "Database ready!"
        break
    else
        echo "Connection failed, retrying in 2s..."
        sleep 2
    fi
done

if [ $RETRY_COUNT -eq $MAX_RETRIES ]; then
    echo "ERROR: Could not connect to database after $MAX_RETRIES attempts"
    exit 1
fi

echo "Running migrations..."
uv run python manage.py migrate --noinput

echo "Setting up super admin..."
uv run python manage.py setup_superadmin

echo "Starting application..."
exec "$@"
