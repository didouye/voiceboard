# Building and Testing Guide

## Platform Requirements

**IMPORTANT: VoiceBoard is Windows-only**

This project uses Windows Audio Session API (WASAPI) and must be built on Windows. Building on Linux or macOS will fail.

### Supported Platforms
- ✅ Windows 10 (64-bit)
- ✅ Windows 11 (64-bit)
- ❌ Linux
- ❌ macOS

## Build Requirements

### 1. Install Rust

Download and install Rust from: https://rustup.rs/

```powershell
# Verify installation
rustc --version
cargo --version
```

Expected output:
```
rustc 1.70.0 (or higher)
cargo 1.70.0 (or higher)
```

### 2. Install Visual Studio Build Tools

WASAPI requires the MSVC compiler:

1. Download from: https://visualstudio.microsoft.com/downloads/
2. Install "Desktop development with C++"
3. Reboot if prompted

### 3. Verify Setup

```powershell
# Check Rust toolchain
rustup show

# Should show stable-x86_64-pc-windows-msvc
```

## Building

### Development Build

For quick iteration during development:

```powershell
# Navigate to project directory
cd voiceboard

# Build in debug mode (faster compilation, slower runtime)
cargo build

# Time: ~2-5 minutes on first build
# Output: target/debug/voiceboard.exe
```

### Release Build

For actual use with optimizations:

```powershell
# Build in release mode (slower compilation, faster runtime)
cargo build --release

# Time: ~5-10 minutes on first build
# Output: target/release/voiceboard.exe
```

### Check Without Building

To verify code compiles without producing an executable:

```powershell
cargo check
```

This is faster than `cargo build` and useful for quick feedback.

## Testing

### Automated Tests

Currently, the project focuses on integration testing with real audio hardware. To add unit tests:

```powershell
# Run tests (when available)
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Manual Testing

Since this is real-time audio software, manual testing is essential:

#### Test 1: Verify Compilation

```powershell
cargo check
```

Expected: No errors

#### Test 2: Run with Default Config

```powershell
# Create minimal config
echo "[effects]" > config.toml

# Run with logging
$env:RUST_LOG="info"
cargo run --release
```

Expected output:
```
VoiceBoard - Real-time Voice Changer
====================================
Configuration loaded: buffer_size=512, sample_rate=48000
Effect chain initialized
Audio capture initialized
Audio renderer initialized
Starting real-time audio processing...
Press Ctrl+C to stop
```

#### Test 3: Test Microphone Input

1. Run VoiceBoard
2. Speak into microphone
3. Check Windows sound level meters
4. Should see input activity

#### Test 4: Test Audio Output

1. Enable "Listen to this device" on CABLE Output
2. Run VoiceBoard
3. Speak into microphone
4. Should hear processed audio in headphones

#### Test 5: Test Each Effect

Create test configs and verify each effect works:

**Pitch Test:**
```toml
[effects]
pitch_shift = -5.0
```

**Formant Test:**
```toml
[effects]
formant_shift = 3.0
```

**Reverb Test:**
```toml
[effects]
reverb_enabled = true
```

**Robot Test:**
```toml
[effects]
robot_enabled = true
```

**Distortion Test:**
```toml
[effects]
distortion = 0.5
```

### Performance Testing

#### CPU Usage Test

```powershell
# Run VoiceBoard
cargo run --release

# In another terminal, check CPU usage
# Task Manager -> Details -> voiceboard.exe
```

Expected CPU usage:
- Idle: <1%
- With all effects: 5-15%
- Single effect: 2-5%

#### Latency Test

```powershell
# Use smallest buffer for lowest latency
echo "buffer_size = 256" > config.toml
echo "[effects]" >> config.toml
echo "pitch_shift = -3.0" >> config.toml

cargo run --release
```

Measure latency by:
1. Tap microphone while monitoring
2. Listen for delay between tap and hearing it
3. Should be <30ms with buffer_size=256

#### Stability Test

Run for extended period:

```powershell
# Run for 1 hour
cargo run --release
# Let it run while doing other activities
# Monitor for crashes or audio dropouts
```

## Troubleshooting Build Issues

### Error: "MSVC not found"

**Solution:**
```powershell
# Install Visual Studio Build Tools
# Then verify:
rustup default stable-msvc
```

### Error: "windows-sys" compilation failed

**Solution:**
```powershell
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Error: Linking failed

**Solution:**
```powershell
# Ensure you're on Windows
# This project cannot be built on Linux/Mac

# Update dependencies
cargo update
cargo build --release
```

### Warning: Many warnings during compilation

This is normal. The warnings are from:
- Unused fields in structs (reserved for future use)
- Unused imports (for future features)

These don't affect functionality.

## Code Quality Checks

### Formatting

Check code formatting:

```powershell
# Install rustfmt if needed
rustup component add rustfmt

# Check formatting
cargo fmt --check

# Auto-format code
cargo fmt
```

### Linting

Run Clippy for additional checks:

```powershell
# Install clippy if needed
rustup component add clippy

# Run linting
cargo clippy

# Run with strict settings
cargo clippy -- -D warnings
```

### Documentation

Generate and view documentation:

```powershell
# Generate docs
cargo doc

# Open in browser
cargo doc --open
```

## Benchmarking

For performance testing:

```powershell
# Future: Benchmark tests
# cargo bench
```

## Continuous Integration

For automated testing in CI/CD:

```yaml
# Example GitHub Actions workflow
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
      - run: cargo test
```

## Release Process

### Creating a Release Build

```powershell
# 1. Clean previous builds
cargo clean

# 2. Update version in Cargo.toml
# version = "0.2.0"

# 3. Build release
cargo build --release

# 4. Test release binary
.\target\release\voiceboard.exe

# 5. Create distribution package
# Copy:
# - target/release/voiceboard.exe
# - config.toml
# - README.md
# - VIRTUAL_AUDIO_SETUP.md
```

### Binary Size Optimization

Already configured in `Cargo.toml`:

```toml
[profile.release]
opt-level = 3      # Maximum optimization
lto = true         # Link-Time Optimization
codegen-units = 1  # Better optimization
strip = true       # Remove debug symbols
```

Expected binary size: ~3-5 MB

## Common Build Errors and Solutions

| Error | Cause | Solution |
|-------|-------|----------|
| "linker not found" | MSVC not installed | Install Visual Studio Build Tools |
| "cannot find -lwindows" | Wrong platform | Must build on Windows |
| "undefined symbol" | Missing Windows feature | Update Cargo.toml features |
| "out of memory" | Large compile | Increase system RAM or reduce parallel builds |

## Development Workflow

Recommended workflow for contributors:

```powershell
# 1. Make code changes
# edit src/...

# 2. Quick check
cargo check

# 3. Format code
cargo fmt

# 4. Run linter
cargo clippy

# 5. Build
cargo build

# 6. Test
cargo run -- # your test args

# 7. Release build
cargo build --release

# 8. Test release
.\target\release\voiceboard.exe
```

## System Requirements for Testing

### Minimum Requirements
- Windows 10 64-bit
- Dual-core CPU (2 GHz+)
- 4 GB RAM
- Microphone
- Speakers or headphones
- Virtual audio cable software

### Recommended Requirements
- Windows 11 64-bit
- Quad-core CPU (3 GHz+)
- 8 GB RAM
- USB microphone
- Studio headphones
- VoiceMeeter

## Known Build Limitations

1. **Windows Only**: Cannot cross-compile from Linux/Mac
2. **MSVC Required**: MinGW not supported
3. **64-bit Only**: 32-bit builds not tested
4. **No ASIO**: Currently only WASAPI (ASIO support planned)

## Getting Help

If you encounter build issues:

1. Check this guide's troubleshooting section
2. Verify you're on Windows with MSVC
3. Try `cargo clean` and rebuild
4. Update Rust: `rustup update`
5. Check GitHub Issues
6. Open new issue with:
   - OS version
   - Rust version (`rustc --version`)
   - Full error output
   - Steps to reproduce

## Next Steps

After successful build:
1. Read [VIRTUAL_AUDIO_SETUP.md](VIRTUAL_AUDIO_SETUP.md)
2. Read [EXAMPLES.md](EXAMPLES.md)
3. Run with default config
4. Experiment with effects
5. Share your configurations!
