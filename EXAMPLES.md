# Usage Examples

This document provides practical examples of using VoiceBoard for various scenarios.

## Basic Usage

### Running with Default Settings

The simplest way to start VoiceBoard:

```bash
cargo run --release
```

This will:
- Use sample rate of 48000 Hz
- Use buffer size of 512 samples
- Apply no effects (pass-through mode)
- Capture from default microphone
- Output to default playback device

### Running with Logging

To see what's happening under the hood:

```bash
# Info level logging
RUST_LOG=info cargo run --release

# Debug level logging
RUST_LOG=debug cargo run --release

# Trace level logging (very verbose)
RUST_LOG=trace cargo run --release
```

## Configuration Examples

### Example 1: Deep Voice Effect

Create a `config.toml` file:

```toml
sample_rate = 48000
buffer_size = 512

[effects]
pitch_shift = -7.0      # Down 7 semitones (about a musical fifth)
formant_shift = -4.0    # Make it sound deeper/older
reverb_enabled = false
robot_enabled = false
distortion = null
```

### Example 2: High-Pitched Voice (Chipmunk)

```toml
sample_rate = 48000
buffer_size = 512

[effects]
pitch_shift = 8.0       # Up 8 semitones
formant_shift = 6.0     # Younger/higher
reverb_enabled = false
robot_enabled = false
distortion = null
```

### Example 3: Robot Voice

```toml
sample_rate = 48000
buffer_size = 512

[effects]
pitch_shift = 0.0       # No pitch change
formant_shift = null    # No formant shift
reverb_enabled = true   # Add some reverb for effect
robot_enabled = true    # Enable robot effect
distortion = 0.4        # Add some distortion
```

### Example 4: Monster Voice

```toml
sample_rate = 48000
buffer_size = 512

[effects]
pitch_shift = -12.0     # Down one octave
formant_shift = -8.0    # Very deep
reverb_enabled = true
robot_enabled = false
distortion = 0.6        # Heavy distortion
```

### Example 5: Radio/Telephone Effect

```toml
sample_rate = 48000
buffer_size = 512

[effects]
pitch_shift = 0.0
formant_shift = null
reverb_enabled = false
robot_enabled = true    # Ring modulation
distortion = 0.3        # Light distortion
```

### Example 6: Natural Enhancement

```toml
sample_rate = 48000
buffer_size = 512

[effects]
pitch_shift = -1.0      # Slightly lower
formant_shift = null    # Keep natural
reverb_enabled = true   # Add space
robot_enabled = false
distortion = 0.1        # Very subtle warmth
```

## Performance Optimization Examples

### Low Latency Gaming Setup

For gaming and live streaming where latency matters most:

```toml
sample_rate = 48000
buffer_size = 256       # Lower latency

[effects]
# Use fewer effects for lower CPU usage
pitch_shift = -3.0
formant_shift = null
reverb_enabled = false  # Reverb is CPU intensive
robot_enabled = false
distortion = null
```

### High Quality Streaming Setup

For pre-recorded content or when CPU usage isn't critical:

```toml
sample_rate = 48000
buffer_size = 1024      # More stable

[effects]
# All effects enabled
pitch_shift = -4.0
formant_shift = -2.0
reverb_enabled = true
robot_enabled = false
distortion = 0.2
```

### Minimal CPU Usage

For older/slower computers:

```toml
sample_rate = 44100     # Lower sample rate
buffer_size = 2048      # Larger buffer

[effects]
# Only simple effects
pitch_shift = -3.0
formant_shift = null
reverb_enabled = false
robot_enabled = false
distortion = null
```

## Application-Specific Examples

### Discord Setup

1. Create `config.toml`:
```toml
sample_rate = 48000
buffer_size = 512

[effects]
pitch_shift = -5.0
formant_shift = null
reverb_enabled = false
robot_enabled = false
distortion = null
```

2. Start VoiceBoard:
```bash
cargo run --release
```

3. In Discord:
   - Voice & Video settings
   - Input Device: CABLE Output
   - Output Device: Your headphones

### OBS Streaming Setup

1. Create `config.toml` for streaming:
```toml
sample_rate = 48000
buffer_size = 1024      # More stable for streaming

[effects]
pitch_shift = -3.0
formant_shift = -1.0
reverb_enabled = true
robot_enabled = false
distortion = 0.15
```

2. Start VoiceBoard

3. In OBS:
   - Add "Audio Input Capture" source
   - Device: CABLE Output
   - Done!

### Zoom/Teams Meeting Setup

1. Create professional-sounding config:
```toml
sample_rate = 48000
buffer_size = 512

[effects]
pitch_shift = -2.0      # Slightly deeper
formant_shift = null    # Natural
reverb_enabled = false  # Clear audio
robot_enabled = false
distortion = null       # Clean
```

2. In Zoom/Teams:
   - Audio settings
   - Microphone: CABLE Output
   - Disable noise suppression (let VoiceBoard handle it)

## Command-Line Usage

### Development Mode

Quick iteration during development:

```bash
# Build and run in debug mode (faster compilation)
cargo run

# With logging
RUST_LOG=debug cargo run
```

### Release Mode

For actual use:

```bash
# Build optimized version
cargo build --release

# Run
./target/release/voiceboard

# Or build and run in one command
cargo run --release
```

### Custom Configuration File

If you want to use a different config file:

```bash
# Future feature: specify config file
# cargo run --release -- --config my_config.toml
```

## Testing Audio Pipeline

### Step 1: Test Microphone Input

First, verify your microphone works without VoiceBoard:
1. Open Windows Sound Settings
2. Recording tab
3. Speak into mic and watch the level meter

### Step 2: Test Pass-Through

Run VoiceBoard with no effects to test the pipeline:

```toml
sample_rate = 48000
buffer_size = 512

[effects]
pitch_shift = null
formant_shift = null
reverb_enabled = false
robot_enabled = false
distortion = null
```

You should hear your voice unchanged (with some latency).

### Step 3: Test Individual Effects

Test each effect one at a time:

**Pitch only:**
```toml
[effects]
pitch_shift = -5.0
formant_shift = null
reverb_enabled = false
robot_enabled = false
distortion = null
```

**Formant only:**
```toml
[effects]
pitch_shift = null
formant_shift = -3.0
reverb_enabled = false
robot_enabled = false
distortion = null
```

And so on...

## Troubleshooting Examples

### High CPU Usage

If you're experiencing high CPU usage:

1. **Increase buffer size:**
```toml
buffer_size = 2048  # Was 512
```

2. **Disable heavy effects:**
```toml
[effects]
pitch_shift = -3.0
formant_shift = null
reverb_enabled = false  # <-- Disable reverb
robot_enabled = false
distortion = null
```

3. **Lower sample rate:**
```toml
sample_rate = 44100  # Was 48000
```

### Audio Crackling

If you hear crackling or dropouts:

1. **Increase buffer size:**
```toml
buffer_size = 1024  # Larger buffer = more stable
```

2. **Check CPU usage:**
```bash
# Windows Task Manager
# Look for voiceboard.exe
# Should be < 20% CPU
```

3. **Close other apps using audio**

### No Audio Output

If you can't hear anything:

1. Check Windows default playback device
2. Check virtual cable is installed
3. Enable "Listen to this device" for monitoring
4. Look at logs:
```bash
RUST_LOG=debug cargo run --release
```

## Advanced Usage Patterns

### Multiple Configurations

Create different config files for different scenarios:

```bash
# configs/discord.toml
# configs/streaming.toml
# configs/gaming.toml
# configs/professional.toml
```

Then copy the one you need:
```bash
cp configs/discord.toml config.toml
cargo run --release
```

### Automation

Create a batch file for quick launching:

```batch
@echo off
cd C:\path\to\voiceboard
set RUST_LOG=info
cargo run --release
```

Save as `start_voiceboard.bat`

### Background Running

To run VoiceBoard in the background:

```bash
# Build first
cargo build --release

# Then run in background (Windows)
start /B target\release\voiceboard.exe
```

## Effect Parameter Guide

### Pitch Shift Values

- `-12` to `-24`: Very deep, monster-like
- `-7` to `-12`: Deep, masculine
- `-3` to `-7`: Slightly deeper, natural
- `-1` to `-3`: Subtle enhancement
- `0`: No change
- `+1` to `+3`: Slightly higher
- `+3` to `+7`: Higher, feminine
- `+7` to `+12`: Very high, cartoon-like
- `+12` to `+24`: Extreme, chipmunk

### Formant Shift Values

- Negative values: Older, deeper, more masculine
- `0` or `null`: No change
- Positive values: Younger, higher, more feminine

### Distortion Values

- `0.0` to `0.1`: Subtle warmth
- `0.1` to `0.3`: Noticeable grit
- `0.3` to `0.5`: Heavy distortion
- `0.5` to `1.0`: Extreme distortion

## Real-World Scenarios

### Scenario: Anonymous Streaming

```toml
sample_rate = 48000
buffer_size = 512

[effects]
pitch_shift = -8.0
formant_shift = -5.0
reverb_enabled = true
robot_enabled = false
distortion = 0.2
```

### Scenario: Voice Acting Character

```toml
sample_rate = 48000
buffer_size = 512

[effects]
pitch_shift = -10.0
formant_shift = -6.0
reverb_enabled = false
robot_enabled = false
distortion = 0.5
```

### Scenario: Professional Podcast

```toml
sample_rate = 48000
buffer_size = 1024

[effects]
pitch_shift = -1.0      # Slight enhancement
formant_shift = null
reverb_enabled = false  # Clean, professional
robot_enabled = false
distortion = 0.05       # Warmth only
```

## Tips and Best Practices

1. **Start Simple**: Begin with just pitch shift, then add other effects
2. **Test Alone**: Always test your setup before going live
3. **Monitor Yourself**: Use "Listen to this device" to hear what others hear
4. **Document Settings**: Save configurations that work well
5. **CPU Headroom**: Leave some CPU for other applications
6. **Backup Config**: Keep a copy of working configurations
7. **Gradual Changes**: Small parameter changes often sound more natural

## Next Steps

- Experiment with different combinations
- Create presets for different scenarios
- Share configurations with friends
- Report issues or suggest features on GitHub
