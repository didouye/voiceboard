# YouTube Audio Import - Design

> **Date:** 2026-02-02
> **Status:** Ready for implementation
> **Phase:** 3 (UI/UX)

## Overview

Allow users to import audio from YouTube videos by entering a URL, then trim the audio using a visual waveform editor before adding it to the soundboard.

## User Workflow

1. **Click "YouTube" button** in sidebar (next to "Import multiple")
2. **Paste YouTube URL** in modal dialog
3. **Click "Download"** → Progress bar while yt-dlp extracts audio as MP3
4. **Trim audio** using waveform editor:
   - Visual waveform display
   - Draggable handles for start/end points
   - Numeric inputs for precise adjustment
   - Play button to preview selection
5. **Click "Import"** → Audio is trimmed, normalized, and added to soundboard
6. **Temporary file cleaned up**

## Technical Architecture

### Bundled Binaries

Both binaries are bundled with the application (no download at runtime):

| Binary | Size | Purpose |
|--------|------|---------|
| yt-dlp | ~10 MB | YouTube audio extraction |
| ffmpeg | ~30-80 MB | MP3 conversion + trimming |

**Bundle structure:**
```
src-tauri/binaries/
├── yt-dlp-x86_64-pc-windows-msvc.exe
├── yt-dlp-x86_64-apple-darwin
├── yt-dlp-aarch64-apple-darwin
├── yt-dlp-x86_64-unknown-linux-gnu
├── ffmpeg-x86_64-pc-windows-msvc.exe
├── ffmpeg-x86_64-apple-darwin
├── ffmpeg-aarch64-apple-darwin
└── ffmpeg-x86_64-unknown-linux-gnu
```

**tauri.conf.json:**
```json
{
  "bundle": {
    "externalBin": [
      "binaries/yt-dlp",
      "binaries/ffmpeg"
    ]
  }
}
```

### Backend Commands (Rust)

```rust
/// Download audio from YouTube URL
/// Returns metadata and path to temporary MP3 file
#[tauri::command]
async fn youtube_download(url: String) -> Result<YouTubeAudioDto, String>

#[derive(Serialize)]
struct YouTubeAudioDto {
    temp_path: String,      // Path to downloaded MP3
    title: String,          // Video title
    duration: f64,          // Duration in seconds
    thumbnail_url: String,  // Video thumbnail (optional display)
}

/// Trim audio and import to soundboard
/// Uses ffmpeg for trimming, then existing normalization pipeline
#[tauri::command]
async fn youtube_trim_and_import(
    temp_path: String,
    start_seconds: f64,
    end_seconds: f64,
    name: String,
) -> Result<ImportedSoundDto, String>

/// Cancel import and cleanup temporary file
#[tauri::command]
async fn youtube_cancel(temp_path: String) -> Result<(), String>
```

**yt-dlp command:**
```bash
yt-dlp -x --audio-format mp3 --audio-quality 0 -o "{temp_dir}/{video_id}.mp3" "{url}"
```

**ffmpeg trim command:**
```bash
ffmpeg -i "{input}" -ss {start} -to {end} -c copy "{output}"
```

### Temporary Storage

- **Location:** `AppData/temp/youtube/`
- **Naming:** `{video_id}.mp3`
- **Cleanup:**
  - Deleted after successful import
  - Deleted when modal closed without import
  - Auto-cleanup on app startup (files > 24h)

### Frontend Components

**New dependency:**
- `wavesurfer.js` (~50 KB) - Waveform display and audio playback

**Component structure:**
```
src/app/features/soundboard/
├── youtube-import/
│   ├── youtube-import-modal.component.ts
│   ├── youtube-url-form.component.ts
│   ├── youtube-download-progress.component.ts
│   └── audio-trimmer.component.ts
```

**YouTubeImportModalComponent:**
- State machine: `idle` → `downloading` → `editing` → `importing`
- Manages transitions between steps
- Handles cleanup on close

**AudioTrimmerComponent:**
- Initializes wavesurfer.js with downloaded MP3
- Region plugin for start/end selection
- Synchronized numeric inputs
- Play selection button
- Displays selected duration in real-time

**YouTubeService:**
```typescript
@Injectable({ providedIn: 'root' })
export class YouTubeService {
  download(url: string): Observable<YouTubeAudioDto>;
  trimAndImport(tempPath: string, start: number, end: number, name: string): Promise<Sound>;
  cancel(tempPath: string): Promise<void>;
}
```

### UI Integration

- New "YouTube" button in sidebar, next to "Import multiple"
- YouTube icon (or link/video icon)
- Opens modal dialog

## Error Handling

### URL Errors
| Error | Message |
|-------|---------|
| Invalid URL | "URL YouTube invalide" |
| Private video | "Vidéo non accessible" |
| Geo-blocked | "Vidéo non disponible dans votre région" |
| Live stream | "Les lives ne sont pas supportés" |

### Download Errors
| Error | Message |
|-------|---------|
| No connection | "Vérifiez votre connexion internet" |
| Timeout (> 5 min) | "Téléchargement trop long, réessayez" |
| yt-dlp failure | Generic error + link to debug console |

## Limits

| Limit | Value | Behavior |
|-------|-------|----------|
| Max video duration | 30 min | Warning (not blocking) |
| Min selection | 0.5 sec | Enforced |
| Max selection | 5 min | Enforced |

## Accessibility

- Trim handles navigable with keyboard (←/→ arrows)
- ARIA labels on all controls
- Focus management in modal

## CI/CD Changes

**New script:** `.github/scripts/download-binaries.sh`
- Downloads yt-dlp from GitHub releases
- Downloads ffmpeg static builds
- Runs before `tauri build` in release workflow

**Bundle size impact:**
- +40-90 MB per platform (yt-dlp + ffmpeg)
