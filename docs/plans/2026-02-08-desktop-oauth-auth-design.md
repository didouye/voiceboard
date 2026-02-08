# Desktop OAuth Authentication Design

> **Date:** 2026-02-08
> **Status:** Active

## Overview

Add optional OAuth authentication (Google/Discord) to the Tauri desktop app, connecting to the existing backend API. The app works fully offline without an account. Users can optionally sign in to unlock future cloud features (sync, teams, etc.).

## Authentication Flow

1. User clicks "Sign in" in the sidebar
2. Dropdown menu offers "Google" or "Discord"
3. Rust backend starts a temporary HTTP server on `localhost:18019`
4. Rust calls backend API: `GET /api/auth/{provider}/url/?redirect_uri=http://localhost:18019/callback`
5. Backend returns the OAuth authorization URL (with `state` param)
6. App opens the system browser to that URL
7. User authenticates with the provider
8. Provider redirects to `http://localhost:18019/callback?code=XXX&state=YYY`
9. Local server captures the code, serves an HTML page: "Sign in successful, you can close this tab"
10. Rust verifies `state` matches, then sends code to backend: `POST /api/auth/{provider}/callback/`
11. Backend exchanges code, creates/finds user, returns JWT tokens + user object
12. App stores tokens in `auth.json` (Tauri Store), updates sidebar, shows success toast
13. Local HTTP server shuts down

**Logout:** Call `POST /api/auth/logout/` to blacklist refresh token, delete local `auth.json`, reset sidebar.

## Rust Backend (Tauri Commands)

### New Tauri Commands

- **`auth_login(provider: String) -> AuthSession`** — Starts local HTTP server on port 18019, calls backend for OAuth URL, opens system browser, waits for callback (2 min timeout). Returns `{ access, refresh, user }`.
- **`auth_logout()`** — Calls `POST /api/auth/logout/` with refresh token, deletes `auth.json`.
- **`auth_get_session() -> Option<AuthSession>`** — Reads `auth.json`, returns current session or null.
- **`auth_refresh_token() -> AuthSession`** — Calls `POST /api/auth/refresh/`, updates `auth.json` with new tokens.

### Local HTTP Server

- Temporary server on `localhost:18019` using a lightweight HTTP library (`tiny_http` or similar)
- Only runs during OAuth flow (max 2 minutes)
- Serves a success HTML page on callback, then extracts `code` and `state` params
- Shuts down after capturing callback or on timeout

### Persistence

File `auth.json` in app data directory via Tauri Plugin Store:

```json
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "user": {
    "id": 1,
    "email": "user@example.com",
    "display_name": "John",
    "avatar_url": "https://...",
    "subscription_tier": "free"
  }
}
```

### Dependencies

- `reqwest` — HTTP client for backend API calls
- `tiny_http` (or similar) — Lightweight HTTP server for OAuth callback
- `open` — Open system browser (already available via Tauri shell plugin)

## Angular Frontend

### New Service: `AuthService`

Location: `src/app/core/services/auth.service.ts`

- Signal `user: WritableSignal<User | null>` — Current user or null
- Computed signal `isLoggedIn: Signal<boolean>` — Derived from `user`
- `login(provider: 'google' | 'discord')` — Calls Tauri `auth_login`, updates `user` signal
- `logout()` — Calls Tauri `auth_logout`, resets `user` signal
- `initialize()` — Called at startup, loads session via `auth_get_session()`. If user exists, silently refreshes token via `auth_refresh_token()` to validate session. On refresh failure (expired), resets session silently.

### Sidebar Modification

**Not signed in:**
- Generic user icon + "Sign in" text
- Click opens dropdown: Google logo + "Google", Discord logo + "Discord"

**Signed in:**
- Round avatar image + display name (or first name)
- Click opens dropdown: "Signed in as {email}", separator, "Sign out"

### Startup Flow

Add `AuthService.initialize()` in `AppComponent.ngOnInit()` after existing checks (VB-Cable, etc.). Non-blocking — app loads normally, auth resolves in background.

## Provider Configuration

### Google Cloud Console

- Add `http://localhost:18019/callback` to authorized redirect URIs (or use "Desktop" client type which allows any localhost port)

### Discord Developer Portal

- Add `http://localhost:18019/callback` to OAuth2 redirect URIs

### Backend CORS

No changes needed. Desktop app calls go through Rust (`reqwest`), not browser — no CORS constraints.

## Security

- **CSRF protection:** Backend generates `state` parameter. App stores it and verifies it matches on callback.
- **Minimal exposure:** Local server runs only during OAuth flow (max 2 min), then shuts down.
- **Port interception risk:** Fixed port 18019 is known — a local malicious process could listen on it. Accepted risk (same model as VS Code, gcloud CLI). The `state` verification protects against interception.
- **Token storage:** `auth.json` in OS app data directory, protected by OS user permissions. Same security level as settings, soundboard data.

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Port 18019 occupied | Toast: "Cannot start authentication, port busy" |
| 2-minute timeout | Shut down server, toast: "Authentication cancelled" |
| Backend error (invalid code, provider down) | Toast with error message |
| Refresh token expired | Silent session reset, user becomes "not signed in" |
| No internet connection | Toast: "No internet connection" |

## Out of Scope

- Profile editing page (future)
- Subscription management UI (future)
- Auto-refresh of access token on API calls (future, when cloud features need API calls)
- Account linking (connecting both Google and Discord to same account)
