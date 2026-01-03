# Voiceboard Roadmap

> **Last updated:** January 2026

## Phase 1 - Core Application (MVP) - 95% Complete

### Done
- [x] Hexagonal architecture (DDD, Ports & Adapters)
- [x] AudioEngine with real-time mixing (microphone + sounds)
- [x] Audio device management (input/output) via CPAL
- [x] Audio file decoding (MP3, OGG, WAV, FLAC) via Rodio
- [x] Mic volume/master volume and mic mute
- [x] Settings and soundboard persistence
- [x] Soundboard with 12 pads and keyboard shortcuts (1-9, 0, -, =)
- [x] Device selector (input/output/preview)
- [x] Master control with volume and start/stop button
- [x] Modern UI with dark theme
- [x] Sound preview on system output with device selection
- [x] Level visualization (VU meters) - Real-time audio levels display
- [x] Virtual microphone output - Send mixed audio to VB-Cable
- [x] **VB-Audio Virtual Cable Setup** (Windows)
  - Detect if VB-Cable is installed on startup
  - Download VB-Cable installer from official website if not present
  - Launch VB-Cable installer with UAC elevation
  - Setup wizard guides user through installation

### To Do
- [x] Mic monitoring on preview output - Hear your own microphone in preview
- [ ] Unit and integration tests - Increase test coverage
- [ ] Individual volume control per pad in UI
- [x] Bulk import - Import multiple audio files at once (with drag & drop and dynamic pad rows)
- [ ] **Speed Playback**
  - Button on each pad (next to preview button)
  - Play sound at different speeds (accelerated/slowed)
  - Outputs to both virtual output and monitoring
  - Speed options: 0.5x, 0.75x, 1x, 1.25x, 1.5x, 2x

---

## Phase 2 - Distribution & CI/CD - 90% Complete

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
  - Cryptographic signing for update verification (macOS/Windows)

- [x] **Debug Console**
  - Debug console UI (accessible via menu or keyboard shortcut)
  - Toggle debug mode from application menu
  - Display backend audio engine logs in real-time
  - Log export functionality

### In Progress
- [ ] **Error Tracking (Sentry)**
  - Integrate Sentry SDK in Rust backend
  - Integrate Sentry SDK in Angular frontend
  - Capture panics and errors
  - Source maps for frontend errors
  - Release tracking with version tags

### To Do
- [ ] **GitHub Pages Landing Page**
  - Product presentation with features overview
  - Download section with latest release
  - OS detection to highlight matching download (Windows/macOS/Linux)
  - Tailwind CSS for styling
  - Responsive design (mobile-friendly)
  - Auto-update download links from GitHub Releases API

- [ ] **Windows Installer Improvements**
  - Bundled Virtual Audio Driver

- [ ] **Linux Improvements**
  - Fix update signature for Linux
  - Virtual audio device setup guide (PulseAudio/PipeWire)

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

- [ ] **Sound Organization**
  - Folders/categories to organize sounds
  - Drag & drop sounds into folders
  - Folder navigation in UI

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

## Phase 7 - Code Signing & Security - 33% Complete

### Done
- [x] **Signed Update Manifests**
  - Generated signing keypair
  - Sign artifacts during CI build
  - Signatures included in latest.json
  - Updater pubkey verification configured

### To Do
- [ ] **Windows Code Signing**
  - Obtain code signing certificate (EV recommended)
  - Sign executables and installers
  - Eliminate SmartScreen warnings

- [ ] **macOS Code Signing & Notarization**
  - Obtain Apple Developer certificate ($99/year)
  - Sign app bundle
  - Notarize with Apple
  - Eliminate Gatekeeper warnings

---

## Phase 8 - Improvements & Compatibility

### Audio Engine
- [ ] **Sample Rate Resampling**
  - Support microphones with different sample rates (44.1kHz, 48kHz, 96kHz)
  - Automatic resampling when input and output devices have different sample rates
  - Currently requires matching sample rates between input device and VB-Cable

### Quality of Life
- [ ] **Launch at Startup**
  - Option to start Voiceboard automatically when Windows/macOS boots
  - Configurable from application settings
  - Start minimized to system tray option

- [ ] **Minimize to System Tray**
  - Close button minimizes to system tray instead of quitting
  - Tray icon with context menu (Show/Hide, Quit)
  - Double-click tray icon to restore window

- [ ] **Error Recovery**
  - Automatic reconnection when audio devices are disconnected/reconnected
  - Graceful handling of device changes during mixing

- [ ] **Performance Optimizations**
  - Reduce CPU usage during idle
  - Optimize ring buffer size based on latency requirements

### Advanced Features
- [ ] **Team Synchronized Sound Playback**
  - Alternative to sending sounds through virtual microphone
  - Avoids issues with noise/echo cancellation systems (Krisp, Discord, etc.)
  - Desktop sends command to cloud server instead of audio
  - Server relays command to all team members
  - Sound plays locally on each member's device
  - Benefits: better audio quality (no voice codec compression), bypasses noise gates

- [ ] **Noise & Echo Cancellation**
  - Implement noise reduction similar to Krisp
  - Research open-source alternatives (RNNoise, Speex, etc.)
  - Real-time processing in audio pipeline
  - Configurable sensitivity/aggressiveness

---

## Priorities

1. Finish Phase 1 (functional MVP)
2. Phase 2 (distribution)
3. Phase 3 (UI)
4. Phase 4 & 5 (cloud and bot - can be parallelized)
5. Phase 6 (mobile remote - requires Phase 4 cloud infrastructure for remote mode)
6. Phase 7 (code signing - when certificates are obtained)
7. Phase 8 (improvements - ongoing)
