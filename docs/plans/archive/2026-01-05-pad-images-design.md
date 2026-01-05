# Pad Images - Design Document

> **Date:** 2026-01-05
> **Status:** Ready for implementation

## Overview

Allow users to add custom images to soundboard pads for quick visual identification.

## Scope

- Upload image from local file
- Paste URL
- Search images via Pexels API
- Auto-suggestion on import

## Decisions Summary

| Aspect | Choice |
|--------|--------|
| Scope | Upload file + URL + Pexels search |
| Default provider | Pexels (free, 200 req/h) |
| Optional provider | Custom API key (Google/Bing) in Settings |
| UI access | Section in existing Pad Settings modal |
| Search display | Expandable section |
| Storage | Local copy (`~/.voiceboard/images/`) |
| Pad display | Full background + text with shadow |
| Auto-search | Suggestion on import (accept/other choices/ignore) |

## Data Model

### SoundPad modifications

```typescript
// src/app/features/soundboard/models/sound-pad.model.ts
interface SoundPad {
  // ... existing fields
  image?: PadImage;
}

interface PadImage {
  localPath: string;      // Relative path in ~/.voiceboard/images/
  originalUrl?: string;   // Source URL (for Pexels attribution)
  attribution?: string;   // "Photo by X on Pexels" (if required)
}
```

### Image storage

```
~/.voiceboard/
├── soundboard.json      # Existing
├── settings.json        # Existing
└── images/
    ├── pad-0-abc123.jpg
    ├── pad-1-def456.png
    └── ...
```

**File naming:** `{padId}-{hash8}.{ext}`
- Hash of first 8 characters of content to avoid duplicates
- Original extension preserved

## Backend (Rust) - New Commands

```rust
#[tauri::command]
async fn save_pad_image(pad_id: String, image_data: Vec<u8>, extension: String) -> Result<String, String>

#[tauri::command]
async fn delete_pad_image(pad_id: String) -> Result<(), String>

#[tauri::command]
async fn get_images_dir() -> Result<String, String>
```

## User Interface

### Pad Settings Modal - Image Section

```
┌─────────────────────────────────────────────┐
│  Pad Settings - "Airhorn"            [X]    │
├─────────────────────────────────────────────┤
│  Name: [Airhorn________________]            │
│                                             │
│  ┌─────────┐                                │
│  │  Image  │  [Upload] [URL] [Rechercher]   │
│  │ Preview │  [Supprimer]                   │
│  └─────────┘                                │
│                                             │
│  ▼ Recherche d'images (expandable)          │
│  ┌─────────────────────────────────────────┐│
│  │ [🔍 airhorn_______________] [Chercher]  ││
│  │                                         ││
│  │  ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐   ││
│  │  │   │ │   │ │   │ │   │ │   │ │   │   ││
│  │  └───┘ └───┘ └───┘ └───┘ └───┘ └───┘   ││
│  │         (grid of 6 results)             ││
│  │            [Load more]                  ││
│  └─────────────────────────────────────────┘│
│                                             │
│  Volume: [━━━━━━━●━━━] 100%                 │
│  Speed:  [0.5x] [1x] [1.5x] [2x]            │
│  Shortcut: [Ctrl+1] [Record]                │
└─────────────────────────────────────────────┘
```

### Pad Display with Image

```
┌────────────────┐
│░░░░░░░░░░░░░░░░│  ← Image as background (object-fit: cover)
│░░░░░░░░░░░░░░░░│
│░░░░░░░░░░░░░░░░│
│▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  ← Transparent to black gradient
│  Airhorn       │  ← Text with text-shadow
└────────────────┘
```

**Tailwind classes for text:** `drop-shadow-[0_2px_4px_rgba(0,0,0,0.8)]`

## API Integration

### Two-phase architecture

**Current phase (without Django backend):**
- User provides their own Pexels API key in Settings
- Direct requests from frontend to Pexels API
- Required field to use image search

**Future phase (with Django backend):**
- Voiceboard API key stored server-side (Django admin)
- Proxy endpoint: `POST /api/images/search?q=...`
- Backend makes the Pexels request and returns results
- No key exposed on client side

### Settings - Image Search Section

```
┌─ Image Search ──────────────────────────────┐
│ Pexels API Key: [________________________]  │
│                                             │
│ ℹ️ Create a free key at pexels.com          │
│    (200 requests/hour)                      │
│                                             │
│ [Test connection]                           │
└─────────────────────────────────────────────┘
```

### Frontend Service

```typescript
// src/app/core/services/image-search.service.ts

interface ImageSearchResult {
  id: string;
  thumbnailUrl: string;   // For grid (small)
  fullUrl: string;        // For download (medium)
  attribution: string;    // "Photo by X on Pexels"
  photographer: string;
}

interface ImageSearchProvider {
  search(query: string, page: number): Promise<ImageSearchResult[]>;
  name: string;
}
```

## Auto-suggestion on Import

### Single import - Toast with 3 options

```
┌──────────────────────────────────────────────────┐
│ 🖼️ Suggested image for "airhorn"                 │
│ ┌──────┐                                         │
│ │ img  │  [Accept] [Other choices] [Ignore]      │
│ └──────┘                                         │
└──────────────────────────────────────────────────┘

         ↓ Click "Other choices"

┌──────────────────────────────────────────────────┐
│ 🖼️ Image for "airhorn"                           │
│ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐    │
│ │  ✓   │ │      │ │      │ │      │ │      │    │
│ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘    │
│ [🔍 airhorn_______________] [Search]             │
│                                                  │
│              [Select] [Ignore]                   │
└──────────────────────────────────────────────────┘
```

### Bulk import - Sequential flow

```
1. Import 20 sounds
2. Initial notification:
   ┌────────────────────────────────────────────┐
   │ 🖼️ Assign images to 20 sounds?             │
   │                                            │
   │         [Yes] [No thanks]                  │
   └────────────────────────────────────────────┘

3. If "Yes" → Sequential modal (1/20, 2/20, etc.):
   ┌────────────────────────────────────────────┐
   │ Image for "airhorn" (1/20)          [Skip] │
   │ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐       │
   │ │  ✓   │ │      │ │      │ │      │       │
   │ └──────┘ └──────┘ └──────┘ └──────┘       │
   │ [🔍 airhorn_______________] [Search]       │
   │                                            │
   │    [← Previous] [Select] [Next →]          │
   │              [Finish]                      │
   └────────────────────────────────────────────┘
```

- **Previous/Next:** Navigate without selecting
- **Skip:** Skip this sound (no image)
- **Finish:** Stop wizard, keep already selected images

### Name extraction rules

- Remove extension, replace `_-` with spaces
- Example: `funny_airhorn_sound.mp3` → `"funny airhorn sound"`
- Query: First 3 words max (avoid overly specific queries)
- Selection: First image from results

### Settings option

```
☑️ Suggest images automatically on import
```

(Enabled by default, can be disabled)

## Error Handling

### API Errors

| Situation | Behavior |
|-----------|----------|
| No API key configured | "Search" button disabled, tooltip "Configure Pexels key in Settings" |
| Invalid API key | Error toast "Invalid API key", link to Settings |
| Rate limit reached | Toast "Rate limit reached, retry in 1h" |
| No results | Message "No images found" + suggestion to modify search |
| Network error | Toast "Connection error", "Retry" button |

### Image Edge Cases

| Situation | Behavior |
|-----------|----------|
| Image deleted from disk | Show placeholder + "Missing image" indicator |
| Unsupported format | Accept: JPG, PNG, WebP, GIF (static). Reject others with message |
| Image too large (>10MB) | Auto-resize to 512x512 max |
| Invalid URL | Toast "Invalid URL or image inaccessible" |

### Cleanup

- **Pad deletion:** Also delete associated image from folder
- **Image change:** Delete old image before saving new one
- **Garbage collection:** On startup, delete orphaned images (not referenced)

## Roadmap Updates Required

Add to **Phase 4 - Cloud & Collaboration**:
```
- [ ] **Image Search Proxy**
  - Proxy endpoint for Pexels API
  - API key configurable in Django admin
  - Automatic migration (removes local key when connected to cloud)
```
