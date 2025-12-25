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

### In Progress
- [ ] Sound preview on system output (listen before sending to virtual mic)

### To Do
- [ ] Virtual microphone output - Implement WASAPI to send audio to Virtual Audio Driver
- [ ] Level visualization (VU meters) - AudioEngine emits `LevelUpdate`, connect to UI
- [ ] Unit and integration tests
- [ ] Individual volume control per pad in UI

---

## Phase 2 - Distribution & CI/CD

### To Do
- [ ] **GitHub Actions CI**
  - Automated build and compilation
  - Automated tests
  - Application packaging

- [ ] **Windows Installer**
  - Evaluate open source solutions:
    - NSIS (Nullsoft Scriptable Install System)
    - WiX Toolset
    - Inno Setup
  - Create installer with:
    - **Bundled Virtual Audio Driver** (auto-install, no separate download)
    - Desktop/Start menu shortcuts
    - Clean uninstallation

- [ ] **Linux Support**
  - Virtual audio device (PulseAudio/PipeWire virtual sink)
  - Linux packaging (AppImage, .deb, .rpm)
  - Test on major distributions (Ubuntu, Fedora, Arch)

- [ ] **Auto-Update System**
  - Use tauri-plugin-updater
  - Update server (GitHub Releases or custom)
  - In-app update notifications
  - Background updates

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

## Priorities

1. Finish Phase 1 (functional MVP)
2. Phase 2 (distribution)
3. Phase 3 (UI)
4. Phase 4 & 5 (cloud and bot - can be parallelized)
