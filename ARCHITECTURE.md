# Architecture Documentation

## System Architecture

VoiceBoard implements a real-time audio processing pipeline optimized for low latency voice transformation on Windows.

## Components

### 1. Audio Capture (audio/capture.rs)

**Technology**: WASAPI (Windows Audio Session API)

**Responsibilities**:
- Initialize audio capture device
- Set up event-driven callbacks
- Capture microphone input in real-time
- Convert audio to 32-bit float PCM

**Key Features**:
- Event-driven architecture for minimal latency
- Shared mode for compatibility
- Automatic buffer management

**WASAPI Flow**:
```
1. Create IMMDeviceEnumerator
2. Get default capture device (eCapture, eConsole)
3. Activate IAudioClient
4. Initialize with AUDCLNT_SHAREMODE_SHARED
5. Set event callback (AUDCLNT_STREAMFLAGS_EVENTCALLBACK)
6. Get IAudioCaptureClient
7. Start capture
8. Wait for event → Get buffer → Process → Release buffer
```

### 2. DSP Effect Chain (dsp/mod.rs)

**Architecture**: Sequential processing chain

**Processing Order**:
1. Pitch Shifter
2. Formant Shifter
3. Robot Effect
4. Distortion
5. Reverb (last for natural sound)

**Design Decisions**:
- Sequential processing for predictable results
- Optional effects (only instantiated when enabled)
- Stateful effects maintain internal buffers
- 32-bit float processing throughout

### 3. Audio Renderer (audio/renderer.rs)

**Technology**: WASAPI

**Responsibilities**:
- Initialize audio render device
- Output processed audio
- Handle buffer management
- Minimize output latency

**WASAPI Flow**:
```
1. Create IMMDeviceEnumerator
2. Get default render device (eRender, eConsole)
3. Activate IAudioClient
4. Initialize with AUDCLNT_SHAREMODE_SHARED
5. Get IAudioRenderClient
6. Start rendering
7. Get buffer size → Check padding → Write samples → Release buffer
```

## DSP Algorithms

### Pitch Shifting (Phase Vocoder)

**Algorithm**: Short-Time Fourier Transform (STFT) based phase vocoder

**Parameters**:
- FFT Size: 2048 samples
- Hop Size: 512 samples (75% overlap)
- Window: Hann window

**Process**:
1. Apply Hann window to input frame
2. Forward FFT → frequency domain
3. Extract magnitude and phase
4. Calculate phase differences
5. Compute true frequency (unwrap phase)
6. Scale frequency by pitch ratio
7. Accumulate new phase
8. Reconstruct complex spectrum
9. Inverse FFT → time domain
10. Overlap-add with previous frames

**Latency**: ~43ms (2048 samples @ 48kHz)

### Formant Shifting (Spectral Warping)

**Algorithm**: Frequency axis warping in spectral domain

**Process**:
1. Apply window to input
2. Forward FFT
3. Warp frequency bins (resample spectrum)
4. Inverse FFT
5. Overlap-add

**Effect**: Changes vocal characteristics without changing pitch

### Reverb (Schroeder Reverberator)

**Algorithm**: Parallel comb filters + series allpass filters

**Structure**:
```
Input → [Comb1]
     → [Comb2] → Sum → [Allpass1] → [Allpass2] → Output
     → [Comb3]    ↓
     → [Comb4]
```

**Parameters**:
- Comb delay times: 29.7ms, 37.1ms, 41.1ms, 43.7ms
- Comb gains: 0.742
- Allpass delay times: 5.0ms, 1.7ms
- Allpass gains: 0.7
- Wet/Dry mix: 30% wet, 70% dry

### Robot Effect (Ring Modulation)

**Algorithm**: Amplitude modulation with low-frequency carrier

**Parameters**:
- Carrier frequency: 30 Hz
- Clipping threshold: ±0.8

**Process**:
1. Generate sine wave carrier
2. Multiply input signal by carrier
3. Apply hard clipping
4. Output modulated signal

### Distortion (Wave Shaping)

**Algorithm**: Soft clipping using hyperbolic tangent

**Parameters**:
- Gain: 1x to 10x (based on amount)
- Mix: 50% dry/wet

**Process**:
1. Apply gain to input
2. Pass through tanh() function
3. Mix with dry signal
4. Output

## Latency Analysis

### Total System Latency

```
Total Latency = Input Buffer + Processing + Output Buffer + System

Input Buffer:  buffer_size / sample_rate
Processing:    ~1-5ms (depends on effects)
Output Buffer: buffer_size / sample_rate
System:        ~5-10ms (Windows audio stack)
```

### Example Calculations

**Configuration 1: Ultra Low Latency**
```
Sample Rate: 48000 Hz
Buffer Size: 256 samples

Input:     256 / 48000 = 5.33ms
Processing: ~3ms
Output:    256 / 48000 = 5.33ms
System:    ~8ms
Total:     ~21.66ms
```

**Configuration 2: Balanced**
```
Sample Rate: 48000 Hz
Buffer Size: 512 samples

Input:     512 / 48000 = 10.67ms
Processing: ~3ms
Output:    512 / 48000 = 10.67ms
System:    ~8ms
Total:     ~32.34ms
```

**Configuration 3: Stable**
```
Sample Rate: 44100 Hz
Buffer Size: 1024 samples

Input:     1024 / 44100 = 23.22ms
Processing: ~3ms
Output:    1024 / 44100 = 23.22ms
System:    ~8ms
Total:     ~57.44ms
```

## Threading Model

**Current**: Single-threaded event-driven architecture

**Audio Thread**:
- Runs at high priority (WASAPI manages this)
- Event-driven callbacks from WASAPI
- Processes audio in real-time
- Must complete processing within buffer duration

**Future Improvements**:
- Parallel effect processing using Rayon
- Separate control thread for GUI
- Lock-free ring buffers for thread communication

## Memory Management

### Buffer Allocation

**Static Buffers** (allocated once):
- FFT buffers (2048 samples each)
- Window functions
- Delay line buffers (reverb)
- Overlap buffers

**Dynamic Buffers** (per callback):
- Input buffer (from WASAPI)
- Output buffer (to WASAPI)
- Temporary processing buffers

### Memory Usage Estimate

Per effect instance:
- Pitch Shifter: ~50KB
- Formant Shifter: ~50KB
- Reverb: ~20KB
- Robot Effect: <1KB
- Distortion: <1KB

Total: ~150KB for all effects

## Error Handling

**Strategy**: Graceful degradation

1. **Initialization Errors**: Fail fast with descriptive messages
2. **Runtime Errors**: Log and continue (don't crash)
3. **Buffer Underruns**: Zero-pad output
4. **Invalid Parameters**: Clamp to valid ranges

## Performance Optimization

### Compilation Flags

```toml
[profile.release]
opt-level = 3           # Maximum optimization
lto = true             # Link-Time Optimization
codegen-units = 1      # Better optimization
strip = true           # Remove debug symbols
```

### Runtime Optimizations

1. **Minimize Allocations**: Reuse buffers
2. **SIMD**: FFT libraries use SIMD when available
3. **Cache Locality**: Sequential memory access
4. **Avoid Locks**: Lock-free where possible

### CPU Usage Targets

- Idle: <1% CPU
- Active with all effects: 5-15% CPU (on modern CPUs)
- Single effect: 2-5% CPU

## Virtual Audio Device Integration

### Routing Options

**Option 1: Direct to Virtual Cable**
```
VoiceBoard → CABLE Input → [Applications read from CABLE Output]
```

**Option 2: Through Mixer**
```
VoiceBoard → VoiceMeeter → Virtual Outputs → Applications
```

### Monitoring Setup

```
Physical Mic → VoiceBoard → Virtual Cable
                              ↓
                         Applications
                              ↓
                      Physical Speakers
```

## Future Architecture Improvements

1. **Plugin System**: Load effects dynamically
2. **GPU Acceleration**: Use compute shaders for FFT
3. **Multi-Channel**: Support stereo and surround
4. **ASIO Support**: Even lower latency
5. **Network Streaming**: Remote audio processing
6. **Preset System**: Save/load effect configurations
