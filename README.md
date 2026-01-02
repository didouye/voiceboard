# Voiceboard

**Virtual Microphone Mixer** - Mix your microphone with audio files and output to a virtual microphone device.

![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## Features

- **Real-time Audio Mixing** - Mix your microphone with sound files in real-time
- **Soundboard with 12 Pads** - Trigger sounds with keyboard shortcuts (1-9, 0, -, =)
- **Multi-format Support** - MP3, OGG, WAV, FLAC
- **Sound Preview** - Preview sounds on your system speakers before playing
- **VU Meters** - Real-time audio level visualization
- **Auto-Update** - Automatic updates with cryptographic verification
- **Modern UI** - Dark theme with intuitive controls
- **Persistent Settings** - Your configuration is saved between sessions

## Download

Download the latest release for your platform:

| Platform | Installer | Portable |
|----------|-----------|----------|
| **Windows x64** | [.msi](https://github.com/didouye/voiceboard/releases/latest) | [.zip](https://github.com/didouye/voiceboard/releases/latest) |
| **macOS Apple Silicon** | [.dmg](https://github.com/didouye/voiceboard/releases/latest) | [.tar.gz](https://github.com/didouye/voiceboard/releases/latest) |
| **macOS Intel** | [.dmg](https://github.com/didouye/voiceboard/releases/latest) | [.tar.gz](https://github.com/didouye/voiceboard/releases/latest) |
| **Linux x64** | [.AppImage](https://github.com/didouye/voiceboard/releases/latest) | [.tar.gz](https://github.com/didouye/voiceboard/releases/latest) |

> **Note:** The app is not yet code-signed. On Windows, you may see a SmartScreen warning. On macOS, right-click and select "Open" to bypass Gatekeeper.

## Prerequisites

### Virtual Audio Driver (Required for virtual microphone output)

- **Windows**: [VB-Audio Virtual Cable](https://vb-audio.com/Cable/) or [Virtual Audio Driver](https://github.com/VirtualDrivers/Virtual-Audio-Driver)
- **macOS**: [BlackHole](https://existential.audio/blackhole/) (free) or [Loopback](https://rogueamoeba.com/loopback/)
- **Linux**: PulseAudio/PipeWire virtual sink (built-in)

## Usage

1. **Select Input Device** - Choose your microphone
2. **Select Output Device** - Choose the virtual audio device
3. **Add Sounds** - Click on pads to assign audio files
4. **Start Mixing** - Click the Start button to begin mixing
5. **Trigger Sounds** - Click pads or use keyboard shortcuts

In Discord/Zoom/etc., select the virtual audio device as your microphone input.

## Development

### Prerequisites

- [Rust](https://rustup.rs/) 1.70+
- [Node.js](https://nodejs.org/) 18+
- Platform-specific dependencies (see [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites))

### Setup

```bash
# Clone the repository
git clone https://github.com/didouye/voiceboard.git
cd voiceboard

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

### Testing

```bash
# Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Angular tests
npm test
```

## Architecture

This project follows **Hexagonal Architecture** (Ports & Adapters) with **DDD** principles:

```
src-tauri/src/
├── domain/          # Pure business logic (entities, value objects)
├── ports/           # Interfaces (traits)
├── adapters/        # Concrete implementations (CPAL, Rodio)
├── application/     # Use cases, Tauri commands
└── infrastructure/  # Cross-cutting concerns
```

## Tech Stack

### Desktop App

| Component | Technology |
|-----------|------------|
| Framework | Tauri 2.0 |
| Backend | Rust |
| Frontend | Angular 18+ |
| Audio I/O | cpal |
| Audio Decoding | Rodio |
| Async Runtime | Tokio |

### Future

- **Cloud Backend**: Django + PostgreSQL
- **Discord Bot**: Rust (serenity + songbird)
- **Mobile Remote**: Flutter

## Roadmap

See [ROADMAP.md](ROADMAP.md) for detailed progress and planned features.

**Current Status:**
- Phase 1 (Core MVP): 75% complete
- Phase 2 (Distribution): 80% complete

## Contributing

Contributions are welcome! Please read the contributing guidelines before submitting a PR.

## License

MIT License - see [LICENSE](LICENSE) file.
