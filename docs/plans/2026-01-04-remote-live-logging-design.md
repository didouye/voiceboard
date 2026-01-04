# Remote Live Logging Design

> **Date**: 2026-01-04
> **Status**: Approved
> **Depends on**: Phase 4 (Django backend, user accounts)

## Overview

Remote Live Logging enables the developer to monitor logs from a remote user in real-time to facilitate support.

### User Flow

1. User encounters a problem and contacts support (Discord/email)
2. Support asks user to enable **Debug Mode** in the app
3. App displays consent message: *"Logs will be shared with Voiceboard support"*
4. Once enabled, logs are streamed to Django server
5. Developer receives an **email** with link to log session
6. Developer opens dashboard and views logs in real-time
7. After resolution, user disables debug mode
8. Logs are retained for **7 days** then auto-deleted

### Data Transmitted

| Category | Data |
|----------|------|
| **Logs** | Timestamp, level, message, context |
| **System** | OS, app version, architecture |
| **Audio** | Selected devices, sample rates, VB-Cable status |
| **App State** | Mixer running/stopped, loaded sounds, soundboard config |

## Architecture

### Desktop Side (Tauri/Angular)

```
┌─────────────────────────────────────────────────────────┐
│                    Voiceboard App                        │
│  ┌─────────────────┐    ┌─────────────────────────────┐ │
│  │ Debug Console   │───▶│ RemoteLogService            │ │
│  │ Service         │    │ - WebSocket connection      │ │
│  │ (local logs)    │    │ - Batch & send every 500ms  │ │
│  └─────────────────┘    │ - Reconnect on disconnect   │ │
│                         └──────────────┬──────────────┘ │
└────────────────────────────────────────┼────────────────┘
                                         │ WSS
                                         ▼
```

### Server Side (Django)

```
┌─────────────────────────────────────────────────────────┐
│                    Django Backend                        │
│  ┌─────────────────┐    ┌─────────────────────────────┐ │
│  │ Django Channels │───▶│ LogSession Model            │ │
│  │ (WebSocket)     │    │ - user, started_at          │ │
│  │                 │    │ - metadata (OS, version...) │ │
│  └─────────────────┘    │ - expires_at (7 days)       │ │
│           │             └──────────────┬──────────────┘ │
│           │                            │                │
│           ▼                            ▼                │
│  ┌─────────────────┐    ┌─────────────────────────────┐ │
│  │ Dashboard       │◀───│ LogEntry Model              │ │
│  │ (htmx or Angular│    │ - session_id, timestamp     │ │
│  │  SPA embedded)  │    │ - level, message, context   │ │
│  └─────────────────┘    └─────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### WebSocket Protocol

```json
// Initial connection (auth + metadata)
{
  "type": "session_start",
  "token": "jwt_token",
  "metadata": {
    "os": "Windows 11",
    "app_version": "20260104.1200",
    "devices": {...}
  }
}

// Log batch (sent every 500ms)
{
  "type": "logs",
  "entries": [
    {"ts": 1704365400, "level": "info", "msg": "...", "ctx": {...}}
  ]
}
```

## Dashboard

**URL**: `voiceboard.app/admin/logs/` (staff only)

### Features

| Feature | Description |
|---------|-------------|
| **Active sessions list** | Users with debug mode ON, sorted by date |
| **Real-time view** | Logs arriving live (auto-scroll) |
| **Filters** | By level (debug/info/warn/error), by text |
| **Metadata** | Side panel with OS, devices, app state |
| **Export** | Download logs as `.txt` or `.json` |
| **History** | Sessions from last 7 days |

### Session Interface

```
┌─────────────────────────────────────────────────────────┐
│  🟢 JohnDoe • Windows 11 • v20260104.1200    [Export ▼] │
├─────────────────────────────────────────────────────────┤
│  Devices: Microphone (Realtek) → VB-Cable               │
│  Sample Rate: 48000 Hz • Mixer: Running • Sounds: 12    │
├─────────────────────────────────────────────────────────┤
│  [All] [Debug] [Info] [Warn] [Error]     🔍 Filter...   │
├─────────────────────────────────────────────────────────┤
│  14:32:01.123 [INFO]  AudioEngine started               │
│  14:32:01.456 [DEBUG] Input device opened: Realtek...   │
│  14:32:02.789 [WARN]  Sample rate mismatch detected     │
│  14:32:03.012 [ERROR] Buffer underrun on output         │
│  ▼ auto-scroll                                          │
└─────────────────────────────────────────────────────────┘
```

## Email Notification

**Subject**: `[Voiceboard] Debug session started: JohnDoe`

**Body**:
```
User: JohnDoe
OS: Windows 11
App Version: 20260104.1200
Started: 2026-01-04 14:32:00 UTC

View logs: https://voiceboard.app/admin/logs/session/abc123/
```

## Security & Privacy

### User Consent

- **First activation**: Explicit message *"By enabling debug mode, your logs will be shared with Voiceboard support to help resolve your issue. Logs are deleted after 7 days."*
- **"Don't show again" checkbox**: Optional for regular users
- **Visible indicator**: Badge in UI when streaming is active (e.g., 🔴 "Logs shared")

### Security Measures

| Aspect | Measure |
|--------|---------|
| **Auth** | JWT token required for WebSocket connection |
| **Transport** | WSS (TLS mandatory) |
| **Dashboard** | Access restricted to staff accounts (`is_staff=True`) |
| **Rate limit** | Max 100 logs/second per session |

### Excluded Data

Never stream:
- Full file paths (only file names)
- Tokens/secrets
- Audio content

### Retention

- **Auto-expiration**: `expires_at = created_at + 7 days`
- **Cron job**: Daily deletion of expired sessions
- **Manual deletion**: Available from dashboard

## Implementation Notes

This feature depends on Phase 4 infrastructure:
- Django backend with Django Channels (WebSocket support)
- User authentication system (JWT)
- Redis for WebSocket channel layer

### Desktop Changes Required

1. New `RemoteLogService` in Angular
2. Consent dialog component
3. "Logs shared" indicator in debug console
4. Metadata collection (OS, devices, app state)

### Backend Changes Required

1. `LogSession` and `LogEntry` models
2. WebSocket consumer for log ingestion
3. Admin dashboard views
4. Email notification on session start
5. Celery task for expired session cleanup
