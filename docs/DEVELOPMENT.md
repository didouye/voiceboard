# Voiceboard Development Guide

This guide covers everything you need to know to develop and build the Voiceboard application.

## Prerequisites

### Required Software

1. **Rust** (1.75+)
   ```powershell
   # Install via rustup
   winget install Rustlang.Rustup
   # Or download from: https://rustup.rs/
   ```

2. **Node.js** (18+) and npm
   ```powershell
   winget install OpenJS.NodeJS
   # Or download from: https://nodejs.org/
   ```

3. **Visual Studio Build Tools**
   - Download from: https://visualstudio.microsoft.com/downloads/
   - Install "Desktop development with C++" workload
   - Required for compiling native Windows modules

4. **Git**
   ```powershell
   winget install Git.Git
   ```

### Recommended Software

- **VS Code** with extensions:
  - rust-analyzer
  - Angular Language Service
  - Tauri
  - Error Lens
  - Better TOML

- **Virtual Audio Cable Software** (for testing):
  - VB-Audio Virtual Cable: https://vb-audio.com/Cable/
  - VoiceMeeter: https://vb-audio.com/Voicemeeter/

## Initial Setup

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/voiceboard.git
cd voiceboard
```

### 2. Install Rust Dependencies

```bash
cd src-tauri
cargo build
```

This will download and compile all Rust dependencies. First build may take 5-10 minutes.

### 3. Install Node Dependencies

```bash
cd ../src-ui
npm install
```

### 4. Verify Installation

```bash
# Check Rust
rustc --version
cargo --version

# Check Node
node --version
npm --version

# Check Tauri CLI
npm run tauri --version
```

## Development Workflow

### Running the Development Build

The easiest way to run the app in development mode:

```bash
cd src-ui
npm run tauri:dev
```

This command will:
1. Start the Angular dev server (http://localhost:4200)
2. Build the Rust backend
3. Launch the Tauri window
4. Enable hot-reload for Angular changes
5. Rebuild Rust on file changes

### Running Angular Only (for UI development)

```bash
cd src-ui
npm run dev
```

This starts just the Angular dev server. Useful for UI-only development, but Tauri commands won't work.

### Running Rust Tests

```bash
cd src-tauri
cargo test
```

### Running Angular Tests

```bash
cd src-ui
npm run test
```

## Project Structure

```
voiceboard/
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── audio/          # Audio processing
│   │   ├── soundboard/     # Sound management
│   │   ├── devices/        # Device management
│   │   ├── config/         # Configuration
│   │   ├── commands/       # Tauri commands
│   │   ├── main.rs         # Entry point
│   │   └── lib.rs          # Library exports
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri config
│
└── src-ui/                 # Angular frontend
    ├── src/app/
    │   ├── components/     # UI components
    │   ├── services/       # Business logic
    │   └── models/         # TypeScript types
    ├── package.json        # Node dependencies
    └── angular.json        # Angular config
```

## Development Guidelines

### Rust Development

#### Code Style

```bash
# Format code
cargo fmt

# Run clippy (linter)
cargo clippy
```

#### Adding Dependencies

Edit `src-tauri/Cargo.toml`:

```toml
[dependencies]
new-crate = "1.0"
```

Then run:
```bash
cargo build
```

#### Logging

Use the `tracing` crate for logging:

```rust
use tracing::{debug, info, warn, error};

info!("Starting audio engine");
debug!("Buffer size: {}", size);
error!("Failed to open device: {}", err);
```

Set log level via environment variable:
```powershell
$env:RUST_LOG="voiceboard=debug,tauri=info"
npm run tauri:dev
```

#### Error Handling

Always use the `Result` type:

```rust
use crate::error::Result;

pub fn my_function() -> Result<()> {
    // ... code
    Ok(())
}
```

### Angular Development

#### Code Style

```bash
# Lint
npm run lint

# Format (if using prettier)
npm run format
```

#### Adding Dependencies

```bash
npm install <package-name>
```

#### Component Generation

```bash
ng generate component components/my-component
```

#### Service Generation

```bash
ng generate service services/my-service
```

#### Type Safety

Always define interfaces for Tauri command responses:

```typescript
interface Sound {
  id: string;
  name: string;
  // ...
}

const sounds = await invoke<Sound[]>('get_sounds');
```

## Audio Development

### Testing Audio Capture

```rust
// In src-tauri/src/audio/capture.rs
#[test]
fn test_microphone_capture() {
    let config = CaptureConfig::default();
    let mut capture = MicrophoneCapture::new(config).unwrap();

    capture.start().unwrap();

    // Read some samples
    let mut buffer = vec![0.0f32; 4800];
    let read = capture.read(&mut buffer);

    assert!(read > 0);
}
```

Run with:
```bash
cargo test test_microphone_capture -- --nocapture
```

### Testing Audio Playback

```rust
#[test]
fn test_sound_playback() {
    let format = AudioFormat::standard();
    let buffer = AudioDecoder::decode_file("test.mp3").unwrap();
    let player = SoundPlayer::from_buffer("test".into(), buffer, Volume::max());

    // Test playback
}
```

### Debugging Audio Issues

1. **Check device availability**:
   ```rust
   let devices = DeviceManager::list_input_devices()?;
   for device in devices {
       println!("{}", device.name);
   }
   ```

2. **Monitor audio levels**:
   ```rust
   let level = AudioLevel::from_buffer(&samples);
   println!("RMS: {} dB, Peak: {} dB", level.rms_db, level.peak_db);
   ```

3. **Enable verbose logging**:
   ```powershell
   $env:RUST_LOG="voiceboard::audio=trace"
   ```

## Building for Production

### Development Build

```bash
cd src-ui
npm run tauri build
```

This creates:
- `src-tauri/target/release/voiceboard.exe`
- Installers in `src-tauri/target/release/bundle/`

### Optimized Release Build

The `Cargo.toml` is already configured for optimized builds:

```toml
[profile.release]
codegen-units = 1
lto = true
opt-level = "z"
strip = true
```

### Creating Installers

Tauri automatically creates installers based on `tauri.conf.json`:

- **MSI**: `src-tauri/target/release/bundle/msi/`
- **NSIS**: `src-tauri/target/release/bundle/nsis/`

## Debugging

### Rust Debugging

1. **Using `dbg!` macro**:
   ```rust
   dbg!(&my_variable);
   ```

2. **Using VS Code**:
   - Install "CodeLLDB" extension
   - Add breakpoints in Rust code
   - Run "Debug" from VS Code

3. **Console output**:
   ```rust
   println!("Debug: {:?}", value);
   ```

### Angular Debugging

1. **Chrome DevTools**:
   - Press F12 in the Tauri window
   - Use Console, Network, Elements tabs

2. **Angular DevTools**:
   - Install Chrome extension
   - Inspect component tree and state

### Tauri Debugging

1. **Enable DevTools in production**:
   Edit `tauri.conf.json`:
   ```json
   {
     "tauri": {
       "allowlist": {
         "all": true
       }
     }
   }
   ```

2. **IPC Debugging**:
   ```typescript
   invoke('command', args).then(
     result => console.log('Success:', result),
     error => console.error('Error:', error)
   );
   ```

## Common Issues

### Audio Not Working

**Issue**: No audio captured from microphone

**Solutions**:
1. Check Windows microphone permissions
2. Verify correct device is selected
3. Check audio levels in Windows Sound settings
4. Try running as Administrator

**Issue**: Virtual microphone not found

**Solutions**:
1. Install VB-Audio Cable or VoiceMeeter
2. Restart computer after installation
3. Verify virtual device appears in Windows Sound settings

### Build Errors

**Issue**: `link.exe` not found

**Solution**: Install Visual Studio Build Tools with C++ workload

**Issue**: Rust compilation errors

**Solutions**:
1. Update Rust: `rustup update`
2. Clean build: `cargo clean && cargo build`
3. Check Cargo.toml syntax

**Issue**: Angular build errors

**Solutions**:
1. Delete `node_modules` and reinstall: `rm -rf node_modules && npm install`
2. Clear Angular cache: `ng cache clean`

### Runtime Errors

**Issue**: Tauri commands not working

**Solutions**:
1. Check command is registered in `main.rs`
2. Verify command signature matches
3. Check browser console for errors

**Issue**: Database errors

**Solutions**:
1. Delete database: `%APPDATA%/voiceboard/voiceboard.db`
2. Check file permissions
3. Verify SQLite is working: `cargo test storage`

## Performance Optimization

### Rust Optimization

1. **Use release builds for testing**:
   ```bash
   cargo run --release
   ```

2. **Profile with `perf`** (Linux):
   ```bash
   cargo flamegraph
   ```

3. **Reduce allocations in audio path**:
   - Reuse buffers
   - Avoid `Vec` in hot loops
   - Use `&[f32]` instead of `Vec<f32>`

### Angular Optimization

1. **Use OnPush change detection**:
   ```typescript
   @Component({
     changeDetection: ChangeDetectionStrategy.OnPush
   })
   ```

2. **Lazy load components**:
   ```typescript
   const routes = [
     { path: 'soundboard', loadComponent: () => import('./soundboard') }
   ];
   ```

3. **Throttle event handlers**:
   ```typescript
   fromEvent(slider, 'input')
     .pipe(throttleTime(100))
     .subscribe(value => updateVolume(value));
   ```

## Testing Strategy

### Unit Tests

**Rust**:
```bash
cargo test
```

**Angular**:
```bash
npm run test
```

### Integration Tests

Create tests in `src-tauri/tests/`:

```rust
#[tokio::test]
async fn test_full_playback_flow() {
    let manager = SoundboardManager::new("test.db").await.unwrap();
    let engine = AudioEngine::new(AudioConfig::default());

    // Test adding and playing a sound
    // ...
}
```

### End-to-End Tests

Use Tauri's testing framework (future enhancement).

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make changes and commit: `git commit -am 'Add feature'`
4. Push to branch: `git push origin feature/my-feature`
5. Create Pull Request

### Code Review Checklist

- [ ] Code follows project style (rustfmt, prettier)
- [ ] Tests added for new functionality
- [ ] Documentation updated
- [ ] No compiler warnings
- [ ] Builds successfully on Windows
- [ ] Manual testing completed

## Resources

- [Tauri Documentation](https://tauri.app/v2/guides/)
- [cpal Documentation](https://docs.rs/cpal/)
- [Symphonia Documentation](https://docs.rs/symphonia/)
- [Angular Documentation](https://angular.io/docs)
- [Rust Book](https://doc.rust-lang.org/book/)

## Getting Help

- Check GitHub Issues
- Join Discord server (if available)
- Read architecture documentation
- Review API reference

---

**Next Steps**: After setting up your development environment, see [AUDIO_PIPELINE.md](./AUDIO_PIPELINE.md) for details on implementing the audio processing pipeline.
