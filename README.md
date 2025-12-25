# Voiceboard

Virtual Microphone Mixer for Windows - Mix real microphone audio with audio files (MP3, OGG) and output to a virtual microphone.

## Features

- **Virtual Microphone Output**: Create a virtual microphone that other applications can use
- **Real-time Audio Mixing**: Mix your microphone with audio files in real-time
- **Multi-channel Mixer**: Add multiple audio sources with individual volume controls
- **File Format Support**: MP3, OGG, WAV, FLAC
- **Modern UI**: Built with Angular and Tauri

## Architecture

This project follows **Hexagonal Architecture** (Ports & Adapters) with **DDD** principles:

```
src-tauri/src/
├── domain/          # Pure business logic (DDD entities, value objects)
│   ├── audio/       # Sample, Buffer, Format
│   ├── device/      # AudioDevice, DeviceId
│   └── mixer/       # MixerConfig, MixerChannel
├── ports/           # Interfaces (traits) - contracts with outside world
│   ├── audio_input.rs
│   ├── audio_output.rs
│   ├── file_decoder.rs
│   └── device_manager.rs
├── adapters/        # Concrete implementations
│   ├── cpal_input.rs         # CPAL audio input
│   ├── cpal_device_manager.rs
│   ├── rodio_decoder.rs      # Rodio file decoder
│   └── windows_virtual_output.rs
├── application/     # Use cases, Tauri commands
│   ├── commands.rs  # Tauri IPC commands
│   ├── services.rs  # Business orchestration
│   └── state.rs     # Application state
└── infrastructure/  # Cross-cutting concerns
    └── logging.rs
```

## Prerequisites

### Windows

- Windows 10 (Build 1903+) or Windows 11
- [Virtual Audio Driver](https://github.com/VirtualDrivers/Virtual-Audio-Driver) - Open source virtual audio device

### Development

- [Rust](https://rustup.rs/) 1.70+
- [Node.js](https://nodejs.org/) 18+
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

## Installation

1. Install the Virtual Audio Driver from [GitHub](https://github.com/VirtualDrivers/Virtual-Audio-Driver/releases)

2. Clone the repository:

```bash
git clone https://github.com/yourusername/voiceboard.git
cd voiceboard
```

3. Install dependencies:

```bash
npm install
```

4. Run in development mode:

```bash
npm run tauri dev
```

## Building

```bash
npm run tauri build
```

## Testing

### Rust tests

```bash
cd src-tauri
cargo test
```

### Angular tests

```bash
npm test
```

## Tech Stack

### Desktop App

#### Backend (Rust)

- **Tauri 2.0** - Desktop application framework
- **cpal** - Cross-platform audio I/O
- **rodio** - Audio playback and decoding
- **tokio** - Async runtime
- **thiserror/anyhow** - Error handling

#### Frontend (Angular)

- **Angular 18+** - UI framework
- **TypeScript** - Type-safe JavaScript
- **Angular Signals** - Reactive state management

### Cloud (Future)

- **Backend**: Django + Django REST Framework
- **Database**: PostgreSQL
- **Storage**: S3-compatible

### Discord Bot (Future)

- **Preferred**: Rust (serenity + songbird)
- **Fallback**: Python (discord.py)

### Architecture

- **Hexagonal Architecture** (Ports & Adapters)
- **Domain-Driven Design** (DDD)
- **Test-Driven Development** (TDD) - when writing new features

## Development Methodology

- **TDD** (Test-Driven Development) - Tests written first for domain logic
- **DDD** (Domain-Driven Design) - Rich domain model with entities and value objects
- **Hexagonal Architecture** - Separation of concerns with ports and adapters

## Project Structure

```
voiceboard/
├── src/                    # Angular frontend
│   └── app/
│       ├── core/           # Services, models
│       └── features/       # Feature modules
├── src-tauri/              # Rust backend
│   └── src/
│       ├── domain/         # Business logic (DDD)
│       ├── ports/          # Interfaces (traits)
│       ├── adapters/       # Implementations
│       ├── application/    # Commands, services, state
│       └── infrastructure/ # Cross-cutting concerns
└── ROADMAP.md              # Project roadmap and progress
```

## License

MIT License - see [LICENSE](LICENSE) file.
