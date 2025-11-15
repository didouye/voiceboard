# Voiceboard

A Windows soundboard application that allows you to play sound effects through a virtual microphone, mixing them with your real microphone input. Perfect for Discord, Zoom, OBS, and other streaming/communication applications.

## Features

- **Sound Management**: Add, delete, rename, and reorder sound effects
- **Drag & Drop**: Intuitive sound organization with drag-and-drop reordering
- **Microphone Selection**: Choose from all available system audio input devices
- **Virtual Microphone**: Create a virtual audio device that mixes your mic with sound effects
- **Low Latency**: Real-time audio processing with minimal delay (< 20ms)
- **Persistent Configuration**: Your soundboard setup is saved automatically
- **Modern UI**: Built with Angular and Tailwind CSS for a clean, responsive interface

## Tech Stack

- **Backend**: Rust (audio processing, device management)
- **Desktop Framework**: Tauri 2.x
- **Frontend**: Angular 18+
- **Audio**: WASAPI (Windows Audio Session API)

## Architecture

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed architecture documentation.

See [PROJECT_STRUCTURE.md](./PROJECT_STRUCTURE.md) for complete folder structure.

## Prerequisites

### Required

- **Windows 10/11** (64-bit)
- **Node.js** 18+ and npm
- **Rust** 1.75+ (install via [rustup](https://rustup.rs/))
- **Visual Studio Build Tools** (for Windows development)
  - Install via: https://visualstudio.microsoft.com/downloads/
  - Select "Desktop development with C++" workload

### Recommended

- **Virtual Audio Cable** (for virtual microphone functionality)
  - [VB-Audio Virtual Cable](https://vb-audio.com/Cable/) (free)
  - [VoiceMeeter](https://vb-audio.com/Voicemeeter/) (free, more features)
  - Install before running the application

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/voiceboard.git
cd voiceboard
```

### 2. Install Dependencies

#### Rust Dependencies
```bash
cd src-tauri
cargo build
cd ..
```

#### Angular Dependencies
```bash
cd src-ui
npm install
cd ..
```

### 3. Run Development Build

```bash
# From project root
cd src-ui
npm run tauri dev
```

This will:
- Build the Angular frontend
- Start the Tauri application
- Open the Voiceboard window

### 4. Build for Production

```bash
# From project root
cd src-ui
npm run tauri build
```

The installer will be created in `src-tauri/target/release/bundle/`

## Setup Virtual Audio Device

For the virtual microphone feature to work, you need a virtual audio cable:

### Option 1: VB-Audio Virtual Cable (Recommended)

1. Download from: https://vb-audio.com/Cable/
2. Run the installer as Administrator
3. Restart your computer
4. The virtual cable will appear as "CABLE Input" and "CABLE Output" in your audio devices

### Option 2: VoiceMeeter

1. Download from: https://vb-audio.com/Voicemeeter/
2. Install and run VoiceMeeter
3. Configure VoiceMeeter as needed
4. Use VoiceMeeter Input as your virtual microphone in other apps

## Usage

### Adding Sounds

1. Click the "Add Sound" button
2. Select an audio file (MP3, WAV, OGG, FLAC)
3. Give it a name
4. Click to play!

### Selecting Your Microphone

1. Open the device selector dropdown
2. Choose your physical microphone
3. The app will start capturing from this device

### Using in Other Applications

1. Ensure virtual audio cable is installed
2. In Voiceboard, the mixed audio will be sent to the virtual cable
3. In Discord/Zoom/OBS, select the virtual cable as your microphone:
   - For VB-Cable: Select "CABLE Output" as your microphone
   - For VoiceMeeter: Select "VoiceMeeter Output" as your microphone

### Volume Controls

- **Mic Volume**: Control your microphone input level
- **Effects Volume**: Control sound effects volume
- **Master Volume**: Control overall output volume

## Configuration

Configuration is stored in:
- **Windows**: `%APPDATA%/voiceboard/voiceboard.db`

You can reset configuration by deleting this file.

## Development Guide

See [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md) for detailed development instructions.

### Project Structure

```
voiceboard/
├── src-tauri/          # Rust backend + Tauri
│   └── src/
│       ├── audio/      # Audio processing
│       ├── soundboard/ # Sound management
│       ├── devices/    # Device management
│       ├── config/     # Configuration
│       └── commands/   # Tauri commands
├── src-ui/             # Angular frontend
│   └── src/app/
│       ├── components/ # UI components
│       ├── services/   # Business logic
│       └── models/     # TypeScript types
└── docs/               # Documentation
```

### Key Commands

```bash
# Development
cd src-ui && npm run tauri dev

# Build for production
cd src-ui && npm run tauri build

# Run Rust tests
cd src-tauri && cargo test

# Run Angular tests
cd src-ui && npm run test

# Format Rust code
cd src-tauri && cargo fmt

# Lint Angular code
cd src-ui && npm run lint
```

## API Reference

See [docs/API.md](./docs/API.md) for complete API documentation.

### Tauri Commands (Angular → Rust)

```typescript
// Soundboard
invoke('get_sounds')
invoke('add_sound', { filePath, name })
invoke('delete_sound', { id })
invoke('play_sound', { id })

// Devices
invoke('get_input_devices')
invoke('select_input_device', { id })

// Audio
invoke('set_master_volume', { volume })
invoke('start_audio_engine')
```

### Events (Rust → Angular)

```typescript
listen('audio-level', (event) => { ... })
listen('device-changed', (event) => { ... })
listen('sound-playback-started', (event) => { ... })
```

## Troubleshooting

### Audio Not Working

1. **Check virtual audio cable installation**
   - Verify CABLE devices appear in Windows Sound settings
   - Try reinstalling the virtual cable

2. **Check device selection**
   - Ensure you've selected your physical microphone in Voiceboard
   - Verify the virtual cable is selected in your target app (Discord, etc.)

3. **Check Windows permissions**
   - Grant microphone permission to Voiceboard in Windows Settings
   - Privacy → Microphone → Allow apps to access your microphone

### Sounds Not Playing

1. **Check file format**
   - Supported: MP3, WAV, OGG, FLAC
   - Try converting to WAV if issues persist

2. **Check volume levels**
   - Ensure Effects Volume and Master Volume are not muted
   - Check individual sound volume settings

### High Latency

1. **Check buffer size** (in config)
   - Default: 480 samples (10ms)
   - Lower values = less latency but more CPU usage

2. **Close other audio applications**
   - Other apps may be using the audio device exclusively

3. **Update audio drivers**
   - Ensure you have the latest audio drivers installed

## Performance Tips

- Close unnecessary applications while using Voiceboard
- Use WAV files for best performance (no decoding overhead)
- Keep sound files under 10MB for optimal load times
- Limit soundboard to 50-100 sounds for best UI performance

## Known Limitations

- Windows only (uses WASAPI)
- Requires virtual audio cable software
- Maximum 2 channels (stereo)
- Sound files must be on local disk (no network paths)

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

See [docs/DEVELOPMENT.md](./docs/DEVELOPMENT.md) for development setup.

## License

MIT License - see [LICENSE](./LICENSE) for details

## Acknowledgments

- [Tauri](https://tauri.app/) - Desktop application framework
- [cpal](https://github.com/RustAudio/cpal) - Cross-platform audio I/O
- [Symphonia](https://github.com/pdeljanov/Symphonia) - Audio decoding
- [VB-Audio](https://vb-audio.com/) - Virtual audio cable software
- [Angular](https://angular.io/) - Frontend framework

## Support

- **Issues**: https://github.com/yourusername/voiceboard/issues
- **Discussions**: https://github.com/yourusername/voiceboard/discussions
- **Email**: support@example.com

## Roadmap

- [ ] Custom hotkeys for sound triggering
- [ ] Sound categories/folders
- [ ] Audio effects (reverb, pitch shift, echo)
- [ ] Cloud sync of soundboards
- [ ] VST plugin support
- [ ] Streaming integration (Twitch, YouTube)
- [ ] Mobile companion app
- [ ] Soundboard sharing/marketplace

## Screenshots

(Screenshots will be added once UI is implemented)

---

**Made with ❤️ using Rust, Tauri, and Angular**
