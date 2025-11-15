# Voiceboard Architecture

## Overview

Voiceboard is a Windows-only soundboard application that allows users to play sound effects through a virtual microphone, mixing them with their real microphone input. This enables use in applications like Discord, Zoom, OBS, etc.

## Tech Stack

- **Backend**: Rust
- **Desktop Framework**: Tauri 2.x
- **Frontend**: Angular 18+
- **Audio Processing**: Windows WASAPI
- **Virtual Audio Device**: Virtual Audio Cable approach with WASAPI loopback

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Angular Frontend (UI)                    │
│  ┌──────────────┬──────────────┬───────────────────────┐   │
│  │ Soundboard   │ Device       │ Audio Controls        │   │
│  │ Manager      │ Selector     │ (Play/Stop/Volume)    │   │
│  └──────────────┴──────────────┴───────────────────────┘   │
└─────────────────────────┬───────────────────────────────────┘
                          │ Tauri IPC (Commands/Events)
┌─────────────────────────┴───────────────────────────────────┐
│                   Tauri Application Layer                    │
│  ┌────────────────────────────────────────────────────────┐ │
│  │  Command Handlers (bridge Angular ↔ Rust)             │ │
│  │  - soundboard_commands                                 │ │
│  │  - device_commands                                     │ │
│  │  - audio_commands                                      │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────┬───────────────────────────────────┘
                          │ Direct function calls
┌─────────────────────────┴───────────────────────────────────┐
│                    Rust Backend (Core Logic)                 │
│                                                              │
│  ┌──────────────────┐  ┌──────────────────┐                │
│  │ Soundboard       │  │ Audio Device     │                │
│  │ Manager          │  │ Manager          │                │
│  │                  │  │                  │                │
│  │ - CRUD ops       │  │ - Enumerate      │                │
│  │ - Persistence    │  │ - Select input   │                │
│  │ - Sorting        │  │ - Monitor state  │                │
│  └──────────────────┘  └──────────────────┘                │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │            Audio Engine (Core)                        │  │
│  │                                                        │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │  │
│  │  │ Microphone   │  │ Sound File   │  │ Audio      │ │  │
│  │  │ Capture      │  │ Player       │  │ Mixer      │ │  │
│  │  │              │  │              │  │            │ │  │
│  │  │ (WASAPI)     │  │ (Symphonia)  │  │ (Custom)   │ │  │
│  │  └──────┬───────┘  └──────┬───────┘  └─────┬──────┘ │  │
│  │         │                 │                 │         │  │
│  │         └─────────────────┴────────────────►│         │  │
│  │                                              │         │  │
│  │                                    ┌─────────▼──────┐ │  │
│  │                                    │ Virtual Mic    │ │  │
│  │                                    │ Output         │ │  │
│  │                                    │ (WASAPI Loop)  │ │  │
│  │                                    └────────────────┘ │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │            Configuration Manager                      │  │
│  │  - Local storage (SQLite or JSON)                    │  │
│  │  - Settings persistence                              │  │
│  └──────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

## Layer Responsibilities

### Angular Frontend
- **Purpose**: User interface and interaction
- **Responsibilities**:
  - Display soundboard with sound list
  - Handle user interactions (add, delete, rename, reorder sounds)
  - Show audio device selection
  - Display playback controls and volume sliders
  - Provide drag-and-drop functionality
  - Show real-time audio levels

### Tauri Layer
- **Purpose**: Bridge between frontend and backend
- **Responsibilities**:
  - Expose Rust functions as Tauri commands
  - Handle IPC communication
  - Emit events to frontend (audio level updates, device changes)
  - Manage application lifecycle
  - Provide file system access for sound files

### Rust Backend
- **Purpose**: Core application logic and audio processing
- **Responsibilities**:
  - Audio device enumeration and selection
  - Real-time audio capture from microphone
  - Audio file decoding and playback
  - Audio mixing (mic + sound effects)
  - Virtual microphone implementation
  - Soundboard configuration persistence
  - Low-latency audio pipeline management

## Audio Pipeline

```
┌─────────────────┐
│ Real Microphone │
│   (WASAPI)      │
└────────┬────────┘
         │ PCM Audio Stream
         ▼
┌─────────────────┐      ┌──────────────────┐
│  Mic Capture    │      │  Sound Files     │
│  Thread         │      │  (.mp3, .wav,    │
│  (Ring Buffer)  │      │   .ogg, etc.)    │
└────────┬────────┘      └────────┬─────────┘
         │                        │
         │                        ▼
         │               ┌──────────────────┐
         │               │  Audio Decoder   │
         │               │  (Symphonia)     │
         │               └────────┬─────────┘
         │                        │ PCM Audio
         │                        ▼
         │               ┌──────────────────┐
         │               │  Playback Thread │
         │               │  (Ring Buffer)   │
         │               └────────┬─────────┘
         │                        │
         ▼                        ▼
┌──────────────────────────────────────────┐
│          Audio Mixer Thread               │
│  - Resample to common rate (48kHz)       │
│  - Mix multiple sources                   │
│  - Apply volume/gain controls             │
│  - Output to ring buffer                  │
└────────────────┬─────────────────────────┘
                 │ Mixed PCM Stream
                 ▼
┌──────────────────────────────────────────┐
│    Virtual Microphone Output Thread      │
│    (WASAPI Loopback or Virtual Cable)    │
└──────────────────────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────┐
│   Applications (Discord, Zoom, OBS...)   │
└──────────────────────────────────────────┘
```

## Virtual Microphone Implementation Strategy

### Option 1: WASAPI Loopback + Virtual Audio Cable (Recommended)
- Use existing virtual audio cable driver (VB-Audio Cable, VoiceMeeter)
- Rust app writes mixed audio to the virtual cable using WASAPI
- Other apps read from the virtual cable
- **Pros**: No driver development, stable, works with existing tools
- **Cons**: Requires user to install virtual audio cable software

### Option 2: Virtual Audio Device Driver (Advanced)
- Create a custom kernel-mode audio driver (WDM/AVStream)
- Rust app communicates with driver via IOCTL
- **Pros**: Complete control, no external dependencies
- **Cons**: Complex, requires driver signing, maintenance burden

**Recommendation**: Start with Option 1, document Option 2 for future enhancement

## Key Rust Crates

| Crate | Purpose |
|-------|---------|
| `tauri` | Desktop application framework |
| `cpal` | Cross-platform audio I/O (WASAPI wrapper) |
| `symphonia` | Audio decoding (mp3, ogg, flac, wav) |
| `rubato` | Audio resampling |
| `ringbuf` | Lock-free ring buffers for audio threads |
| `serde` | Serialization for IPC and config |
| `tokio` | Async runtime |
| `sqlx` or `sled` | Configuration persistence |
| `windows` | Windows-specific audio APIs |

## Data Flow

### Sound Playback Flow
1. User clicks sound in Angular UI
2. Angular calls Tauri command `play_sound(sound_id)`
3. Tauri command handler calls Rust `AudioEngine::play_sound()`
4. Audio engine:
   - Loads sound file from disk
   - Decodes to PCM
   - Pushes to mixer queue
   - Mixer combines with mic input
   - Outputs to virtual device
5. Audio level events emitted back to Angular for visualization

### Device Selection Flow
1. User selects microphone in Angular UI
2. Angular calls `select_input_device(device_id)`
3. Rust stops current capture thread
4. Starts new capture thread with selected device
5. Updates configuration
6. Emits device change event to UI

## Configuration Storage

**Format**: SQLite database with the following tables:

### `sounds` table
```sql
CREATE TABLE sounds (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    volume REAL DEFAULT 1.0,
    sort_order INTEGER,
    created_at INTEGER,
    updated_at INTEGER
);
```

### `settings` table
```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

Settings stored:
- `selected_input_device_id`
- `selected_output_device_id`
- `master_volume`
- `mic_volume`
- `effects_volume`

## API Interface Specification

### Angular → Tauri Commands

```typescript
// Soundboard commands
invoke('get_sounds'): Promise<Sound[]>
invoke('add_sound', { filePath: string, name: string }): Promise<Sound>
invoke('delete_sound', { id: string }): Promise<void>
invoke('rename_sound', { id: string, name: string }): Promise<void>
invoke('reorder_sounds', { ids: string[] }): Promise<void>
invoke('update_sound_volume', { id: string, volume: number }): Promise<void>

// Device commands
invoke('get_input_devices'): Promise<AudioDevice[]>
invoke('get_output_devices'): Promise<AudioDevice[]>
invoke('select_input_device', { id: string }): Promise<void>

// Audio commands
invoke('play_sound', { id: string }): Promise<void>
invoke('stop_sound', { id: string }): Promise<void>
invoke('stop_all_sounds'): Promise<void>
invoke('set_master_volume', { volume: number }): Promise<void>
invoke('set_mic_volume', { volume: number }): Promise<void>
invoke('set_effects_volume', { volume: number }): Promise<void>

// Audio engine
invoke('start_audio_engine'): Promise<void>
invoke('stop_audio_engine'): Promise<void>
```

### Tauri → Angular Events

```typescript
listen('audio-level', (event) => {
  // { micLevel: number, outputLevel: number }
})

listen('device-changed', (event) => {
  // { deviceId: string, deviceType: 'input' | 'output' }
})

listen('sound-playback-started', (event) => {
  // { soundId: string }
})

listen('sound-playback-stopped', (event) => {
  // { soundId: string }
})

listen('error', (event) => {
  // { message: string, details: string }
})
```

## Performance Considerations

### Low Latency Requirements
- Target latency: < 20ms total
- Use ring buffers for audio thread communication
- Audio processing in dedicated real-time threads
- Avoid allocations in audio callback path
- Use lock-free data structures

### Audio Settings
- Sample rate: 48,000 Hz (standard for WASAPI)
- Bit depth: 32-bit float
- Channels: 2 (stereo)
- Buffer size: 480 samples (10ms at 48kHz)

## Security Considerations

1. **File Access**: Validate all file paths to prevent directory traversal
2. **Input Validation**: Sanitize all user inputs from Angular
3. **Resource Limits**: Limit number of sounds, file sizes
4. **Sandboxing**: Leverage Tauri's security features
5. **Updates**: Use Tauri's updater for secure updates

## Development Phases

### Phase 1: Bootstrap (Current)
- Project structure
- Basic Tauri + Angular setup
- Documentation

### Phase 2: Core Audio
- Device enumeration
- Microphone capture
- Basic playback
- Simple mixing

### Phase 3: Soundboard
- CRUD operations
- Persistence
- File management
- UI implementation

### Phase 4: Advanced Features
- Drag and drop reordering
- Advanced mixing controls
- Audio visualization
- Hotkeys

### Phase 5: Polish
- Error handling
- Performance optimization
- User settings
- Installer creation

## Build and Deployment

### Development Build
```bash
cd src-ui
npm install
npm run build
cd ..
cargo tauri dev
```

### Production Build
```bash
cd src-ui
npm run build
cd ..
cargo tauri build
```

### Distribution
- MSI installer for Windows
- NSIS installer option
- Auto-update via Tauri updater
- Code signing for Windows SmartScreen

## Future Enhancements

1. Custom hotkeys for sound triggering
2. Sound categories/folders
3. Audio effects (reverb, pitch shift)
4. Cloud sync of soundboards
5. VST plugin support
6. Streaming integration (Twitch, YouTube)
