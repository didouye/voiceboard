# Landing Page Design

> **Date**: 2026-01-04
> **Status**: Approved

## Overview

Single-page marketing website for Voiceboard. Dark theme, minimal, with smart OS detection for downloads.

## Structure

```
┌─────────────────────────────────────────────────────────────┐
│  HERO                                                        │
│  - Logo + "Voiceboard"                                       │
│  - Tagline: "Mix your voice with sounds. Send it anywhere." │
│  - Download button (OS auto-detected)                        │
│  - "Also available for Windows, Linux" links                 │
├─────────────────────────────────────────────────────────────┤
│  SCREENSHOT                                                  │
│  - Mockup window (macOS style frame)                         │
│  - App screenshot inside                                     │
├─────────────────────────────────────────────────────────────┤
│  FEATURES (3 columns)                                        │
│  - Mix: Real-time mic + sounds                               │
│  - Soundboard: Trigger with keyboard shortcuts               │
│  - Cross-platform: Windows, macOS, Linux                     │
├─────────────────────────────────────────────────────────────┤
│  FOOTER                                                      │
│  - GitHub link                                               │
│  - Made by didouye                                           │
│  - Version number                                            │
└─────────────────────────────────────────────────────────────┘
```

## Technical Stack

| Component | Choice |
|-----------|--------|
| **HTML** | Single page, semantic |
| **CSS** | Tailwind CDN |
| **JS** | Vanilla JS (OS detection + GitHub API fetch) |
| **Fonts** | Inter (Google Fonts) |

## File Structure

```
/website
├── index.html          # Single page
├── assets/
│   ├── screenshot.png  # App screenshot
│   ├── logo.svg        # Voiceboard logo
│   └── favicon.ico     # Favicon
└── README.md           # Deployment instructions
```

## Color Palette (Dark Theme)

| Element | Color | Tailwind |
|---------|-------|----------|
| Background | #0f172a | `bg-slate-900` |
| Surface | #1e293b | `bg-slate-800` |
| Text primary | #f1f5f9 | `text-slate-100` |
| Text secondary | #94a3b8 | `text-slate-400` |
| Accent | #3b82f6 | `bg-blue-500` |
| Accent hover | #2563eb | `hover:bg-blue-600` |

## OS Detection

```javascript
function detectOS() {
  const ua = navigator.userAgent;
  if (ua.includes('Win')) return 'windows';
  if (ua.includes('Mac')) return 'macos';
  if (ua.includes('Linux')) return 'linux';
  return 'windows'; // fallback
}
```

## GitHub Releases API

```javascript
const REPO = 'didouye/voiceboard';
const API = `https://api.github.com/repos/${REPO}/releases/latest`;

async function getDownloadUrl(os) {
  const release = await fetch(API).then(r => r.json());
  const asset = release.assets.find(a => a.name.includes(os));
  return asset?.browser_download_url;
}
```

## Download Button

```html
<a href="#" class="
  inline-flex items-center gap-2
  px-8 py-4
  bg-blue-500 hover:bg-blue-600
  text-white font-semibold text-lg
  rounded-xl
  shadow-lg shadow-blue-500/25
  transition-all
">
  <svg><!-- OS icon --></svg>
  Download for macOS
</a>
<p class="mt-3 text-slate-400 text-sm">
  Also available for
  <a href="#" class="underline">Windows</a> and
  <a href="#" class="underline">Linux</a>
</p>
```

## Mockup Window

macOS-style frame with traffic light buttons:

```
┌──────────────────────────────────────┐
│ ● ● ●                    Voiceboard  │
├──────────────────────────────────────┤
│                                      │
│         [Screenshot here]            │
│                                      │
└──────────────────────────────────────┘
```

## Responsive Design

| Size | Behavior |
|------|----------|
| Mobile (<640px) | Features stacked, screenshot smaller |
| Tablet (640-1024px) | Features 2 columns |
| Desktop (>1024px) | Full layout, 3 column features |

## Accessibility

- AAA contrast (light text on dark background)
- `alt` on all images
- Visible focus on links/buttons
- `prefers-reduced-motion` respected

## Hosting

To be determined - separate task in Phase 2. Options:
- Vercel (free tier)
- Netlify (free tier)
- Cloudflare Pages (free tier)
- Custom domain setup
