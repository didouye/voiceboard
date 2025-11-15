# Quick Reference Guide

## Essential Commands

### Build and Run
```powershell
# Build release version
cargo build --release

# Run with logging
$env:RUST_LOG="info"; cargo run --release

# Run in background
start /B target\release\voiceboard.exe
```

### Configuration
```toml
# config.toml
sample_rate = 48000
buffer_size = 512

[effects]
pitch_shift = -5.0        # -24 to +24 semitones
formant_shift = 3.0       # -24 to +24 semitones
reverb_enabled = true     # true/false
robot_enabled = false     # true/false
distortion = 0.3          # 0.0 to 1.0
```

## Quick Setup Steps

1. **Install Prerequisites**
   - Rust: https://rustup.rs/
   - Visual Studio Build Tools
   - VB-CABLE: https://vb-audio.com/Cable/

2. **Build VoiceBoard**
   ```powershell
   git clone https://github.com/didouye/voiceboard.git
   cd voiceboard
   cargo build --release
   ```

3. **Configure Windows Audio**
   - Default Playback: CABLE Input
   - Default Recording: Your microphone

4. **Configure Application**
   - Discord/Zoom/etc.: Input = CABLE Output

5. **Run**
   ```powershell
   cargo run --release
   ```

## Effect Presets

### Deep Voice
```toml
[effects]
pitch_shift = -7.0
formant_shift = -4.0
```

### High Voice
```toml
[effects]
pitch_shift = 8.0
formant_shift = 6.0
```

### Robot
```toml
[effects]
robot_enabled = true
reverb_enabled = true
distortion = 0.4
```

### Monster
```toml
[effects]
pitch_shift = -12.0
formant_shift = -8.0
distortion = 0.6
```

### Natural Enhancement
```toml
[effects]
pitch_shift = -1.0
reverb_enabled = true
distortion = 0.1
```

## Latency vs Buffer Size

| Buffer | Latency @48kHz | Use Case |
|--------|----------------|----------|
| 256    | ~21ms          | Gaming   |
| 512    | ~32ms          | General  |
| 1024   | ~57ms          | Stable   |
| 2048   | ~100ms         | Old PCs  |

## Common Issues

| Problem | Solution |
|---------|----------|
| No audio | Check CABLE Input is default playback |
| Crackling | Increase buffer_size |
| High CPU | Disable reverb, increase buffer |
| Can't hear | Enable "Listen to this device" |
| Echo | Disable duplicate monitoring |

## Virtual Cable Ports

**VB-CABLE:**
- Output to: "CABLE Input"
- Apps read from: "CABLE Output"

**VoiceMeeter:**
- Output to: "VoiceMeeter Input (VAIO)"
- Apps read from: "VoiceMeeter Output (VAIO)"

## Application Settings

### Discord
```
Settings → Voice & Video
Input Device: CABLE Output
Output Device: Your Headphones
```

### OBS
```
Add Source → Audio Input Capture
Device: CABLE Output
```

### Zoom
```
Settings → Audio
Microphone: CABLE Output
Speakers: Your Headphones
```

## Monitoring Setup

Enable listening to hear yourself:
```
Sound Settings → Recording → CABLE Output
Properties → Listen tab
☑ Listen to this device
Select: Your Headphones
```

## Environment Variables

```powershell
# Set log level
$env:RUST_LOG="debug"    # debug, info, warn, error, trace

# Run
cargo run --release
```

## File Locations

```
voiceboard/
├── config.toml          # Your configuration
├── target/
│   └── release/
│       └── voiceboard.exe    # Compiled binary
└── src/                 # Source code
```

## Parameter Ranges

| Parameter | Min | Max | Default | Units |
|-----------|-----|-----|---------|-------|
| sample_rate | 8000 | 96000 | 48000 | Hz |
| buffer_size | 64 | 8192 | 512 | samples |
| pitch_shift | -24 | +24 | 0 | semitones |
| formant_shift | -24 | +24 | 0 | semitones |
| distortion | 0.0 | 1.0 | 0.0 | ratio |

## Keyboard Shortcuts

Currently no runtime keyboard shortcuts. Future versions will support:
- Hotkeys for effect toggling
- Preset switching
- Mute/unmute

## Useful Links

- **VB-CABLE**: https://vb-audio.com/Cable/
- **VoiceMeeter**: https://vb-audio.com/Voicemeeter/
- **Rust**: https://rustup.rs/
- **VS Build Tools**: https://visualstudio.microsoft.com/downloads/

## Documentation

- [README.md](README.md) - Overview
- [BUILD.md](BUILD.md) - Build instructions
- [EXAMPLES.md](EXAMPLES.md) - Usage examples
- [VIRTUAL_AUDIO_SETUP.md](VIRTUAL_AUDIO_SETUP.md) - Audio setup
- [ARCHITECTURE.md](ARCHITECTURE.md) - Technical details
- [FAQ.md](FAQ.md) - Common questions

## Getting Help

1. Check [FAQ.md](FAQ.md)
2. Review documentation above
3. Check GitHub Issues
4. Open new issue with:
   - OS version
   - Error messages
   - Steps to reproduce

## Contributing

```powershell
# Fork repository
# Make changes
cargo fmt              # Format code
cargo clippy           # Lint
cargo test             # Test
# Submit pull request
```

## Version Info

```powershell
# Check versions
rustc --version
cargo --version
git --version
```

## Performance Tips

1. **Low Latency**: buffer_size = 256, fewer effects
2. **Stable**: buffer_size = 1024, all effects OK
3. **Low CPU**: Disable reverb, buffer_size = 2048
4. **Best Quality**: sample_rate = 48000, buffer_size = 1024

## Effect Combinations

**Masculine Voice:**
```toml
pitch_shift = -7.0
formant_shift = -4.0
reverb_enabled = false
```

**Feminine Voice:**
```toml
pitch_shift = 7.0
formant_shift = 5.0
reverb_enabled = false
```

**Scary:**
```toml
pitch_shift = -10.0
distortion = 0.7
reverb_enabled = true
```

**Announcer:**
```toml
pitch_shift = -3.0
formant_shift = -1.0
reverb_enabled = true
distortion = 0.05
```

## Troubleshooting Checklist

- [ ] VB-CABLE installed and rebooted
- [ ] CABLE Input is default playback
- [ ] Physical mic is default recording
- [ ] Application using CABLE Output
- [ ] VoiceBoard running
- [ ] config.toml present
- [ ] No compilation errors
- [ ] Audio permissions granted

## System Requirements

**Minimum:**
- Windows 10 64-bit
- Dual-core 2GHz CPU
- 4GB RAM
- Any microphone

**Recommended:**
- Windows 11 64-bit
- Quad-core 3GHz CPU
- 8GB RAM
- USB microphone
- Studio headphones

## Log Levels

```powershell
# No logs
cargo run --release

# Basic info
$env:RUST_LOG="info"; cargo run --release

# Detailed
$env:RUST_LOG="debug"; cargo run --release

# Everything
$env:RUST_LOG="trace"; cargo run --release
```

## Build Variants

```powershell
# Debug (fast compile, slow run)
cargo build

# Release (slow compile, fast run)
cargo build --release

# Check only (no binary)
cargo check

# With optimizations
cargo build --release
```

---

**Remember**: Always test your configuration before going live in important calls or streams!
