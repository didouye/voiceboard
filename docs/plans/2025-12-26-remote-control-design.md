# Remote Control Design

This document describes the architecture for remote control functionality in Voiceboard, enabling users to trigger sounds from mobile devices and web browsers.

## Overview

The remote control system allows users to control their desktop Voiceboard application from:
- **Mobile app** (iOS/Android) - local network or cloud relay
- **Web remote** (Angular) - cloud only

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLOUD (Django)                          │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────────────┐ │
│  │   REST API   │  │  WebSocket   │  │   Remote Registry     │ │
│  │  (auth, sync)│  │   Gateway    │  │  (tokens, desktops)   │ │
│  └──────────────┘  └──────────────┘  └───────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
         ▲                   ▲                      ▲
         │                   │                      │
         │ HTTPS             │ WSS                  │ WSS
         │                   │                      │
┌────────┴───────┐   ┌───────┴────────┐    ┌───────┴────────┐
│  Mobile App    │   │  Desktop App   │    │   Web Remote   │
│   (Flutter)    │   │    (Tauri)     │    │   (Angular)    │
└────────────────┘   └────────────────┘    └────────────────┘
         │                   ▲
         │ WS local          │
         └───────────────────┘
              (same network)
```

### Key Components

- **Desktop**: Exposes a local WebSocket server + maintains a WSS connection to cloud
- **Mobile**: Connects locally (WS) or remotely (via cloud relay)
- **Web Remote**: Always via cloud (WSS)
- **Cloud Gateway**: Relays commands to the correct desktop

### Communication Model

**Hybrid approach:**
- Local mode: Direct WebSocket connection for minimal latency
- Remote mode: Cloud relay via persistent WebSocket connection

### Authentication

| Scenario | Flow | Authentication |
|----------|------|----------------|
| Mobile local | Mobile → Desktop direct | Signature (shared secret from QR code) |
| Web cloud | Web → Cloud → Desktop | Cloud session (JWT) |
| Mobile remote | Mobile → Cloud → Desktop | Derived token from pairing |

## Pairing Flow

### Step 1: mDNS Discovery

The desktop broadcasts its presence on the local network using mDNS service type `_voiceboard._tcp`.

### Step 2: QR Code Pairing

```
┌──────────────┐                    ┌──────────────┐
│    Mobile    │                    │   Desktop    │
│   (Flutter)  │                    │   (Tauri)    │
└──────┬───────┘                    └──────┬───────┘
       │                                   │
       │  1. mDNS Discovery                │
       │  ◄──────────────────────────────  │ Broadcast "_voiceboard._tcp"
       │                                   │
       │  2. Select desktop                │
       │  ─────────────────────────────►   │
       │                                   │
       │  3. Desktop shows QR code         │
       │                                   │ QR contains:
       │                                   │ - desktop_id
       │  4. Scan QR code                  │ - local_secret
       │  ◄─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │ - local_ip:port
       │                                   │ - pairing_nonce
       │  5. Local WebSocket handshake     │
       │  ─────────────────────────────►   │ Signed with local_secret
       │                                   │
       │  6. Desktop confirms              │
       │  ◄──────────────────────────────  │ + sends remote_name
       │                                   │
       │         LOCAL PAIRING OK          │
       │                                   │
```

### Step 3: Cloud Sync (if cloud enabled)

```
┌──────────────┐       ┌──────────────┐       ┌──────────────┐
│   Desktop    │       │    Cloud     │       │    Mobile    │
└──────┬───────┘       └──────┬───────┘       └──────┬───────┘
       │                      │                      │
       │  7. Register remote  │                      │
       │  ───────────────────►│                      │
       │  {desktop_id,        │                      │
       │   remote_id,         │                      │
       │   derived_token}     │                      │
       │                      │                      │
       │  8. Token stored     │                      │
       │  ◄───────────────────│                      │
       │                      │                      │
       │                      │  9. Mobile sync      │
       │                      │◄─────────────────────│
       │                      │  {derived_token}     │
       │                      │                      │
       │                      │  10. Token validated │
       │                      │─────────────────────►│
       │                      │                      │
       │       REMOTE MODE AVAILABLE                │
```

### Generated Data

- `remote_id`: Unique UUID for the remote
- `local_secret`: 32 random bytes for signing local requests
- `derived_token`: HMAC(local_secret, desktop_id + remote_id) for cloud auth

## Command Protocol

### Message Format (JSON over WebSocket)

```typescript
// Command from remote → Desktop
interface RemoteCommand {
  id: string;           // UUID for request/response correlation
  action: "play" | "stop" | "stop_all" | "set_volume" | "get_state";
  pad_id?: number;      // 0-11 for pads
  value?: number;       // volume (0-100), etc.
  timestamp: number;    // Unix ms for anti-replay
  signature?: string;   // HMAC-SHA256 in local mode only
}

// Response Desktop → Remote
interface RemoteResponse {
  id: string;           // Same UUID as command
  success: boolean;
  error?: string;
}

// Push Desktop → Remote (real-time state)
interface StateUpdate {
  type: "state";
  pads: PadState[];
  master_volume: number;
  is_running: boolean;
}

interface PadState {
  id: number;
  name: string;
  is_playing: boolean;
  volume: number;
  has_sound: boolean;
  icon?: string;        // URL or base64
}
```

### Local Mode Signature

```
payload = JSON.stringify({action, pad_id, value, timestamp})
signature = HMAC-SHA256(local_secret, payload)
```

### Anti-Replay Protection

- Desktop rejects commands with `timestamp` > 30 seconds in the past
- Desktop caches recent `id` values to prevent duplicates

### Supported Commands

| Action | Description | Parameters |
|--------|-------------|------------|
| `play` | Play a pad | `pad_id` |
| `stop` | Stop a pad | `pad_id` |
| `stop_all` | Stop all sounds | - |
| `set_volume` | Change pad volume | `pad_id`, `value` |
| `get_state` | Request full state | - |

## Cloud Architecture (Django)

### New Components

```
voiceboard-backend/
├── apps/
│   ├── remotes/                    # New Django app
│   │   ├── models.py               # Remote, DesktopConnection
│   │   ├── consumers.py            # WebSocket consumers (Channels)
│   │   ├── views.py                # REST API registration
│   │   └── routing.py              # WS routes
│   └── ...
├── config/
│   └── asgi.py                     # ASGI for WebSocket
└── requirements.txt                # + channels, channels-redis
```

### Models

```python
class DesktopConnection(models.Model):
    """Desktop connected to cloud"""
    desktop_id = models.UUIDField(unique=True)
    user = models.ForeignKey(User, on_delete=models.CASCADE)
    channel_name = models.CharField(max_length=255)  # WS channel
    connected_at = models.DateTimeField(auto_now=True)

class Remote(models.Model):
    """Paired remote control"""
    remote_id = models.UUIDField(unique=True)
    desktop = models.ForeignKey(DesktopConnection, on_delete=models.CASCADE)
    derived_token_hash = models.CharField(max_length=64)  # SHA256
    name = models.CharField(max_length=100)  # "John's iPhone"
    created_at = models.DateTimeField(auto_now_add=True)
    last_used = models.DateTimeField(null=True)
    is_revoked = models.BooleanField(default=False)
```

### WebSocket Gateway (Django Channels)

```python
# consumers.py
class DesktopConsumer(AsyncWebsocketConsumer):
    """Persistent connection Desktop → Cloud"""
    async def connect(self):
        # Auth via JWT in query string
        # Register channel_name in DesktopConnection

    async def relay_command(self, event):
        # Relay command to desktop
        await self.send(json.dumps(event["command"]))

class RemoteConsumer(AsyncWebsocketConsumer):
    """Remote connection (web or mobile distant)"""
    async def receive(self, text_data):
        command = json.loads(text_data)
        # Validate derived_token
        # Find target desktop
        # Send via channel_layer to DesktopConsumer
```

### REST Endpoints

| Method | Route | Description |
|--------|-------|-------------|
| POST | `/api/remotes/register/` | Desktop registers a remote |
| DELETE | `/api/remotes/{id}/` | Revoke a remote |
| GET | `/api/remotes/` | List remotes |

## Mobile Application (Flutter)

### Project Structure

```
voiceboard_remote/
├── lib/
│   ├── main.dart
│   ├── core/
│   │   ├── crypto.dart             # HMAC signatures
│   │   └── websocket_client.dart   # WS connection local/cloud
│   ├── discovery/
│   │   ├── mdns_scanner.dart       # mDNS discovery
│   │   └── qr_scanner.dart         # QR code scanning
│   ├── models/
│   │   ├── desktop.dart            # Paired desktop
│   │   ├── pad.dart                # Pad state
│   │   └── command.dart            # Commands
│   ├── services/
│   │   ├── pairing_service.dart    # Pairing logic
│   │   ├── remote_service.dart     # Send commands
│   │   └── storage_service.dart    # Local persistence
│   └── ui/
│       ├── screens/
│       │   ├── home_screen.dart    # Paired desktops list
│       │   ├── discovery_screen.dart
│       │   ├── pairing_screen.dart # QR scanner
│       │   └── remote_screen.dart  # Pad grid
│       └── widgets/
│           ├── pad_button.dart     # Pad button
│           └── connection_status.dart
├── pubspec.yaml
└── ...
```

### Key Dependencies

```yaml
dependencies:
  flutter:
    sdk: flutter
  bonsoir: ^5.1.0           # mDNS discovery
  mobile_scanner: ^3.5.0    # QR code scanner
  web_socket_channel: ^2.4.0
  crypto: ^3.0.3            # HMAC-SHA256
  hive: ^2.2.3              # Local storage
  provider: ^6.1.0          # State management
```

### Connection Logic

```dart
class RemoteService {
  WebSocketChannel? _channel;

  Future<void> connect(Desktop desktop) async {
    if (await _isOnSameNetwork(desktop)) {
      // Local mode: direct WS
      _channel = WebSocketChannel.connect(
        Uri.parse('ws://${desktop.localIp}:${desktop.localPort}')
      );
      _mode = ConnectionMode.local;
    } else {
      // Remote mode: via cloud
      _channel = WebSocketChannel.connect(
        Uri.parse('wss://api.voiceboard.io/ws/remote/')
      );
      _mode = ConnectionMode.cloud;
      _authenticate(desktop.derivedToken);
    }
  }

  void playPad(int padId) {
    final command = Command(action: 'play', padId: padId);
    if (_mode == ConnectionMode.local) {
      command.sign(desktop.localSecret);
    }
    _channel?.sink.add(jsonEncode(command));
  }
}
```

### Remote Screen UI

- 4x3 pad grid (matching desktop)
- Connection indicator (local green / cloud blue / disconnected red)
- Prominent "Stop All" button
- Master volume slider
- Pull-to-refresh to sync state

## Web Remote (Angular)

The web remote is integrated into the cloud dashboard as a dedicated section:

- Route: `/dashboard/remote`
- Uses existing Angular infrastructure
- WebSocket connection to cloud gateway
- Same command protocol as mobile
- Authentication via existing cloud session (JWT)
