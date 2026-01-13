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
