#!/bin/sh
set -e

echo "Waiting for database..."
while ! uv run python -c "import django; django.setup(); from django.db import connection; connection.ensure_connection()" 2>/dev/null; do
    sleep 1
done
echo "Database ready!"

echo "Running migrations..."
uv run python manage.py migrate --noinput

echo "Starting application..."
exec "$@"
