# YouTube Audio Import - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Allow users to import audio from YouTube videos with a visual trimming editor.

**Architecture:** Frontend modal with URL input → Backend downloads audio via yt-dlp → Frontend displays waveform with wavesurfer.js → User trims → Backend trims with ffmpeg and normalizes → Sound added to soundboard.

**Tech Stack:** yt-dlp (bundled), ffmpeg (bundled), wavesurfer.js, Tauri commands

---

## Task 1: Create Binary Download Script for CI

**Files:**
- Create: `.github/scripts/download-binaries.sh`

**Step 1: Create the download script**

```bash
#!/bin/bash
# Download yt-dlp and ffmpeg binaries for all platforms
set -e

BINARIES_DIR="src-tauri/binaries"
mkdir -p "$BINARIES_DIR"

# Get target from argument or detect
TARGET="${1:-}"

download_ytdlp() {
    local target=$1
    local ext=""
    local ytdlp_name=""

    case "$target" in
        x86_64-pc-windows-msvc)
            ytdlp_name="yt-dlp.exe"
            ext=".exe"
            ;;
        x86_64-apple-darwin)
            ytdlp_name="yt-dlp_macos"
            ;;
        aarch64-apple-darwin)
            ytdlp_name="yt-dlp_macos"
            ;;
        x86_64-unknown-linux-gnu)
            ytdlp_name="yt-dlp_linux"
            ;;
        *)
            echo "Unknown target: $target"
            exit 1
            ;;
    esac

    local output="$BINARIES_DIR/yt-dlp-${target}${ext}"

    if [ ! -f "$output" ]; then
        echo "Downloading yt-dlp for $target..."
        curl -L "https://github.com/yt-dlp/yt-dlp/releases/latest/download/${ytdlp_name}" -o "$output"
        chmod +x "$output"
    else
        echo "yt-dlp for $target already exists"
    fi
}

download_ffmpeg() {
    local target=$1
    local ext=""

    case "$target" in
        x86_64-pc-windows-msvc)
            ext=".exe"
            echo "Downloading ffmpeg for Windows..."
            curl -L "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip" -o /tmp/ffmpeg.zip
            unzip -o /tmp/ffmpeg.zip -d /tmp/ffmpeg
            cp /tmp/ffmpeg/ffmpeg-master-latest-win64-gpl/bin/ffmpeg.exe "$BINARIES_DIR/ffmpeg-${target}.exe"
            rm -rf /tmp/ffmpeg /tmp/ffmpeg.zip
            ;;
        x86_64-apple-darwin)
            echo "Downloading ffmpeg for macOS x64..."
            curl -L "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip" -o /tmp/ffmpeg.zip
            unzip -o /tmp/ffmpeg.zip -d /tmp/ffmpeg
            cp /tmp/ffmpeg/ffmpeg "$BINARIES_DIR/ffmpeg-${target}"
            chmod +x "$BINARIES_DIR/ffmpeg-${target}"
            rm -rf /tmp/ffmpeg /tmp/ffmpeg.zip
            ;;
        aarch64-apple-darwin)
            echo "Downloading ffmpeg for macOS ARM..."
            curl -L "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip" -o /tmp/ffmpeg.zip
            unzip -o /tmp/ffmpeg.zip -d /tmp/ffmpeg
            cp /tmp/ffmpeg/ffmpeg "$BINARIES_DIR/ffmpeg-${target}"
            chmod +x "$BINARIES_DIR/ffmpeg-${target}"
            rm -rf /tmp/ffmpeg /tmp/ffmpeg.zip
            ;;
        x86_64-unknown-linux-gnu)
            echo "Downloading ffmpeg for Linux..."
            curl -L "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz" -o /tmp/ffmpeg.tar.xz
            tar -xf /tmp/ffmpeg.tar.xz -C /tmp
            cp /tmp/ffmpeg-*-amd64-static/ffmpeg "$BINARIES_DIR/ffmpeg-${target}"
            chmod +x "$BINARIES_DIR/ffmpeg-${target}"
            rm -rf /tmp/ffmpeg-* /tmp/ffmpeg.tar.xz
            ;;
    esac
}

if [ -n "$TARGET" ]; then
    download_ytdlp "$TARGET"
    download_ffmpeg "$TARGET"
else
    echo "Usage: $0 <target>"
    echo "Targets: x86_64-pc-windows-msvc, x86_64-apple-darwin, aarch64-apple-darwin, x86_64-unknown-linux-gnu"
    exit 1
fi

echo "Binaries downloaded to $BINARIES_DIR"
ls -la "$BINARIES_DIR"
```

**Step 2: Make script executable and commit**

```bash
chmod +x .github/scripts/download-binaries.sh
git add .github/scripts/download-binaries.sh
git commit -m "ci: add binary download script for yt-dlp and ffmpeg"
```

---

## Task 2: Update CI Workflow to Download Binaries

**Files:**
- Modify: `.github/workflows/release.yml`

**Step 1: Add download step after Linux dependencies**

In `.github/workflows/release.yml`, add after the "Install Linux dependencies" step (around line 100):

```yaml
      - name: Download external binaries
        run: |
          chmod +x .github/scripts/download-binaries.sh
          .github/scripts/download-binaries.sh ${{ matrix.target }}
```

**Step 2: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: download yt-dlp and ffmpeg binaries before build"
```

---

## Task 3: Configure Tauri External Binaries

**Files:**
- Modify: `src-tauri/tauri.conf.json`

**Step 1: Add externalBin configuration**

Add the `externalBin` array inside the `bundle` object:

```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "externalBin": [
      "binaries/yt-dlp",
      "binaries/ffmpeg"
    ],
    "icon": [
```

**Step 2: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "feat: configure yt-dlp and ffmpeg as external binaries"
```

---

## Task 4: Create YouTube DTOs

**Files:**
- Modify: `src-tauri/src/application/commands.rs`

**Step 1: Add YouTubeAudioDto struct**

Add after the existing DTO structs (around line 50):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YouTubeAudioDto {
    pub temp_path: String,
    pub title: String,
    pub duration: f64,
    pub video_id: String,
}
```

**Step 2: Commit**

```bash
git add src-tauri/src/application/commands.rs
git commit -m "feat(youtube): add YouTubeAudioDto struct"
```

---

## Task 5: Implement youtube_download Command

**Files:**
- Modify: `src-tauri/src/application/commands.rs`

**Step 1: Add youtube_download command**

Add at the end of commands.rs:

```rust
/// Download audio from a YouTube URL using yt-dlp
#[tauri::command]
pub async fn youtube_download(
    app: tauri::AppHandle,
    url: String,
) -> Result<YouTubeAudioDto, String> {
    use regex::Regex;
    use std::process::Command;

    // Validate YouTube URL and extract video ID
    let video_id_regex = Regex::new(
        r"(?:youtube\.com/watch\?v=|youtu\.be/|youtube\.com/embed/)([a-zA-Z0-9_-]{11})"
    ).map_err(|e| format!("Regex error: {}", e))?;

    let video_id = video_id_regex
        .captures(&url)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or("Invalid YouTube URL")?;

    // Get temp directory
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let temp_dir = app_data_dir.join("temp").join("youtube");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    let output_path = temp_dir.join(format!("{}.mp3", video_id));

    // Get yt-dlp binary path
    let ytdlp_path = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?
        .join("binaries")
        .join(if cfg!(windows) { "yt-dlp.exe" } else { "yt-dlp" });

    // Get ffmpeg binary path (yt-dlp needs it for conversion)
    let ffmpeg_path = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?
        .join("binaries")
        .join(if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" });

    tracing::info!("Downloading YouTube audio: {} -> {:?}", video_id, output_path);

    // Run yt-dlp to download and convert to MP3
    let output = Command::new(&ytdlp_path)
        .args([
            "-x",                           // Extract audio
            "--audio-format", "mp3",        // Convert to MP3
            "--audio-quality", "0",         // Best quality
            "--ffmpeg-location", ffmpeg_path.to_str().unwrap(),
            "--no-playlist",                // Don't download playlists
            "--print", "title",             // Print title to stdout
            "--print", "duration",          // Print duration to stdout
            "-o", output_path.to_str().unwrap(),
            &url,
        ])
        .output()
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("yt-dlp failed: {}", stderr);

        // Parse common errors
        if stderr.contains("Video unavailable") || stderr.contains("Private video") {
            return Err("Video not accessible (private or deleted)".to_string());
        }
        if stderr.contains("geo restriction") || stderr.contains("not available in your country") {
            return Err("Video not available in your region".to_string());
        }
        if stderr.contains("is a live event") || stderr.contains("live stream") {
            return Err("Live streams are not supported".to_string());
        }

        return Err(format!("Failed to download: {}", stderr.lines().last().unwrap_or("Unknown error")));
    }

    // Parse output (title and duration)
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.trim().lines().collect();

    let title = lines.first().unwrap_or(&"Unknown").to_string();
    let duration: f64 = lines.get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    tracing::info!("Downloaded: {} ({:.1}s)", title, duration);

    Ok(YouTubeAudioDto {
        temp_path: output_path.to_string_lossy().to_string(),
        title,
        duration,
        video_id,
    })
}
```

**Step 2: Add regex dependency to Cargo.toml**

In `src-tauri/Cargo.toml`, add under `[dependencies]`:

```toml
regex = "1"
```

**Step 3: Register command in lib.rs**

In `src-tauri/src/lib.rs`, add `youtube_download` to the `invoke_handler`:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::youtube_download,
])
```

**Step 4: Run tests and commit**

```bash
cd src-tauri && cargo check && cd ..
git add src-tauri/src/application/commands.rs src-tauri/Cargo.toml src-tauri/src/lib.rs
git commit -m "feat(youtube): implement youtube_download command"
```

---

## Task 6: Implement youtube_trim_and_import Command

**Files:**
- Modify: `src-tauri/src/application/commands.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add youtube_trim_and_import command**

Add after youtube_download:

```rust
/// Trim audio using ffmpeg and import to soundboard
#[tauri::command]
pub async fn youtube_trim_and_import(
    app: tauri::AppHandle,
    temp_path: String,
    start_seconds: f64,
    end_seconds: f64,
    name: String,
) -> Result<ImportedSoundDto, String> {
    use std::process::Command;

    // Validate inputs
    if start_seconds < 0.0 || end_seconds <= start_seconds {
        return Err("Invalid trim range".to_string());
    }

    let duration = end_seconds - start_seconds;
    if duration < 0.5 {
        return Err("Selection must be at least 0.5 seconds".to_string());
    }
    if duration > 300.0 {
        return Err("Selection must be at most 5 minutes".to_string());
    }

    // Get temp directory for trimmed file
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let temp_dir = app_data_dir.join("temp").join("youtube");
    let trimmed_path = temp_dir.join(format!("trimmed_{}.mp3", uuid::Uuid::new_v4()));

    // Get ffmpeg binary path
    let ffmpeg_path = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?
        .join("binaries")
        .join(if cfg!(windows) { "ffmpeg.exe" } else { "ffmpeg" });

    tracing::info!(
        "Trimming audio: {:.2}s - {:.2}s -> {:?}",
        start_seconds, end_seconds, trimmed_path
    );

    // Run ffmpeg to trim
    let output = Command::new(&ffmpeg_path)
        .args([
            "-y",                           // Overwrite output
            "-i", &temp_path,               // Input file
            "-ss", &format!("{:.3}", start_seconds),  // Start time
            "-to", &format!("{:.3}", end_seconds),    // End time
            "-c", "copy",                   // Stream copy (fast, no re-encode)
            trimmed_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run ffmpeg: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("ffmpeg failed: {}", stderr);
        return Err(format!("Failed to trim audio: {}", stderr.lines().last().unwrap_or("Unknown error")));
    }

    // Import the trimmed file using existing normalization pipeline
    let result = import_and_normalize_sound(app.clone(), trimmed_path.to_string_lossy().to_string()).await?;

    // Clean up temp files
    let _ = std::fs::remove_file(&temp_path);
    let _ = std::fs::remove_file(&trimmed_path);

    // Return with custom name
    Ok(ImportedSoundDto {
        hash: result.hash,
        name: if name.is_empty() { result.name } else { name },
        path: result.path,
        duration: result.duration,
    })
}
```

**Step 2: Add uuid dependency to Cargo.toml**

In `src-tauri/Cargo.toml`, add under `[dependencies]`:

```toml
uuid = { version = "1", features = ["v4"] }
```

**Step 3: Register command in lib.rs**

Add `youtube_trim_and_import` to the invoke_handler.

**Step 4: Run tests and commit**

```bash
cd src-tauri && cargo check && cd ..
git add src-tauri/src/application/commands.rs src-tauri/Cargo.toml src-tauri/src/lib.rs
git commit -m "feat(youtube): implement youtube_trim_and_import command"
```

---

## Task 7: Implement youtube_cancel Command

**Files:**
- Modify: `src-tauri/src/application/commands.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Add youtube_cancel command**

```rust
/// Cancel YouTube import and cleanup temp file
#[tauri::command]
pub async fn youtube_cancel(temp_path: String) -> Result<(), String> {
    if !temp_path.is_empty() && std::path::Path::new(&temp_path).exists() {
        std::fs::remove_file(&temp_path)
            .map_err(|e| format!("Failed to delete temp file: {}", e))?;
        tracing::info!("Cleaned up temp file: {}", temp_path);
    }
    Ok(())
}
```

**Step 2: Register command in lib.rs**

Add `youtube_cancel` to the invoke_handler.

**Step 3: Add temp cleanup on startup**

In `src-tauri/src/lib.rs`, add cleanup function in the setup hook:

```rust
// In run() function, after existing setup code:
// Cleanup old YouTube temp files (>24h)
let temp_dir = app.path().app_data_dir()?.join("temp").join("youtube");
if temp_dir.exists() {
    let now = std::time::SystemTime::now();
    if let Ok(entries) = std::fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(age) = now.duration_since(modified) {
                        if age.as_secs() > 24 * 60 * 60 {
                            let _ = std::fs::remove_file(entry.path());
                        }
                    }
                }
            }
        }
    }
}
```

**Step 4: Commit**

```bash
git add src-tauri/src/application/commands.rs src-tauri/src/lib.rs
git commit -m "feat(youtube): implement youtube_cancel and temp cleanup"
```

---

## Task 8: Install wavesurfer.js

**Files:**
- Modify: `package.json`

**Step 1: Install wavesurfer.js**

```bash
npm install wavesurfer.js
```

**Step 2: Commit**

```bash
git add package.json package-lock.json
git commit -m "feat(youtube): add wavesurfer.js dependency"
```

---

## Task 9: Create YouTubeService

**Files:**
- Create: `src/app/core/services/youtube.service.ts`

**Step 1: Create the service**

```typescript
import { Injectable, inject } from '@angular/core';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';

export interface YouTubeAudioDto {
  temp_path: string;
  title: string;
  duration: number;
  video_id: string;
}

@Injectable({ providedIn: 'root' })
export class YouTubeService {

  /**
   * Validate YouTube URL format
   */
  isValidUrl(url: string): boolean {
    const pattern = /(?:youtube\.com\/watch\?v=|youtu\.be\/|youtube\.com\/embed\/)([a-zA-Z0-9_-]{11})/;
    return pattern.test(url);
  }

  /**
   * Download audio from YouTube URL
   */
  async download(url: string): Promise<YouTubeAudioDto> {
    return invoke<YouTubeAudioDto>('youtube_download', { url });
  }

  /**
   * Trim audio and import to soundboard
   */
  async trimAndImport(
    tempPath: string,
    startSeconds: number,
    endSeconds: number,
    name: string
  ): Promise<{ hash: string; name: string; path: string; duration: number }> {
    return invoke('youtube_trim_and_import', {
      tempPath,
      startSeconds,
      endSeconds,
      name,
    });
  }

  /**
   * Cancel import and cleanup temp file
   */
  async cancel(tempPath: string): Promise<void> {
    return invoke('youtube_cancel', { tempPath });
  }

  /**
   * Get asset URL for audio file (for wavesurfer)
   */
  getAudioUrl(tempPath: string): string {
    return convertFileSrc(tempPath);
  }
}
```

**Step 2: Commit**

```bash
git add src/app/core/services/youtube.service.ts
git commit -m "feat(youtube): create YouTubeService"
```

---

## Task 10: Create AudioTrimmerComponent

**Files:**
- Create: `src/app/features/soundboard/youtube-import/audio-trimmer.component.ts`

**Step 1: Create the component**

```typescript
import {
  Component,
  Input,
  Output,
  EventEmitter,
  OnInit,
  OnDestroy,
  ElementRef,
  ViewChild,
  signal,
  computed,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import WaveSurfer from 'wavesurfer.js';
import RegionsPlugin from 'wavesurfer.js/dist/plugins/regions.js';

@Component({
  selector: 'app-audio-trimmer',
  standalone: true,
  imports: [CommonModule, FormsModule],
  template: `
    <div class="space-y-4">
      <!-- Waveform container -->
      <div
        #waveformContainer
        class="w-full h-32 bg-surface-hover rounded-lg overflow-hidden"
      ></div>

      <!-- Time inputs -->
      <div class="flex items-center gap-4">
        <div class="flex-1">
          <label class="text-xs text-text-muted block mb-1">Start (s)</label>
          <input
            type="number"
            [ngModel]="startTime()"
            (ngModelChange)="onStartTimeChange($event)"
            min="0"
            [max]="endTime() - 0.5"
            step="0.1"
            class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary"
          />
        </div>
        <div class="flex-1">
          <label class="text-xs text-text-muted block mb-1">End (s)</label>
          <input
            type="number"
            [ngModel]="endTime()"
            (ngModelChange)="onEndTimeChange($event)"
            [min]="startTime() + 0.5"
            [max]="duration"
            step="0.1"
            class="w-full px-3 py-2 bg-surface border border-border rounded text-sm text-text-primary"
          />
        </div>
        <div class="flex-1">
          <label class="text-xs text-text-muted block mb-1">Duration</label>
          <div class="px-3 py-2 bg-surface-hover border border-border rounded text-sm text-text-primary">
            {{ formatDuration(selectedDuration()) }}
          </div>
        </div>
      </div>

      <!-- Playback controls -->
      <div class="flex items-center justify-center gap-2">
        <button
          class="px-4 py-2 bg-accent hover:bg-accent/80 text-white rounded text-sm transition-colors"
          (click)="playSelection()"
        >
          {{ isPlaying() ? '⏸ Pause' : '▶ Play Selection' }}
        </button>
        <button
          class="px-4 py-2 bg-surface-hover hover:bg-border text-text-secondary rounded text-sm transition-colors"
          (click)="resetSelection()"
        >
          ↺ Reset
        </button>
      </div>
    </div>
  `,
})
export class AudioTrimmerComponent implements OnInit, OnDestroy {
  @ViewChild('waveformContainer', { static: true }) waveformContainer!: ElementRef;

  @Input() audioUrl!: string;
  @Input() duration!: number;

  @Output() selectionChange = new EventEmitter<{ start: number; end: number }>();

  startTime = signal(0);
  endTime = signal(0);
  isPlaying = signal(false);

  selectedDuration = computed(() => this.endTime() - this.startTime());

  private wavesurfer: WaveSurfer | null = null;
  private regionsPlugin: RegionsPlugin | null = null;
  private activeRegion: any = null;

  ngOnInit(): void {
    this.endTime.set(Math.min(this.duration, 30)); // Default 30s or full duration
    this.initWavesurfer();
  }

  ngOnDestroy(): void {
    this.wavesurfer?.destroy();
  }

  private initWavesurfer(): void {
    this.regionsPlugin = RegionsPlugin.create();

    this.wavesurfer = WaveSurfer.create({
      container: this.waveformContainer.nativeElement,
      waveColor: '#6366f1',
      progressColor: '#818cf8',
      cursorColor: '#c084fc',
      height: 128,
      normalize: true,
      plugins: [this.regionsPlugin],
    });

    this.wavesurfer.load(this.audioUrl);

    this.wavesurfer.on('ready', () => {
      this.createRegion();
    });

    this.wavesurfer.on('play', () => this.isPlaying.set(true));
    this.wavesurfer.on('pause', () => this.isPlaying.set(false));
    this.wavesurfer.on('finish', () => this.isPlaying.set(false));

    this.regionsPlugin.on('region-updated', (region: any) => {
      this.startTime.set(Math.round(region.start * 10) / 10);
      this.endTime.set(Math.round(region.end * 10) / 10);
      this.emitSelection();
    });
  }

  private createRegion(): void {
    if (!this.regionsPlugin) return;

    this.activeRegion = this.regionsPlugin.addRegion({
      start: this.startTime(),
      end: this.endTime(),
      color: 'rgba(139, 92, 246, 0.3)',
      drag: true,
      resize: true,
    });
  }

  onStartTimeChange(value: number): void {
    const newStart = Math.max(0, Math.min(value, this.endTime() - 0.5));
    this.startTime.set(newStart);
    this.updateRegion();
    this.emitSelection();
  }

  onEndTimeChange(value: number): void {
    const newEnd = Math.max(this.startTime() + 0.5, Math.min(value, this.duration));
    this.endTime.set(newEnd);
    this.updateRegion();
    this.emitSelection();
  }

  private updateRegion(): void {
    if (this.activeRegion) {
      this.activeRegion.setOptions({
        start: this.startTime(),
        end: this.endTime(),
      });
    }
  }

  private emitSelection(): void {
    this.selectionChange.emit({
      start: this.startTime(),
      end: this.endTime(),
    });
  }

  playSelection(): void {
    if (!this.wavesurfer) return;

    if (this.isPlaying()) {
      this.wavesurfer.pause();
    } else {
      this.wavesurfer.setTime(this.startTime());
      this.wavesurfer.play();

      // Stop at end time
      const checkEnd = setInterval(() => {
        if (this.wavesurfer && this.wavesurfer.getCurrentTime() >= this.endTime()) {
          this.wavesurfer.pause();
          clearInterval(checkEnd);
        }
      }, 50);
    }
  }

  resetSelection(): void {
    this.startTime.set(0);
    this.endTime.set(Math.min(this.duration, 30));
    this.updateRegion();
    this.emitSelection();
  }

  formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    const ms = Math.floor((seconds % 1) * 10);
    return `${mins}:${secs.toString().padStart(2, '0')}.${ms}`;
  }
}
```

**Step 2: Commit**

```bash
mkdir -p src/app/features/soundboard/youtube-import
git add src/app/features/soundboard/youtube-import/audio-trimmer.component.ts
git commit -m "feat(youtube): create AudioTrimmerComponent with wavesurfer.js"
```

---

## Task 11: Create YouTubeImportModalComponent

**Files:**
- Create: `src/app/features/soundboard/youtube-import/youtube-import-modal.component.ts`

**Step 1: Create the component**

```typescript
import {
  Component,
  Output,
  EventEmitter,
  signal,
  inject,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { YouTubeService, YouTubeAudioDto } from '../../../core/services/youtube.service';
import { AudioTrimmerComponent } from './audio-trimmer.component';

type ModalState = 'idle' | 'downloading' | 'editing' | 'importing';

@Component({
  selector: 'app-youtube-import-modal',
  standalone: true,
  imports: [CommonModule, FormsModule, AudioTrimmerComponent],
  template: `
    <!-- Backdrop -->
    <div
      class="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center"
      (click)="onBackdropClick($event)"
    >
      <!-- Modal -->
      <div
        class="bg-surface border border-border rounded-xl shadow-2xl w-full max-w-xl mx-4 overflow-hidden"
        (click)="$event.stopPropagation()"
      >
        <!-- Header -->
        <div class="flex items-center justify-between px-6 py-4 border-b border-border">
          <h2 class="text-lg font-semibold text-text-primary">
            Import from YouTube
          </h2>
          <button
            class="text-text-muted hover:text-text-primary transition-colors"
            (click)="onClose()"
          >
            ✕
          </button>
        </div>

        <!-- Content -->
        <div class="p-6">
          @switch (state()) {
            @case ('idle') {
              <!-- URL Input -->
              <div class="space-y-4">
                <div>
                  <label class="text-sm text-text-muted block mb-2">YouTube URL</label>
                  <input
                    type="url"
                    [(ngModel)]="url"
                    placeholder="https://youtube.com/watch?v=..."
                    class="w-full px-4 py-3 bg-surface-hover border border-border rounded-lg text-text-primary placeholder-text-muted focus:border-accent focus:outline-none"
                    (keydown.enter)="onDownload()"
                  />
                </div>
                @if (error()) {
                  <p class="text-sm text-status-error">{{ error() }}</p>
                }
                <button
                  class="w-full px-4 py-3 bg-accent hover:bg-accent/80 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  [disabled]="!isValidUrl()"
                  (click)="onDownload()"
                >
                  Download Audio
                </button>
              </div>
            }

            @case ('downloading') {
              <!-- Progress -->
              <div class="text-center py-8 space-y-4">
                <div class="animate-spin w-12 h-12 border-4 border-accent border-t-transparent rounded-full mx-auto"></div>
                <p class="text-text-primary">Downloading audio...</p>
                <p class="text-sm text-text-muted">This may take a moment</p>
              </div>
            }

            @case ('editing') {
              <!-- Trimmer -->
              <div class="space-y-4">
                <div>
                  <h3 class="font-medium text-text-primary">{{ audioData()?.title }}</h3>
                  <p class="text-sm text-text-muted">
                    Duration: {{ formatDuration(audioData()?.duration || 0) }}
                  </p>
                </div>

                <app-audio-trimmer
                  [audioUrl]="audioUrl()"
                  [duration]="audioData()?.duration || 0"
                  (selectionChange)="onSelectionChange($event)"
                />

                <div>
                  <label class="text-sm text-text-muted block mb-2">Sound name</label>
                  <input
                    type="text"
                    [(ngModel)]="soundName"
                    [placeholder]="audioData()?.title || 'Sound name'"
                    class="w-full px-4 py-3 bg-surface-hover border border-border rounded-lg text-text-primary placeholder-text-muted focus:border-accent focus:outline-none"
                  />
                </div>

                <div class="flex gap-3">
                  <button
                    class="flex-1 px-4 py-3 bg-surface-hover hover:bg-border text-text-secondary rounded-lg font-medium transition-colors"
                    (click)="onCancel()"
                  >
                    Cancel
                  </button>
                  <button
                    class="flex-1 px-4 py-3 bg-accent hover:bg-accent/80 text-white rounded-lg font-medium transition-colors"
                    (click)="onImport()"
                  >
                    Import Sound
                  </button>
                </div>
              </div>
            }

            @case ('importing') {
              <!-- Importing progress -->
              <div class="text-center py-8 space-y-4">
                <div class="animate-spin w-12 h-12 border-4 border-accent border-t-transparent rounded-full mx-auto"></div>
                <p class="text-text-primary">Importing sound...</p>
              </div>
            }
          }
        </div>
      </div>
    </div>
  `,
})
export class YouTubeImportModalComponent {
  private youtube = inject(YouTubeService);

  @Output() close = new EventEmitter<void>();
  @Output() imported = new EventEmitter<{ hash: string; name: string; path: string; duration: number }>();

  state = signal<ModalState>('idle');
  error = signal<string | null>(null);
  audioData = signal<YouTubeAudioDto | null>(null);
  audioUrl = signal<string>('');

  url = '';
  soundName = '';
  selection = { start: 0, end: 30 };

  isValidUrl(): boolean {
    return this.youtube.isValidUrl(this.url);
  }

  async onDownload(): Promise<void> {
    if (!this.isValidUrl()) return;

    this.state.set('downloading');
    this.error.set(null);

    try {
      const data = await this.youtube.download(this.url);
      this.audioData.set(data);
      this.audioUrl.set(this.youtube.getAudioUrl(data.temp_path));
      this.soundName = data.title;
      this.selection.end = Math.min(data.duration, 30);
      this.state.set('editing');
    } catch (err: any) {
      this.error.set(err?.message || err || 'Download failed');
      this.state.set('idle');
    }
  }

  onSelectionChange(selection: { start: number; end: number }): void {
    this.selection = selection;
  }

  async onImport(): Promise<void> {
    const data = this.audioData();
    if (!data) return;

    this.state.set('importing');

    try {
      const result = await this.youtube.trimAndImport(
        data.temp_path,
        this.selection.start,
        this.selection.end,
        this.soundName || data.title
      );
      this.imported.emit(result);
      this.close.emit();
    } catch (err: any) {
      this.error.set(err?.message || err || 'Import failed');
      this.state.set('editing');
    }
  }

  async onCancel(): Promise<void> {
    const data = this.audioData();
    if (data) {
      await this.youtube.cancel(data.temp_path);
    }
    this.close.emit();
  }

  onClose(): void {
    this.onCancel();
  }

  onBackdropClick(event: MouseEvent): void {
    if (event.target === event.currentTarget) {
      this.onCancel();
    }
  }

  formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
}
```

**Step 2: Commit**

```bash
git add src/app/features/soundboard/youtube-import/youtube-import-modal.component.ts
git commit -m "feat(youtube): create YouTubeImportModalComponent"
```

---

## Task 12: Integrate YouTube Button in Soundboard

**Files:**
- Modify: `src/app/features/soundboard/soundboard.component.ts`

**Step 1: Add imports**

Add to imports array:

```typescript
import { YouTubeImportModalComponent } from './youtube-import/youtube-import-modal.component';
```

And in the `imports` array of the component:

```typescript
imports: [
  // ... existing imports
  YouTubeImportModalComponent,
],
```

**Step 2: Add signal for modal visibility**

Add after existing signals:

```typescript
showYouTubeModal = signal(false);
```

**Step 3: Add YouTube button in template**

Replace the footer section (around line 148) with:

```html
<!-- Footer -->
<div class="mt-4 pb-4 flex justify-center gap-3">
  <button
    class="px-6 py-3 bg-surface-hover border border-dashed border-border hover:border-accent text-text-secondary hover:text-text-primary rounded-lg text-sm transition-all flex items-center gap-2"
    [class.opacity-50]="soundboard.loading()"
    [class.cursor-not-allowed]="soundboard.loading()"
    [disabled]="soundboard.loading()"
    (click)="importMultiple()"
  >
    <span>📁</span>
    Import Multiple
  </button>
  <button
    class="px-6 py-3 bg-surface-hover border border-dashed border-border hover:border-red-500 text-text-secondary hover:text-red-400 rounded-lg text-sm transition-all flex items-center gap-2"
    [class.opacity-50]="soundboard.loading()"
    [class.cursor-not-allowed]="soundboard.loading()"
    [disabled]="soundboard.loading()"
    (click)="showYouTubeModal.set(true)"
  >
    <span>▶</span>
    YouTube
  </button>
</div>
```

**Step 4: Add modal at end of template**

Add before the closing `</div>` of the template:

```html
<!-- YouTube Import Modal -->
@if (showYouTubeModal()) {
  <app-youtube-import-modal
    (close)="showYouTubeModal.set(false)"
    (imported)="onYouTubeImported($event)"
  />
}
```

**Step 5: Add handler method**

Add method in the component class:

```typescript
onYouTubeImported(result: { hash: string; name: string; path: string; duration: number }): void {
  // Add the imported sound to the soundboard
  this.soundboard.addImportedSound(result);
  this.showYouTubeModal.set(false);
}
```

**Step 6: Add addImportedSound method to SoundboardService**

In `src/app/core/services/soundboard.service.ts`, add:

```typescript
/**
 * Add a sound that was imported externally (e.g., from YouTube)
 */
addImportedSound(imported: { hash: string; name: string; path: string; duration: number }): void {
  // Check for duplicate
  if (this._sounds().has(imported.hash)) {
    console.warn('Sound already exists:', imported.name);
    return;
  }

  const sound: Sound = {
    id: imported.hash,
    name: imported.name,
    path: imported.path,
    duration: imported.duration,
    volume: 1.0,
    speed: 1.0,
    folderIds: [this._activeFolderId()],
    isPlaying: false,
    addedAt: Date.now(),
  };

  const sounds = new Map(this._sounds());
  sounds.set(sound.id, sound);
  this._sounds.set(sounds);

  this.saveState();
}
```

**Step 7: Build and commit**

```bash
npm run build
git add src/app/features/soundboard/soundboard.component.ts src/app/core/services/soundboard.service.ts
git commit -m "feat(youtube): integrate YouTube import button in soundboard"
```

---

## Task 13: Test End-to-End Flow

**Step 1: Download binaries locally for testing**

```bash
mkdir -p src-tauri/binaries

# Download yt-dlp for your platform
# macOS ARM:
curl -L "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos" -o src-tauri/binaries/yt-dlp-aarch64-apple-darwin
chmod +x src-tauri/binaries/yt-dlp-aarch64-apple-darwin

# Download ffmpeg for your platform (or use system ffmpeg for testing)
```

**Step 2: Run the app**

```bash
npm run tauri dev
```

**Step 3: Test the flow**

1. Click "YouTube" button
2. Paste a YouTube URL (e.g., a short video)
3. Wait for download
4. Adjust trim points with waveform
5. Click "Import Sound"
6. Verify sound appears in soundboard

**Step 4: Add binaries to .gitignore**

```bash
echo "src-tauri/binaries/" >> .gitignore
git add .gitignore
git commit -m "chore: ignore local binaries directory"
```

---

## Task 14: Update Roadmap

**Files:**
- Modify: `ROADMAP.md`

**Step 1: Mark YouTube Audio Import as in progress**

Update Phase 3 section:

```markdown
- [ ] **YouTube Audio Import** _(in progress)_
  - Enter YouTube URL to extract audio from video
  - Audio trimming editor (cut start/end points)
  - Preview trimmed audio before import
  - Import trimmed audio as sound pad
```

**Step 2: Commit**

```bash
git add ROADMAP.md
git commit -m "docs: mark YouTube Audio Import in progress"
```

---

## Summary

| Task | Description |
|------|-------------|
| 1 | Create binary download script for CI |
| 2 | Update CI workflow to download binaries |
| 3 | Configure Tauri external binaries |
| 4 | Create YouTube DTOs |
| 5 | Implement youtube_download command |
| 6 | Implement youtube_trim_and_import command |
| 7 | Implement youtube_cancel command + cleanup |
| 8 | Install wavesurfer.js |
| 9 | Create YouTubeService |
| 10 | Create AudioTrimmerComponent |
| 11 | Create YouTubeImportModalComponent |
| 12 | Integrate YouTube button in Soundboard |
| 13 | Test end-to-end flow |
| 14 | Update roadmap |
