# Rust Tests Design

## Overview

Add comprehensive unit and integration tests for the Rust backend, focusing on AudioEngine, MixerService, and end-to-end flows.

## Structure

```
src-tauri/
├── src/
│   ├── application/
│   │   ├── audio_engine.rs      # Inline unit tests
│   │   └── services.rs          # Inline unit tests with mocks
│   └── ...
└── tests/                        # Integration tests
    ├── common/
    │   └── mod.rs               # Shared helpers and utilities
    ├── audio_engine_test.rs     # AudioEngine integration tests
    └── mixer_flow_test.rs       # End-to-end flow tests
```

## Dependencies

Add to `Cargo.toml`:

```toml
[dev-dependencies]
mockall = "0.13"
```

## Test Categories

### 1. AudioEngine Unit Tests

Location: `src/application/audio_engine.rs`

Tests to add:
- `test_send_command_when_running` - Verify command sending returns Ok
- `test_mic_volume_clamping` - SetMicVolume clamps between 0.0 and 2.0
- `test_master_volume_clamping` - SetMasterVolume clamps between 0.0 and 2.0
- `test_shutdown_stops_engine` - After shutdown, is_running() == false
- `test_play_sound_with_samples` - PlaySound accepts valid samples
- `test_stop_sound_by_id` - StopSound removes sound from list

### 2. MixerService Unit Tests with Mocks

Location: `src/application/services.rs`

Uses `mockall` to generate mocks for AudioInput, AudioOutput, DeviceManager traits.

Tests to add:
- `test_mixer_service_start_stop` - Create service with mocks, verify start/stop
- `test_add_remove_channel` - Add channel, verify in config, remove
- `test_set_channel_volume` - Set volume, verify in config
- `test_channel_not_found_error` - Operation on non-existent channel returns error
- `test_master_volume_clamping` - Master volume clamped between 0.0 and 1.0

### 3. Integration Tests

Location: `tests/`

**Common helpers (`tests/common/mod.rs`):**
- `wait_for_event()` - Wait for specific event with timeout
- `create_test_samples()` - Generate test audio samples

**AudioEngine tests (`tests/audio_engine_test.rs`):**
- `test_engine_lifecycle` - Create, verify not running, shutdown
- `test_volume_commands_accepted` - Volume commands accepted without error
- `test_play_stop_sound` - Play and stop sound commands work

## Notes

- Real audio device tests are difficult without hardware
- Focus on command/event logic, not actual audio flow
- Use mocks to isolate units from I/O dependencies
- Async tests use `#[tokio::test]`
