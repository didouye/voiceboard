# Voiceboard Project Structure

## Overview

This document describes the complete folder structure of the Voiceboard application.

```
voiceboard/
├── src-tauri/                    # Rust backend and Tauri application
│   ├── src/
│   │   ├── main.rs              # Application entry point
│   │   ├── lib.rs               # Library root (re-exports modules)
│   │   │
│   │   ├── audio/               # Audio processing core
│   │   │   ├── mod.rs           # Module exports
│   │   │   ├── engine.rs        # Main audio engine orchestrator
│   │   │   ├── capture.rs       # Microphone capture (WASAPI)
│   │   │   ├── playback.rs      # Sound file playback
│   │   │   ├── mixer.rs         # Audio mixing logic
│   │   │   ├── decoder.rs       # Audio file decoding (Symphonia)
│   │   │   ├── resampler.rs     # Sample rate conversion
│   │   │   ├── virtual_device.rs # Virtual microphone output
│   │   │   └── types.rs         # Audio-related type definitions
│   │   │
│   │   ├── soundboard/          # Soundboard management
│   │   │   ├── mod.rs           # Module exports
│   │   │   ├── manager.rs       # Soundboard CRUD operations
│   │   │   ├── sound.rs         # Sound entity and operations
│   │   │   └── storage.rs       # Persistence layer (SQLite)
│   │   │
│   │   ├── devices/             # Audio device management
│   │   │   ├── mod.rs           # Module exports
│   │   │   ├── manager.rs       # Device enumeration and selection
│   │   │   ├── monitor.rs       # Device change monitoring
│   │   │   └── types.rs         # Device-related types
│   │   │
│   │   ├── config/              # Configuration management
│   │   │   ├── mod.rs           # Module exports
│   │   │   ├── manager.rs       # Settings CRUD
│   │   │   ├── storage.rs       # SQLite storage implementation
│   │   │   └── types.rs         # Configuration types
│   │   │
│   │   ├── commands/            # Tauri command handlers
│   │   │   ├── mod.rs           # Module exports and command registration
│   │   │   ├── soundboard.rs    # Soundboard-related commands
│   │   │   ├── devices.rs       # Device-related commands
│   │   │   ├── audio.rs         # Audio control commands
│   │   │   └── config.rs        # Configuration commands
│   │   │
│   │   ├── events.rs            # Tauri event emission helpers
│   │   ├── error.rs             # Error types and handling
│   │   └── utils.rs             # Utility functions
│   │
│   ├── Cargo.toml               # Rust dependencies
│   ├── tauri.conf.json          # Tauri configuration
│   ├── build.rs                 # Build script
│   └── icons/                   # Application icons
│
├── src-ui/                      # Angular frontend
│   ├── src/
│   │   ├── app/
│   │   │   ├── components/
│   │   │   │   ├── soundboard/
│   │   │   │   │   ├── soundboard.component.ts
│   │   │   │   │   ├── soundboard.component.html
│   │   │   │   │   ├── soundboard.component.scss
│   │   │   │   │   ├── sound-item/
│   │   │   │   │   │   ├── sound-item.component.ts
│   │   │   │   │   │   ├── sound-item.component.html
│   │   │   │   │   │   └── sound-item.component.scss
│   │   │   │   │   └── sound-list/
│   │   │   │   │       ├── sound-list.component.ts
│   │   │   │   │       ├── sound-list.component.html
│   │   │   │   │       └── sound-list.component.scss
│   │   │   │   │
│   │   │   │   ├── device-selector/
│   │   │   │   │   ├── device-selector.component.ts
│   │   │   │   │   ├── device-selector.component.html
│   │   │   │   │   └── device-selector.component.scss
│   │   │   │   │
│   │   │   │   └── audio-controls/
│   │   │   │       ├── audio-controls.component.ts
│   │   │   │       ├── audio-controls.component.html
│   │   │   │       ├── audio-controls.component.scss
│   │   │   │       ├── volume-slider/
│   │   │   │       │   ├── volume-slider.component.ts
│   │   │   │       │   ├── volume-slider.component.html
│   │   │   │       │   └── volume-slider.component.scss
│   │   │   │       └── audio-visualizer/
│   │   │   │           ├── audio-visualizer.component.ts
│   │   │   │           ├── audio-visualizer.component.html
│   │   │   │           └── audio-visualizer.component.scss
│   │   │   │
│   │   │   ├── services/
│   │   │   │   ├── soundboard.service.ts      # Soundboard operations
│   │   │   │   ├── audio-device.service.ts    # Device management
│   │   │   │   ├── audio-engine.service.ts    # Audio playback control
│   │   │   │   ├── tauri.service.ts           # Tauri IPC wrapper
│   │   │   │   └── event.service.ts           # Event listening
│   │   │   │
│   │   │   ├── models/
│   │   │   │   ├── sound.model.ts             # Sound interface
│   │   │   │   ├── audio-device.model.ts      # Audio device interface
│   │   │   │   ├── audio-level.model.ts       # Audio level interface
│   │   │   │   └── settings.model.ts          # Settings interface
│   │   │   │
│   │   │   ├── app.component.ts
│   │   │   ├── app.component.html
│   │   │   ├── app.component.scss
│   │   │   ├── app.config.ts                  # Angular app configuration
│   │   │   └── app.routes.ts                  # Routing configuration
│   │   │
│   │   ├── assets/
│   │   │   ├── icons/                         # UI icons
│   │   │   └── styles/
│   │   │       └── variables.scss             # SCSS variables
│   │   │
│   │   ├── index.html
│   │   ├── main.ts
│   │   └── styles.scss
│   │
│   ├── angular.json
│   ├── package.json
│   ├── tsconfig.json
│   ├── tsconfig.app.json
│   └── tailwind.config.js                     # Tailwind CSS configuration
│
├── docs/                                       # Documentation
│   ├── API.md                                 # API reference
│   ├── AUDIO_PIPELINE.md                      # Audio implementation details
│   ├── VIRTUAL_DEVICE.md                      # Virtual device setup guide
│   └── DEVELOPMENT.md                         # Development guide
│
├── scripts/                                    # Build and utility scripts
│   ├── setup-dev.ps1                          # Development environment setup
│   └── install-virtual-cable.ps1              # Virtual audio cable installer
│
├── .gitignore
├── README.md                                   # Project overview and quickstart
├── ARCHITECTURE.md                             # Architecture documentation
├── PROJECT_STRUCTURE.md                        # This file
└── LICENSE
```

## Module Responsibilities

### Rust Backend (`src-tauri/src/`)

#### `audio/` - Audio Processing Core
- **engine.rs**: Orchestrates the entire audio pipeline, manages threads, coordinates components
- **capture.rs**: Captures audio from physical microphone using WASAPI
- **playback.rs**: Manages playback of individual sound files
- **mixer.rs**: Mixes microphone input with sound effects in real-time
- **decoder.rs**: Decodes various audio formats (MP3, WAV, OGG, FLAC) using Symphonia
- **resampler.rs**: Converts audio between different sample rates
- **virtual_device.rs**: Outputs mixed audio to virtual microphone device
- **types.rs**: Common audio types (AudioBuffer, AudioFormat, etc.)

#### `soundboard/` - Soundboard Management
- **manager.rs**: High-level soundboard operations (add, delete, rename, reorder)
- **sound.rs**: Sound entity definition and individual sound operations
- **storage.rs**: SQLite persistence for soundboard configuration

#### `devices/` - Audio Device Management
- **manager.rs**: Enumerate and select audio devices using CPAL/WASAPI
- **monitor.rs**: Watch for device connection/disconnection events
- **types.rs**: Device-related type definitions

#### `config/` - Configuration Management
- **manager.rs**: Application settings management
- **storage.rs**: SQLite-based configuration storage
- **types.rs**: Settings types and defaults

#### `commands/` - Tauri IPC Commands
- **soundboard.rs**: Handlers for soundboard commands from Angular
- **devices.rs**: Handlers for device-related commands
- **audio.rs**: Handlers for audio playback and control commands
- **config.rs**: Handlers for configuration commands
- **mod.rs**: Command registration with Tauri

### Angular Frontend (`src-ui/src/app/`)

#### `components/` - UI Components

**soundboard/**
- Main soundboard container component
- Handles sound list display, filtering, sorting
- Manages drag-and-drop reordering

**device-selector/**
- Audio device selection dropdown
- Shows available input/output devices
- Indicates currently selected device

**audio-controls/**
- Master playback controls
- Volume sliders for mic, effects, and master
- Audio level visualization
- Play/stop all functionality

#### `services/` - Business Logic

**soundboard.service.ts**
- Communicates with Rust backend for soundboard operations
- Maintains local state cache
- Handles file selection for adding sounds

**audio-device.service.ts**
- Fetches available audio devices
- Manages device selection
- Listens for device change events

**audio-engine.service.ts**
- Controls audio playback
- Manages volume levels
- Handles audio engine start/stop

**tauri.service.ts**
- Wrapper around Tauri's invoke and listen APIs
- Provides typed command interface
- Error handling

**event.service.ts**
- Manages event subscriptions
- Distributes events to components
- Cleanup on component destruction

#### `models/` - TypeScript Interfaces
- Type definitions matching Rust backend structures
- Ensures type safety across IPC boundary

## Key Files

### Configuration Files

| File | Purpose |
|------|---------|
| `src-tauri/Cargo.toml` | Rust dependencies and project metadata |
| `src-tauri/tauri.conf.json` | Tauri app configuration, permissions, build settings |
| `src-ui/package.json` | Angular dependencies and build scripts |
| `src-ui/angular.json` | Angular CLI configuration |
| `src-ui/tsconfig.json` | TypeScript compiler options |
| `src-ui/tailwind.config.js` | Tailwind CSS configuration |

### Entry Points

| File | Purpose |
|------|---------|
| `src-tauri/src/main.rs` | Rust application entry point, Tauri setup |
| `src-ui/src/main.ts` | Angular application bootstrap |
| `src-ui/src/index.html` | HTML entry point |

## Data Flow Patterns

### Command Flow (Angular → Rust)
```
Component → Service → Tauri Service → invoke() → Tauri Command Handler → Rust Module → Response → Service → Component
```

### Event Flow (Rust → Angular)
```
Rust Module → Event Emitter → Tauri Event → listen() → Event Service → Observable → Component
```

### Audio Flow
```
Microphone → Capture Thread → Ring Buffer → Mixer Thread ← Sound Player Thread ← Decoder
                                                    ↓
                                          Ring Buffer
                                                    ↓
                                          Virtual Device Output Thread
```

## Build Outputs

### Development
- `src-tauri/target/debug/` - Rust debug binaries
- `src-ui/dist/` - Angular build output
- `src-tauri/target/debug/voiceboard.exe` - Development executable

### Production
- `src-tauri/target/release/` - Rust release binaries
- `src-tauri/target/release/bundle/` - Installer packages
  - `msi/` - Windows MSI installer
  - `nsis/` - NSIS installer

## Database Schema

Location: `%APPDATA%/voiceboard/voiceboard.db`

### Tables
- `sounds` - Soundboard sound entries
- `settings` - Application settings (key-value pairs)
- `metadata` - Database version and migration info

## Important Paths

### Runtime Data
- **Windows**: `%APPDATA%/voiceboard/`
  - Database: `voiceboard.db`
  - Sound files: `sounds/` (optional, sounds can be anywhere)
  - Logs: `logs/`

### Development
- Rust target directory: `src-tauri/target/`
- Angular build output: `src-ui/dist/`
- Node modules: `src-ui/node_modules/`

## Next Steps

1. Initialize Cargo workspace in `src-tauri/`
2. Initialize Angular project in `src-ui/`
3. Configure Tauri to serve Angular build
4. Implement core modules according to architecture
5. Wire up IPC commands and events
6. Build and test on Windows
