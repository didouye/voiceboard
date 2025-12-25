# CI & Testing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Set up GitHub Actions CI with build, lint, and tests passing on all branches.

**Architecture:** Create CI workflow that runs on all branches, fix broken test in services.rs by extracting mix_buffers as a free function, fix clippy warnings.

**Tech Stack:** GitHub Actions, Rust (cargo, clippy), Node.js (npm)

---

## Task 1: Create GitHub Actions Workflow

**Files:**
- Create: `.github/workflows/ci.yml`

**Step 1: Create the workflows directory**

Run: `mkdir -p .github/workflows`

**Step 2: Create the CI workflow file**

Create `.github/workflows/ci.yml`:

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
    defaults:
      run:
        working-directory: src-tauri
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Cache cargo
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri -> target

      - name: Build
        run: cargo build --release

      - name: Clippy
        run: cargo clippy -- -D warnings

      - name: Test
        run: cargo test

  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'

      - name: Install dependencies
        run: npm ci

      - name: Build
        run: npm run build
```

**Step 3: Verify file created**

Run: `cat .github/workflows/ci.yml`

Expected: File contents displayed

**Step 4: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add GitHub Actions workflow for all branches"
```

---

## Task 2: Fix mix_buffers Test in services.rs

**Files:**
- Modify: `src-tauri/src/application/services.rs:150-191`

**Step 1: Extract mix_buffers as free function and fix test**

Replace lines 150-191 in `src-tauri/src/application/services.rs` with:

```rust
    /// Get current mixer configuration
    pub async fn get_config(&self) -> MixerConfig {
        self.config.read().await.clone()
    }
}

/// Mix multiple audio buffers together
///
/// Takes a slice of buffers and corresponding weights, returns mixed result.
/// Returns None if buffers is empty or lengths don't match.
pub fn mix_buffers(buffers: &[AudioBuffer], weights: &[f32]) -> Option<AudioBuffer> {
    if buffers.is_empty() || buffers.len() != weights.len() {
        return None;
    }

    let first = &buffers[0];
    let mut result = first.clone();

    for buffer in buffers.iter().skip(1) {
        if let Ok(mixed) = result.mix(buffer) {
            result = mixed;
        }
    }

    // Apply weights (simplified - in practice would be per-sample)
    let total_weight: f32 = weights.iter().sum();
    if total_weight > 0.0 {
        result.apply_gain(1.0 / total_weight);
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mix_buffers_empty() {
        let result = mix_buffers(&[], &[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_mix_buffers_mismatched_lengths() {
        let result = mix_buffers(&[], &[1.0]);
        assert!(result.is_none());
    }
}
```

**Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`

Expected: Compilation succeeds (warnings OK for now)

**Step 3: Run the specific test**

Run: `cd src-tauri && cargo test mix_buffers`

Expected:
```
running 2 tests
test application::services::tests::test_mix_buffers_empty ... ok
test application::services::tests::test_mix_buffers_mismatched_lengths ... ok
```

**Step 4: Commit**

```bash
git add src-tauri/src/application/services.rs
git commit -m "fix: extract mix_buffers as free function to fix test compilation"
```

---

## Task 3: Fix Clippy Warnings

**Files:**
- Modify: `src-tauri/src/application/services.rs`
- Modify: `src-tauri/src/application/audio_engine.rs`

**Step 1: Check current clippy warnings**

Run: `cd src-tauri && cargo clippy 2>&1 | grep -E "^warning:|^error:"`

Expected: List of warnings

**Step 2: Fix unused variable in services.rs**

In `src-tauri/src/application/services.rs`, the `mix_buffers` function has unused `i` variable. Since we already rewrote the function in Task 2, verify it's fixed.

If not fixed, change:
```rust
for (i, buffer) in buffers.iter().enumerate().skip(1) {
```
To:
```rust
for buffer in buffers.iter().skip(1) {
```

**Step 3: Fix unused fields warning**

In `src-tauri/src/application/audio_engine.rs`, add `#[allow(dead_code)]` to unused fields in `AudioState` struct (lines 67-72):

```rust
/// Shared state for audio processing
#[allow(dead_code)]
struct AudioState {
    playing_sounds: HashMap<String, PlayingSound>,
    mic_volume: f32,
    master_volume: f32,
    mic_muted: bool,
}
```

**Step 4: Fix unused field in services.rs**

In `src-tauri/src/application/services.rs`, add `#[allow(dead_code)]` to `device_manager` field:

```rust
pub struct MixerService<I, O, D>
where
    I: AudioInput,
    O: AudioOutput,
    D: DeviceManager,
{
    input: Arc<RwLock<I>>,
    output: Arc<RwLock<O>>,
    #[allow(dead_code)]
    device_manager: Arc<D>,
    config: Arc<RwLock<MixerConfig>>,
    is_running: Arc<RwLock<bool>>,
}
```

**Step 5: Fix unused assignments in audio_engine.rs**

In `src-tauri/src/application/audio_engine.rs` around lines 441-442, the assignments before return are unused. Remove them or prefix with `_`:

Find:
```rust
                        input_stream = None;
                        output_stream = None;
```

Replace with (just remove these lines since we return immediately after):
```rust
                        // Streams will be dropped when function returns
```

**Step 6: Verify clippy passes**

Run: `cd src-tauri && cargo clippy -- -D warnings`

Expected: No errors, no warnings

**Step 7: Commit**

```bash
git add src-tauri/src/application/services.rs src-tauri/src/application/audio_engine.rs
git commit -m "fix: resolve clippy warnings for CI compliance"
```

---

## Task 4: Run All Tests

**Files:** None (verification only)

**Step 1: Run cargo test**

Run: `cd src-tauri && cargo test`

Expected: All tests pass

```
running X tests
...
test result: ok. X passed; 0 failed; 0 ignored
```

**Step 2: Count passing tests**

Run: `cd src-tauri && cargo test 2>&1 | grep -E "^test .* ok$" | wc -l`

Expected: Number > 0

**Step 3: Run frontend build**

Run: `npm run build`

Expected: Build succeeds

**Step 4: Commit if any pending changes**

```bash
git status
# If clean, skip. Otherwise:
git add -A
git commit -m "fix: ensure all tests pass"
```

---

## Task 5: Update ROADMAP

**Files:**
- Modify: `ROADMAP.md`

**Step 1: Add CI to done items in Phase 2**

Find the Phase 2 section and update GitHub Actions CI from "To Do" to have a checkmark:

```markdown
## Phase 2 - Distribution & CI/CD

### Done
- [x] **GitHub Actions CI**
  - Automated build and compilation
  - Clippy linting
  - Automated tests

### To Do
- [ ] **Windows Installer**
...
```

**Step 2: Commit**

```bash
git add ROADMAP.md
git commit -m "docs: mark GitHub Actions CI as complete"
```

---

## Task 6: Push and Verify CI

**Files:** None

**Step 1: Push to remote**

Run: `git push`

**Step 2: Check GitHub Actions**

Open: `https://github.com/<owner>/voiceboard/actions`

Expected: CI workflow running, all jobs green

**Step 3: Configure branch protection (manual)**

In GitHub:
1. Go to Settings → Branches → Add rule
2. Branch name pattern: `develop`
3. Enable:
   - ✅ Require status checks to pass before merging
   - ✅ Require branches to be up to date before merging
4. Add required checks: `rust`, `frontend`
5. Save

---

## Summary

| Task | Description | Commit |
|------|-------------|--------|
| 1 | Create CI workflow | `ci: add GitHub Actions workflow` |
| 2 | Fix mix_buffers test | `fix: extract mix_buffers as free function` |
| 3 | Fix clippy warnings | `fix: resolve clippy warnings` |
| 4 | Verify all tests pass | (verification) |
| 5 | Update ROADMAP | `docs: mark CI as complete` |
| 6 | Push and verify | (manual GitHub setup) |
