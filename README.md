# VoiceBoard

A high-performance, real-time voice changer for Windows built in Rust. Similar to Voicemod, VoiceBoard captures microphone input, applies DSP effects (pitch shifting, formant shifting, reverb, robot voice, distortion), and outputs through a virtual audio device with optimized low-latency processing.

## Features

- **Real-time Audio Processing**: Low-latency audio pipeline using WASAPI
- **DSP Effects**:
  - Pitch Shifting (phase vocoder algorithm)
  - Formant Shifting (spectral envelope manipulation)
  - Reverb (Schroeder reverberator)
  - Robot Effect (ring modulation)
  - Distortion (wave shaping with soft clipping)
- **Windows Audio Integration**: WASAPI for professional-grade audio I/O
- **Configurable**: JSON/TOML configuration files
- **Optimized**: Release builds with LTO and optimizations for minimal latency

## Architecture

### High-Level Architecture

```
┌─────────────┐      ┌──────────────┐      ┌─────────────┐      ┌──────────────┐
│ Microphone  │─────>│ Audio        │─────>│ DSP Effect  │─────>│ Virtual      │
│ (WASAPI)    │      │ Capture      │      │ Chain       │      │ Audio Device │
└─────────────┘      └──────────────┘      └─────────────┘      └──────────────┘
                            │                     │                      │
                            │                     │                      │
                            v                     v                      v
                     ┌────────────────────────────────────────────────────┐
                     │          Low-Latency Processing Pipeline           │
                     │  - Buffer Size: 512 samples (10.67ms @ 48kHz)     │
                     │  - Sample Rate: 48000 Hz                          │
                     │  - Format: 32-bit Float PCM                       │
                     └────────────────────────────────────────────────────┘
```

### Project Structure

```
voiceboard/
├── src/
│   ├── main.rs              # Application entry point
│   ├── audio/               # Windows WASAPI audio I/O
│   │   ├── mod.rs           # Audio module interface
│   │   ├── capture.rs       # Microphone input capture
│   │   └── renderer.rs      # Audio output rendering
│   ├── dsp/                 # Digital Signal Processing
│   │   ├── mod.rs           # Effect chain manager
│   │   ├── pitch_shifter.rs # Phase vocoder pitch shifting
│   │   ├── formant_shifter.rs # Formant shifting
│   │   ├── reverb.rs        # Schroeder reverberator
│   │   ├── robot_effect.rs  # Ring modulation robot voice
│   │   └── distortion.rs    # Wave shaping distortion
│   └── config/              # Configuration management
│       └── mod.rs           # Config loading/saving
├── Cargo.toml               # Project dependencies
├── config.toml              # Runtime configuration
└── README.md                # This file
```

### DSP Effect Chain

Each effect is processed in sequence for optimal sound quality:

1. **Pitch Shifting** - Changes the pitch while preserving timing
2. **Formant Shifting** - Modifies vocal characteristics (gender, age)
3. **Robot Effect** - Adds synthetic/robotic quality via ring modulation
4. **Distortion** - Adds grit and harmonic content
5. **Reverb** - Adds spatial depth (applied last for natural sound)

## Installation

### Prerequisites

- Windows 10/11 (64-bit)
- Rust toolchain (1.70+): https://rustup.rs/
- Visual Studio Build Tools or MSVC compiler
- Virtual Audio Cable (see Virtual Audio Device Setup below)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/didouye/voiceboard.git
cd voiceboard

# Build release version (optimized for low latency)
cargo build --release

# Run the application
cargo run --release
```

## Virtual Audio Device Setup

To route the processed audio to other applications, you need a virtual audio cable:

### Option 1: VB-CABLE (Free)

1. Download VB-CABLE from: https://vb-audio.com/Cable/
2. Extract and run `VBCABLE_Setup_x64.exe` as Administrator
3. Install the driver
4. Reboot your computer
5. VB-CABLE will appear as "CABLE Input" and "CABLE Output" in Windows sound settings

### Option 2: VoiceMeeter (Free, More Features)

1. Download VoiceMeeter from: https://vb-audio.com/Voicemeeter/
2. Install and configure virtual inputs/outputs
3. More flexible routing options

### Configuring VoiceBoard to Use Virtual Audio

After installation, VoiceBoard will output to your default audio device. To route to a virtual cable:

1. Open Windows Sound Settings
2. Set "CABLE Input" as your default playback device
3. Applications will now receive audio from "CABLE Output"
4. For monitoring, enable "Listen to this device" on CABLE Output

## Configuration

Create a `config.toml` file in the project root:

```toml
sample_rate = 48000
buffer_size = 512

[effects]
pitch_shift = -5.0        # Shift pitch down 5 semitones
formant_shift = 3.0       # Shift formants up
reverb_enabled = true
robot_enabled = false
distortion = 0.3          # 30% distortion
```

### Configuration Options

- `sample_rate`: Audio sample rate (44100 or 48000 recommended)
- `buffer_size`: Buffer size in samples (smaller = lower latency, higher CPU)
- `effects.pitch_shift`: Pitch shift in semitones (-12 to +12)
- `effects.formant_shift`: Formant shift in semitones (-12 to +12)
- `effects.reverb_enabled`: Enable/disable reverb
- `effects.robot_enabled`: Enable/disable robot effect
- `effects.distortion`: Distortion amount (0.0 to 1.0)

## Usage

### Basic Usage

```bash
# Run with default configuration
cargo run --release

# Set log level for debugging
RUST_LOG=debug cargo run --release
```

### Runtime Controls

Currently, effects are configured via the `config.toml` file. Future versions will include:
- Real-time effect parameter control
- GUI interface
- Preset management
- Hotkey support

## Low Latency Optimization

VoiceBoard is optimized for minimal latency:

### Buffer Size vs Latency

| Buffer Size | Latency @ 48kHz | CPU Usage |
|-------------|-----------------|-----------|
| 256         | 5.33ms          | High      |
| 512         | 10.67ms         | Medium    |
| 1024        | 21.33ms         | Low       |

### Recommended Settings

For **best latency** (gaming, live streaming):
```toml
sample_rate = 48000
buffer_size = 256
```

For **balanced performance**:
```toml
sample_rate = 48000
buffer_size = 512
```

For **stability on slower systems**:
```toml
sample_rate = 44100
buffer_size = 1024
```

## Windows Audio APIs (WASAPI)

VoiceBoard uses Windows Audio Session API (WASAPI) for professional audio I/O:

### WASAPI Features Used

- **Shared Mode**: Allows multiple applications to use audio devices
- **Event-Driven Capture**: Efficient callback-based audio capture
- **Low-Latency Rendering**: Direct audio output with minimal buffering
- **32-bit Float PCM**: High-quality audio processing

### Audio Flow

1. **Capture**: WASAPI captures microphone input in 32-bit float format
2. **Processing**: DSP effects process audio in floating-point for quality
3. **Rendering**: WASAPI renders to output device with minimal latency

## Performance Considerations

- **CPU Usage**: Effects like pitch shifting use FFT and are CPU-intensive
- **Memory**: Each effect maintains internal buffers (minimal overhead)
- **Latency**: Total latency = buffer_size / sample_rate + processing time
- **Threading**: Currently single-threaded; future versions may use parallel processing

## Troubleshooting

### No Audio Output

1. Check Windows Sound Settings - ensure correct output device
2. Verify microphone permissions in Windows Privacy settings
3. Check that WASAPI devices are not in use by another application

### High CPU Usage

1. Increase buffer size in config
2. Disable unused effects
3. Lower sample rate to 44100 Hz

### Crackling/Distortion

1. Increase buffer size to reduce underruns
2. Disable other CPU-intensive applications
3. Check audio driver updates

### Virtual Cable Not Working

1. Reinstall virtual audio cable driver
2. Reboot computer
3. Check Windows Sound Settings for proper device selection

## Development

### Building for Development

```bash
# Development build (faster compilation)
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=trace cargo run
```

### Adding New Effects

1. Create new effect module in `src/dsp/`
2. Implement effect processing logic
3. Add to `EffectChain` in `src/dsp/mod.rs`
4. Update configuration in `src/config/mod.rs`

## Technical Details

### DSP Algorithms

**Pitch Shifting**: Phase vocoder using STFT
- FFT Size: 2048 samples
- Hop Size: 512 samples
- Window: Hann window
- Phase locking for coherent resynthesis

**Formant Shifting**: Spectral envelope warping
- Frequency axis warping to shift formants
- Preserves pitch while changing timbre

**Reverb**: Schroeder parallel comb filters + series allpass
- 4 parallel comb filters
- 2 series allpass filters
- Delay times based on Schroeder's recommendations

**Robot Effect**: Ring modulation
- Carrier frequency: 30 Hz
- Hard clipping for enhanced effect

**Distortion**: Soft clipping using tanh
- Configurable gain (1x to 10x)
- Dry/wet mix control

## Future Enhancements

- [ ] GUI for real-time control
- [ ] More voice effects (chorus, flanger, echo)
- [ ] Preset system
- [ ] ASIO support for even lower latency
- [ ] Multi-channel audio support
- [ ] VST plugin version
- [ ] Real-time parameter automation

## License

MIT License - See LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Credits

- DSP algorithms based on standard audio processing techniques
- WASAPI integration using the `windows` crate
- FFT processing using `realfft` and `rustfft` crates