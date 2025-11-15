# Audio Pipeline Implementation Guide

This document provides detailed implementation guidance for the audio processing pipeline in Voiceboard.

## Overview

The audio pipeline consists of four main threads:

1. **Capture Thread**: Reads audio from the microphone
2. **Playback Thread(s)**: Decode and play sound files
3. **Mixer Thread**: Combines mic + sounds
4. **Output Thread**: Sends to virtual microphone

## Architecture Diagram

```
┌─────────────────┐
│  Microphone     │
│  (WASAPI Input) │
└────────┬────────┘
         │
         ▼
┌─────────────────────────┐
│  Capture Thread         │
│  - CPAL stream          │
│  - Ring buffer write    │
└────────┬────────────────┘
         │ PCM samples
         ▼
┌─────────────────────────┐      ┌──────────────────┐
│  Mixer Thread           │◄─────│ Playback Thread  │
│  - Read from capture    │      │ - Decode sound   │
│  - Mix with sounds      │      │ - Ring buffer    │
│  - Apply volumes        │      └──────────────────┘
│  - Write to output      │
└────────┬────────────────┘
         │ Mixed PCM
         ▼
┌─────────────────────────┐
│  Output Thread          │
│  - CPAL output stream   │
│  - Virtual device       │
└────────┬────────────────┘
         │
         ▼
┌─────────────────────────┐
│  Virtual Microphone     │
│  (WASAPI Output)        │
└─────────────────────────┘
```

## Detailed Implementation

### 1. Capture Thread

**File**: `src-tauri/src/audio/capture.rs`

**Purpose**: Capture audio from the real microphone

**Key Components**:

```rust
pub struct MicrophoneCapture {
    device: Device,
    stream: Option<Stream>,
    format: AudioFormat,
    ring_buffer: Arc<HeapRb<Sample>>,
}
```

**Implementation Steps**:

1. **Initialize CPAL device**:
```rust
let host = cpal::default_host();
let device = host.default_input_device()
    .ok_or("No input device")?;
```

2. **Configure stream**:
```rust
let stream_config = StreamConfig {
    channels: 2,
    sample_rate: SampleRate(48000),
    buffer_size: BufferSize::Default,
};
```

3. **Create ring buffer**:
```rust
// Size = 4x the expected buffer size for safety
let buffer_size = 4800 * 4; // 400ms at 48kHz stereo
let ring_buffer = HeapRb::<f32>::new(buffer_size);
```

4. **Build input stream**:
```rust
let stream = device.build_input_stream(
    &stream_config,
    move |data: &[f32], _info| {
        // Write to ring buffer
        ring_buffer.producer().push_slice(data);
    },
    move |err| {
        error!("Stream error: {}", err);
    },
    None,
)?;
```

5. **Start stream**:
```rust
stream.play()?;
```

**Performance Considerations**:

- Ring buffer size: 4x expected buffer size (prevents overflow)
- Use lock-free ring buffer (ringbuf crate)
- No allocations in audio callback
- No blocking operations in audio callback

### 2. Playback Thread

**File**: `src-tauri/src/audio/playback.rs`

**Purpose**: Decode and play sound files

**Key Components**:

```rust
pub struct SoundPlayer {
    id: String,
    buffer: Arc<AudioBuffer>,
    position: usize,
    volume: Volume,
}
```

**Implementation Steps**:

1. **Decode audio file** (on first play):
```rust
let buffer = AudioDecoder::decode_file(path)?;
```

2. **Create player**:
```rust
let player = SoundPlayer::from_buffer(id, buffer, volume);
```

3. **Read samples**:
```rust
pub fn read(&mut self, output: &mut [f32]) -> usize {
    let available = self.buffer.data.len() - self.position;
    let to_read = available.min(output.len());

    for i in 0..to_read {
        output[i] = self.buffer.data[self.position + i] * self.volume.linear();
    }

    self.position += to_read;
    to_read
}
```

**Optimization**:

- Decode files ahead of time (cache decoded buffers)
- Use WAV files for instant playback (no decoding)
- Limit number of concurrent sounds (e.g., 8 max)

### 3. Mixer Thread

**File**: `src-tauri/src/audio/mixer.rs`

**Purpose**: Combine microphone input with sound effects

**Key Components**:

```rust
pub struct AudioMixer {
    format: AudioFormat,
    sources: HashMap<String, MixerSource>,
    mic_buffer: Vec<Sample>,
    master_volume: Volume,
}
```

**Implementation Steps**:

1. **Read microphone samples**:
```rust
let mut mic_samples = vec![0.0f32; buffer_size];
let read = capture.read(&mut mic_samples);
```

2. **Clear output buffer**:
```rust
output.fill(0.0);
```

3. **Add microphone**:
```rust
for i in 0..mic_samples.len() {
    output[i] += mic_samples[i] * mic_volume.linear();
}
```

4. **Mix in sound sources**:
```rust
for (id, source) in &mut self.sources {
    let mut temp = vec![0.0f32; buffer_size];
    let read = source.player.read(&mut temp);

    for i in 0..read {
        output[i] += temp[i] * effects_volume.linear();
    }
}
```

5. **Apply master volume and clipping**:
```rust
for sample in output.iter_mut() {
    *sample *= master_volume.linear();
    *sample = sample.clamp(-1.0, 1.0); // Prevent clipping
}
```

**Performance Considerations**:

- Reuse buffers (don't allocate in loop)
- Remove finished sources immediately
- Use SIMD for mixing (future enhancement)
- Profile mixing loop for bottlenecks

### 4. Output Thread

**File**: `src-tauri/src/audio/virtual_device.rs`

**Purpose**: Send mixed audio to virtual microphone

**Key Components**:

```rust
pub struct VirtualMicrophone {
    device: Device,
    stream: Option<Stream>,
    format: AudioFormat,
    ring_buffer: Arc<HeapRb<Sample>>,
}
```

**Implementation Steps**:

1. **Find virtual audio cable**:
```rust
fn find_virtual_cable(host: &Host) -> Option<Device> {
    let devices = host.output_devices().ok()?;
    for device in devices {
        let name = device.name().ok()?;
        if name.to_lowercase().contains("cable") {
            return Some(device);
        }
    }
    None
}
```

2. **Create ring buffer**:
```rust
let ring_buffer = HeapRb::<f32>::new(buffer_size * 4);
```

3. **Build output stream**:
```rust
let stream = device.build_output_stream(
    &stream_config,
    move |data: &mut [f32], _info| {
        // Read from ring buffer
        let read = ring_buffer.consumer().pop_slice(data);

        // Fill remaining with silence
        for i in read..data.len() {
            data[i] = 0.0;
        }
    },
    move |err| {
        error!("Output stream error: {}", err);
    },
    None,
)?;
```

4. **Start stream**:
```rust
stream.play()?;
```

5. **Write mixed audio**:
```rust
pub fn write(&self, samples: &[Sample]) -> usize {
    self.ring_buffer.producer().push_slice(samples).len()
}
```

## Thread Coordination

### Main Audio Engine Loop

**File**: `src-tauri/src/audio/engine.rs`

```rust
pub async fn run_audio_loop(engine: Arc<AudioEngine>) {
    let mut capture = MicrophoneCapture::new(config)?;
    let mut mixer = AudioMixer::new(format);
    let mut virtual_mic = VirtualMicrophone::new(config)?;

    capture.start()?;
    virtual_mic.start()?;

    // Audio processing loop
    loop {
        // 1. Read from microphone
        let mut mic_buffer = vec![0.0f32; buffer_size];
        let mic_read = capture.read(&mut mic_buffer);

        // 2. Mix with sounds
        let mut output_buffer = vec![0.0f32; buffer_size];
        mixer.set_mic_input(&mic_buffer[..mic_read]);
        mixer.mix(
            &mut output_buffer,
            engine.config().mic_volume,
            engine.config().effects_volume,
        );

        // 3. Write to virtual microphone
        virtual_mic.write(&output_buffer);

        // 4. Calculate audio levels for visualization
        let mic_level = AudioLevel::from_buffer(&mic_buffer);
        let output_level = AudioLevel::from_buffer(&output_buffer);

        // 5. Emit event to frontend
        emit_audio_level_event(mic_level, output_level);

        // Small sleep to prevent busy loop (adjust based on buffer size)
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
```

## Resampling

**File**: `src-tauri/src/audio/resampler.rs`

**When needed**:
- Sound file sample rate ≠ 48kHz
- Microphone sample rate ≠ 48kHz

**Implementation**:

```rust
use rubato::{Resampler, SincFixedIn};

pub fn resample_buffer(
    input: &AudioBuffer,
    target_rate: u32,
) -> Result<AudioBuffer> {
    // De-interleave samples
    let mut channels = separate_channels(input);

    // Create resampler
    let ratio = target_rate as f64 / input.format.sample_rate as f64;
    let mut resampler = SincFixedIn::<f32>::new(
        ratio,
        2.0,
        params,
        input.frames(),
        2,
    )?;

    // Resample
    let resampled = resampler.process(&channels, None)?;

    // Re-interleave
    let output = interleave_channels(&resampled, target_rate);

    Ok(output)
}
```

## Latency Optimization

**Target Latency**: < 20ms total

**Latency Sources**:

1. **Input buffer**: 10ms (480 samples @ 48kHz)
2. **Processing**: ~1ms
3. **Output buffer**: 10ms
4. **Total**: ~21ms

**Optimization Strategies**:

1. **Reduce buffer size**:
```rust
let config = AudioConfig {
    buffer_size: 240, // 5ms @ 48kHz
    // ...
};
```

Note: Smaller buffers = more CPU usage, potential glitches

2. **Use faster resampling**:
```rust
// Linear interpolation instead of sinc
let params = SincInterpolationParameters {
    interpolation: SincInterpolationType::Linear,
    // ...
};
```

3. **Optimize mixer**:
```rust
// Limit concurrent sounds
const MAX_CONCURRENT_SOUNDS: usize = 8;

if mixer.sources.len() >= MAX_CONCURRENT_SOUNDS {
    return Err("Too many concurrent sounds");
}
```

## Error Handling

### Stream Errors

```rust
let stream = device.build_input_stream(
    &config,
    move |data, _| { /* ... */ },
    move |err| {
        match err {
            cpal::StreamError::DeviceNotAvailable => {
                // Device disconnected, try to reconnect
                attempt_reconnect();
            }
            _ => {
                error!("Stream error: {}", err);
            }
        }
    },
    None,
)?;
```

### Buffer Overflow/Underflow

```rust
// On write
match ring_buffer.producer().push_slice(data) {
    Ok(_) => {}
    Err(remaining) => {
        // Buffer full, drop samples
        warn!("Dropped {} samples", remaining.len());
    }
}

// On read
let read = ring_buffer.consumer().pop_slice(output);
if read < output.len() {
    // Underflow, fill with silence
    output[read..].fill(0.0);
}
```

## Testing

### Unit Tests

```rust
#[test]
fn test_mixer_basic() {
    let format = AudioFormat::standard();
    let mut mixer = AudioMixer::new(format);

    // Add test sound
    let buffer = create_test_buffer(format, 480);
    mixer.add_source("test", buffer, Volume::max()).unwrap();

    // Mix
    let mut output = vec![0.0f32; 960];
    mixer.mix(&mut output, Volume::max(), Volume::max());

    // Verify output is not silent
    assert!(output.iter().any(|&s| s != 0.0));
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_pipeline() {
    // Create components
    let capture = MicrophoneCapture::new(config).unwrap();
    let mixer = AudioMixer::new(format);
    let virtual_mic = VirtualMicrophone::new(config).unwrap();

    // Start pipeline
    capture.start().unwrap();
    virtual_mic.start().unwrap();

    // Run for 1 second
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Verify audio flowed through
    assert!(virtual_mic.written_samples() > 0);
}
```

## Performance Profiling

### CPU Usage

```rust
use std::time::Instant;

let start = Instant::now();
mixer.mix(&mut output, mic_vol, fx_vol);
let duration = start.elapsed();

if duration > Duration::from_millis(5) {
    warn!("Mixing took {}ms (slow!)", duration.as_millis());
}
```

### Memory Usage

```bash
# On Windows
cargo run --release
# Monitor in Task Manager

# On Linux
cargo flamegraph --bin voiceboard
```

## Common Issues

### Issue: Crackling/Popping Audio

**Causes**:
- Buffer too small
- CPU overload
- Disk I/O in audio thread

**Solutions**:
- Increase buffer size
- Decode sounds ahead of time
- Use release build for testing

### Issue: High Latency

**Causes**:
- Large buffers
- Slow resampling
- Too many concurrent sounds

**Solutions**:
- Reduce buffer size
- Use faster resampling
- Limit concurrent sounds

### Issue: Audio Dropouts

**Causes**:
- Buffer underflow
- Thread starvation
- Device errors

**Solutions**:
- Increase ring buffer size
- Increase thread priority (if possible)
- Better error handling

## Future Enhancements

1. **SIMD Optimization**:
   ```rust
   #[cfg(target_arch = "x86_64")]
   use std::arch::x86_64::*;

   unsafe fn mix_simd(a: &[f32], b: &[f32], out: &mut [f32]) {
       // AVX2 SIMD mixing
   }
   ```

2. **Custom Resampler**:
   - Faster than rubato for common cases
   - Optimized for 44.1kHz → 48kHz

3. **Audio Effects**:
   - Reverb, echo, pitch shift
   - VST plugin support

4. **Advanced Mixing**:
   - Ducking (lower music when speaking)
   - Compression, limiting
   - EQ per sound

---

**Next Steps**: See [VIRTUAL_DEVICE.md](./VIRTUAL_DEVICE.md) for details on virtual microphone setup.
