# Auto-Deploy Pipeline Design

> **Date:** 2026-01-25
> **Status:** Implemented

## Overview

Automatically deploy the backend to production when changes are pushed to main. Uses Portainer API to update the stack with the latest Docker image.

## Architecture

```
Push to main (backend/**)
       |
       v
+---------------------+
|   build-and-push    |  Build image -> Push to ghcr.io
+---------------------+
       |
       v (needs)
+---------------------+
|      deploy         |  Portainer API -> Pull & redeploy
+---------------------+
       |
       v
   voiceboard.cloud updated
```

## Implementation

### GitHub Secrets Required

| Secret | Description |
|--------|-------------|
| `PORTAINER_URL` | Portainer instance URL (e.g., `https://portainer.example.com`) |
| `PORTAINER_ACCESS_TOKEN` | API token created in Portainer (Account -> Access tokens) |
| `PORTAINER_STACK_ID` | Numeric ID of the stack to update |
| `PORTAINER_ENDPOINT_ID` | Numeric ID of the Docker environment |

### Portainer API Calls

1. **GET /api/stacks/{id}** - Retrieve current environment variables
2. **PUT /api/stacks/{id}?endpointId={id}** - Update stack with:
   - `StackFileContent`: Content from `backend/docker-compose.yml`
   - `Env`: Preserved from step 1
   - `PullImage`: `true` to force pulling the new image

### Key Features

- **Preserves environment variables**: Variables configured in Portainer UI are not overwritten
- **Updates compose file**: Any changes to `docker-compose.yml` are applied
- **Error handling**: Workflow fails if Portainer API returns non-200 status
- **No external dependencies**: Uses only `curl` and `jq` (available on ubuntu-latest)

## Alternatives Considered

1. **SSH to server + docker compose pull**: Rejected - requires SSH access and manual setup
2. **Portainer webhooks**: Requires Business Edition (paid)
3. **wirgen/portainer-stack-redeploy-action**: Outdated, incompatible with latest Portainer

## References

- [Portainer API Documentation](https://app.swaggerhub.com/apis/portainer/portainer-ce/2.33.6)
- [Portainer Stack API Discussion](https://github.com/orgs/portainer/discussions/10597)
