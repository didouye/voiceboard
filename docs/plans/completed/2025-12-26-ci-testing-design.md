# CI & Testing Strategy Design

## Overview

Set up a CI pipeline with GitHub Actions that validates every branch and ensures develop is protected. Fix broken tests to achieve 100% pass rate.

## Requirements

- CI runs on all branches (push + PR)
- GitFlow workflow: feature/* → develop → release/* → main
- Branch protection on develop: require CI pass before merge
- `cargo test` passes 100% on Ubuntu CI (no hardware)

## CI Configuration

### Workflow File: `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: ['**']
  pull_request:
    branches: ['**']

jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri
      - name: Build
        run: cargo build --release
        working-directory: src-tauri
      - name: Clippy
        run: cargo clippy -- -D warnings
        working-directory: src-tauri
      - name: Test
        run: cargo test
        working-directory: src-tauri

  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      - run: npm ci
      - run: npm run build
```

### Branch Protection (Manual GitHub Setup)

1. Go to Settings → Branches → Add rule
2. Branch pattern: `develop`
3. Enable:
   - ✅ Require status checks to pass before merging
   - ✅ Require branches to be up to date before merging
4. Select required checks: `rust`, `frontend`

## Fixing Broken Tests

### Problem

In `src-tauri/src/application/services.rs`, the test uses `CpalAudioInput` as an `AudioOutput` placeholder, which doesn't implement the trait.

### Solution

Extract `mix_buffers` as a standalone function (it's static and doesn't need the generic types):

```rust
// Before (inside impl<I, O, D>):
pub fn mix_buffers(buffers: &[AudioBuffer], weights: &[f32]) -> Option<AudioBuffer>

// After (free function):
pub fn mix_buffers(buffers: &[AudioBuffer], weights: &[f32]) -> Option<AudioBuffer> {
    // ... same implementation ...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix_buffers_empty() {
        let result = mix_buffers(&[], &[]);
        assert!(result.is_none());
    }
}
```

### Hardware-Dependent Tests

Tests in `adapters/*` that require audio hardware should be marked with `#[ignore]`:

```rust
#[test]
#[ignore] // Requires audio hardware
fn test_cpal_device_enumeration() {
    // ...
}
```

Run locally with: `cargo test -- --include-ignored`

## Test Coverage by Module

| Module | Status | Action |
|--------|--------|--------|
| `domain/audio/sample.rs` | ✅ | Verify passes |
| `domain/audio/buffer.rs` | ✅ | Verify passes |
| `domain/audio/format.rs` | ✅ | Verify passes |
| `domain/mixer/channel.rs` | ✅ | Verify passes |
| `domain/mixer/mixer_config.rs` | ✅ | Verify passes |
| `domain/settings.rs` | ✅ | Verify passes |
| `application/services.rs` | ❌ | Fix with free function |
| `application/audio_engine.rs` | ✅ | Verify passes |
| `adapters/*` | ⚠️ | Mark `#[ignore]` if hardware-dependent |

## Success Criteria

- `cargo build --release` passes
- `cargo clippy -- -D warnings` passes (no warnings)
- `cargo test` passes 100%
- `npm run build` passes
- All checks green in GitHub Actions
