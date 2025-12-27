# Voiceboard Roadmap

## Phase 1 - Core Application (MVP)

### Done
- [x] Hexagonal architecture (DDD, Ports & Adapters)
- [x] AudioEngine with real-time mixing (microphone + sounds)
- [x] Audio device management (input/output) via CPAL
- [x] Audio file decoding (MP3, OGG, WAV, FLAC) via Rodio
- [x] Mic volume/master volume and mic mute
- [x] Settings and soundboard persistence
- [x] Soundboard with 12 pads and keyboard shortcuts
- [x] Device selector (input/output)
- [x] Master control with volume and start/stop button
- [x] Modern UI with dark theme
- [x] Sound preview on system output with device selection

### To Do
- [ ] Virtual microphone output - Implement WASAPI to send audio to Virtual Audio Driver
- [x] Level visualization (VU meters) - AudioEngine emits `LevelUpdate`, connect to UI
- [ ] Mic monitoring on preview output - Switch next to VU meter to hear own microphone
- [ ] Unit and integration tests
- [ ] Individual volume control per pad in UI
- [ ] Bulk import - Import multiple audio files at once

---

## Phase 2 - Distribution & CI/CD

### Done
- [x] **GitHub Actions CI**
  - Automated build and compilation
  - Clippy linting
  - Automated tests

- [x] **Multi-platform Release Build**
  - Windows x64 (.msi, .zip)
  - macOS ARM64 and x64 (.dmg, .tar.gz)
  - Linux x64 (.AppImage, .tar.gz)
  - CalVer versioning (YYYYMMDD.HHMM)
  - Automatic GitHub Release on merge to main
  - SHA256 checksums

- [x] **Auto-Update System**
  - Check for updates on startup
  - Toast notification with "Update now" button
  - Auto-download and restart

### To Do
- [ ] **Windows Installer Improvements**
  - Bundled Virtual Audio Driver

- [ ] **Linux Support**
  - Virtual audio device (PulseAudio/PipeWire)
  - Additional packages (.deb, .rpm)

---

## Phase 3 - UI/UX Redesign

### Stack
- **Tailwind CSS** (Tailwind Plus account available)

### To Do
- [ ] **Interface Redesign**
  - Modern and attractive design
  - Smooth animations
  - Compact / extended mode
  - Customizable themes (dark/light/custom)
  - Icons and visuals for pads
  - Drag & drop to reorganize pads

---

## Phase 4 - Cloud & Collaboration

### Stack
- **Backend**: Django + Django REST Framework (DRF)
- **Database**: PostgreSQL
- **Cache**: Redis
- **Storage**: S3-compatible (MinIO, AWS S3, Cloudflare R2)
- **Auth**: Django built-in + JWT
- **Payments**: Stripe

### Backend Cloud
- [ ] **Infrastructure**
  - REST API with DRF
  - PostgreSQL database
  - Audio file storage (S3-compatible)
  - JWT Authentication

- [ ] **User Management**
  - Account creation
  - User profile
  - License management
  - Billing (Stripe)

- [ ] **Teams**
  - Team creation
  - Member invitations
  - Roles and permissions
  - Personal soundboard per user
  - Shared soundboard per team

### Cloud Features
- [ ] **Synchronization**
  - Soundboard sync between devices
  - Real-time modification sync

- [ ] **Sound Search**
  - Integration with sound APIs (Freesound, etc.)
  - Keyword search
  - Preview and direct import

- [ ] **AI Sound Generation**
  - Integration with TTS models (ElevenLabs, etc.)
  - AI sound effect generation
  - Generation history

- [ ] **Remote Control Infrastructure**
  - WebSocket gateway (Django Channels + Redis)
  - Desktop persistent connection to cloud
  - Remote registry (paired devices, tokens, revocation)
  - Command relay from remotes to desktops

- [ ] **Web Remote Control**
  - Angular SPA integrated into dashboard
  - WebSocket connection to cloud gateway
  - Real-time pad grid with state updates
  - Works from anywhere (no local network required)

---

## Phase 5 - Discord Bot

### Stack
- **Preferred**: Rust with [serenity](https://github.com/serenity-rs/serenity) + [songbird](https://github.com/serenity-rs/songbird) for voice
- **Fallback**: Python with discord.py if Rust voice support is insufficient

### To Do
- [ ] **Discord Bot**
  - Bot creation (serenity/songbird or discord.py)
  - Voice channel connection
  - Link to a Voiceboard team

- [ ] **Features**
  - Play shared soundboard sounds in voice channel
  - Slash commands to trigger sounds
  - Web control panel for team
  - Real-time sync with desktop app

---

## Phase 6 - Mobile Remote Control

### Stack
- **Framework**: Flutter (iOS + Android)
- **mDNS**: bonsoir package
- **QR Scanner**: mobile_scanner package
- **State Management**: Provider
- **Local Storage**: Hive

### Desktop App Extensions
- [ ] **Local WebSocket Server**
  - Expose WS server on configurable port
  - HMAC-SHA256 signature validation
  - Anti-replay protection (timestamp + nonce cache)
  - State broadcast to connected remotes

- [ ] **mDNS Broadcast**
  - Service type: `_voiceboard._tcp`
  - Broadcast desktop name and port
  - Auto-discovery on local network

- [ ] **QR Code Pairing**
  - Generate QR code with pairing data
  - Contains: desktop_id, local_secret, local_ip, port
  - Display in settings or dedicated pairing screen

- [ ] **Cloud Sync for Remotes**
  - Register paired remotes to cloud
  - Generate derived token for cloud auth
  - Revocation management

### Mobile App (Flutter)
- [ ] **Discovery & Pairing**
  - mDNS scanner for local desktops
  - QR code scanner for secure pairing
  - Store paired desktops locally

- [ ] **Remote Control UI**
  - 4x3 pad grid (matching desktop)
  - Connection status indicator (local/cloud/offline)
  - Stop All button
  - Master volume slider
  - Real-time state sync

- [ ] **Hybrid Connection**
  - Auto-detect local vs remote mode
  - Direct WebSocket for local (low latency)
  - Cloud relay for remote access
  - Seamless mode switching

- [ ] **Security**
  - HMAC-SHA256 signatures for local commands
  - Derived token for cloud authentication
  - Secure storage for secrets (Hive encrypted)

---

## Phase 7 - Code Signing & Security

### To Do
- [ ] **Windows Code Signing**
  - Obtain code signing certificate
  - Sign executables and installers
  - Eliminate SmartScreen warnings

- [ ] **macOS Code Signing & Notarization**
  - Obtain Apple Developer certificate
  - Sign app bundle
  - Notarize with Apple
  - Eliminate Gatekeeper warnings

- [ ] **Signed Update Manifests**
  - Generate signing keypair
  - Sign latest.json manifests
  - Configure updater pubkey verification

---

## Priorities

1. Finish Phase 1 (functional MVP)
2. Phase 2 (distribution)
3. Phase 3 (UI)
4. Phase 4 & 5 (cloud and bot - can be parallelized)
5. Phase 6 (mobile remote - requires Phase 4 cloud infrastructure for remote mode)
6. Phase 7 (code signing - when certificates are obtained)
