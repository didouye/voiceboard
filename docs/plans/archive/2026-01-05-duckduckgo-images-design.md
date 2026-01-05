# DuckDuckGo Images - Design Document

> **Date:** 2026-01-05
> **Status:** Ready for implementation

## Overview

Replace Pexels API with DuckDuckGo Images for pad image search. DuckDuckGo provides more relevant results for sound/audio-related imagery without requiring an API key.

## Why DuckDuckGo?

- **No API key required** - Works out of the box
- **No rate limiting** - Reasonable usage is fine
- **Better results** - More relevant for sound effect imagery
- **Free** - No cost

## Architecture

```
[Angular Frontend]
    → @tauri-apps/plugin-http (bypass CORS)
    → DuckDuckGo internal API
```

### Search Flow

1. **Get vqd token:**
   ```
   GET https://duckduckgo.com/?q={query}&iax=images&ia=images
   → Parse HTML to extract vqd="xxx" token
   ```

2. **Fetch images:**
   ```
   GET https://duckduckgo.com/i.js?l=fr-fr&o=json&q={query}&vqd={token}
   → JSON response with "results" array
   ```

### Response Structure

```typescript
interface DDGImageResult {
  image: string;      // Full image URL
  thumbnail: string;  // Thumbnail URL
  title: string;      // Image title
  source: string;     // Source website
}
```

## Changes Summary

### Files to Modify

| File | Action |
|------|--------|
| `image-search.service.ts` | Replace Pexels with DDG implementation |
| `settings-popup.component.ts` | Remove "Image Search" section |
| `src-tauri/Cargo.toml` | Add `tauri-plugin-http` |
| `src-tauri/src/lib.rs` | Register HTTP plugin |
| `src-tauri/tauri.conf.json` | Add HTTP permissions |
| `image-search.service.spec.ts` | Update tests |

### Files Unchanged

- `ImageSearchResult` interface (same structure)
- `ImageSuggestionToastComponent`
- `BulkImageWizardComponent`
- `sound-pad.component.ts` (image section)
- Image storage logic

## Error Handling

Simple approach:
- Network error → "Search unavailable, try again later"
- No results → "No images found"
- No retry logic (keep it simple)

## Removed Features

- Pexels API key configuration in Settings
- API key validation/testing
- Attribution field (DDG doesn't require it)
