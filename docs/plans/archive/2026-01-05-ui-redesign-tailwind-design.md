# UI Redesign - Tailwind Migration

> **Date:** 2026-01-05
> **Status:** Design approved
> **Phase:** 3 - UI/UX Redesign

## Overview

Complete UI rewrite from inline CSS to Tailwind CSS with a new Gaming/Pro audio visual style inspired by Elgato Stream Deck and GoXLR.

## Design Decisions

| Aspect | Choice |
|--------|--------|
| Approach | Complete rewrite (remove all inline styles) |
| Style | Gaming/Pro audio (Elgato/GoXLR) |
| Colors | Violet/Magenta neon (#9d4edd, #ff00ff) |
| Background | Dark with subtle grain texture |
| Animations | Expressive (bounce, glow, slide-in) |
| Layout | Sidebar (folders) + Main (pads) + Status bar |

## Layout Structure

```
┌─────────────────────────────────────────────────────────────┐
│  ▣ Voiceboard                              ─  □  ✕         │
├────────────────┬────────────────────────────────────────────┤
│                │                                            │
│  📁 FOLDERS    │     ┌───┐ ┌───┐ ┌───┐ ┌───┐              │
│  ┌──────────┐  │     │ 1 │ │ 2 │ │ 3 │ │ 4 │              │
│  │ Default  │◀─│     └───┘ └───┘ └───┘ └───┘              │
│  └──────────┘  │     ┌───┐ ┌───┐ ┌───┐ ┌───┐              │
│  ┌──────────┐  │     │ 5 │ │ 6 │ │ 7 │ │ 8 │              │
│  │ Memes    │  │     └───┘ └───┘ └───┘ └───┘              │
│  └──────────┘  │     ┌───┐ ┌───┐ ┌───┐ ┌───┐              │
│  ┌──────────┐  │     │ 9 │ │10 │ │11 │ │12 │              │
│  │ Music    │  │     └───┘ └───┘ └───┘ └───┘              │
│  └──────────┘  │                                            │
│                │                          [STOP ALL] 🛑    │
│  ──────────────│────────────────────────────────────────────│
│  ⚙ Settings    │                                            │
├────────────────┴────────────────────────────────────────────┤
│  🎤 Blue Yeti      🔊 VB-Cable       🎧 Speakers           │
│  ████████░░░░░░    ██████░░░░░░░░    ████░░░░░░░░░░       │
└─────────────────────────────────────────────────────────────┘
```

### Sidebar (~180px)

- **Folder list**: Display all folders, active folder highlighted
- **Default folder**: Created automatically, cannot be deleted
- **New folder button**: Prepared but disabled (Phase 3 future)
- **Settings button**: Bottom of sidebar, opens settings popup

### Main Area

- **Pad grid**: 4 columns, responsive rows
- **Stop All button**: Bottom right corner

### Status Bar (~50px)

- **3 columns**: Input | Output | Preview
- **Each column**: Icon + device name + VU meter + status indicator
- **VU meter**: Simple horizontal bar with gradient fill (green → yellow → red)

## Color Palette

```css
/* Backgrounds */
--bg-background: #0a0a0f;    /* Main background (deep black with blue tint) */
--bg-surface: #12121a;       /* Cards, sidebar */
--bg-surface-hover: #1a1a25; /* Hover state */

/* Borders */
--border-default: #2a2a3a;   /* Subtle borders */

/* Accents */
--accent-primary: #9d4edd;   /* Violet */
--accent-glow: #bf5af2;      /* Violet light (for glow effects) */
--accent-hot: #ff00ff;       /* Magenta (active states) */

/* Text */
--text-primary: #ffffff;
--text-secondary: #888899;
--text-muted: #555566;

/* Status colors */
--status-success: #22c55e;   /* Green */
--status-warning: #eab308;   /* Yellow */
--status-error: #ef4444;     /* Red */
--status-info: #00d4ff;      /* Cyan */
```

## Tailwind Configuration

```javascript
// tailwind.config.js
module.exports = {
  content: ["./src/**/*.{html,ts}"],
  theme: {
    extend: {
      colors: {
        background: '#0a0a0f',
        surface: {
          DEFAULT: '#12121a',
          hover: '#1a1a25',
        },
        border: {
          DEFAULT: '#2a2a3a',
        },
        accent: {
          DEFAULT: '#9d4edd',
          glow: '#bf5af2',
          hot: '#ff00ff',
        },
        text: {
          primary: '#ffffff',
          secondary: '#888899',
          muted: '#555566',
        },
      },
      borderRadius: {
        DEFAULT: '8px',
      },
      animation: {
        'glow-pulse': 'glow-pulse 1s ease-in-out infinite',
        'bounce-click': 'bounce-click 200ms ease-out',
      },
      keyframes: {
        'glow-pulse': {
          '0%, 100%': { boxShadow: '0 0 20px rgba(157, 78, 221, 0.5)' },
          '50%': { boxShadow: '0 0 40px rgba(157, 78, 221, 0.8)' },
        },
        'bounce-click': {
          '0%': { transform: 'scale(0.95)' },
          '100%': { transform: 'scale(1)' },
        },
      },
    },
  },
  plugins: [],
}
```

## Grain Texture

Add subtle noise overlay for premium feel:

```css
.grain-overlay {
  position: fixed;
  inset: 0;
  pointer-events: none;
  background-image: url('/assets/noise.svg');
  opacity: 0.03;
  mix-blend-mode: overlay;
}
```

## Component Designs

### Pad States

| State | Style |
|-------|-------|
| **Empty** | Dashed border, `+` icon, "Drop or click" text |
| **With sound** | Gradient background (pad color), solid border |
| **Hover** | Scale 1.02, subtle glow, settings button visible |
| **Playing** | Pulsing glow, magenta border (#ff00ff), animated visualizer |
| **Previewing** | Cyan border (#00d4ff), cyan glow |

### Pad Structure

```
┌─────────────────┐
│ [1]         ⚙  │  <- Hotkey badge + Settings button (hover only)
│                 │
│   🎵 Airhorn    │  <- Sound name (centered)
│     0:02        │  <- Duration
│                 │
│  ▁▂▃▅▃▂▁       │  <- Mini visualizer (when playing)
└─────────────────┘
```

### Glow Effects

```css
/* Pad playing */
.pad-playing {
  box-shadow:
    0 0 20px rgba(157, 78, 221, 0.5),
    0 0 40px rgba(157, 78, 221, 0.3),
    inset 0 0 20px rgba(157, 78, 221, 0.1);
}

/* Subtle hover */
.pad-hover {
  box-shadow: 0 0 15px rgba(157, 78, 221, 0.3);
}
```

### Folder List

| State | Style |
|-------|-------|
| **Inactive** | Transparent background, secondary text |
| **Hover** | Surface-hover background, primary text |
| **Active** | Surface background, 2px left accent border, primary text |

### Settings Popup

```
┌─────────────────────────────────────────────────┐
│  ⚙ Settings                              ✕     │
├─────────────────────────────────────────────────┤
│                                                 │
│  AUDIO DEVICES                                  │
│  ─────────────────────────────────────────────  │
│                                                 │
│  🎤 Input                                       │
│  ┌─────────────────────────────────────────┐   │
│  │ Blue Yeti                            ▼  │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  🔊 Output (Virtual Mic)                        │
│  ┌─────────────────────────────────────────┐   │
│  │ VB-Cable Input                       ▼  │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  🎧 Preview                                     │
│  ┌─────────────────────────────────────────┐   │
│  │ Speakers (Realtek)                   ▼  │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  MIXER                                          │
│  ─────────────────────────────────────────────  │
│                                                 │
│  Mic Volume                          🔇        │
│  ═══════════════════════○─────  80%            │
│                                                 │
│  Master Volume                                  │
│  ══════════════════════════○──  100%           │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │         ▶  START MIXING                 │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
└─────────────────────────────────────────────────┘
```

- Modal centered with backdrop blur
- Scale + fade animation on open
- Custom styled dropdowns (no native select)
- Gradient slider tracks
- Large accent-colored Start/Stop button

### VU Meters (Status Bar)

Simple horizontal fill bar:

```css
.vu-meter {
  height: 6px;
  background: var(--bg-surface);
  border-radius: 3px;
  overflow: hidden;
}

.vu-meter-fill {
  height: 100%;
  background: linear-gradient(to right, #22c55e, #eab308, #ef4444);
  transition: width 50ms ease-out;
}
```

## Animations

| Element | Animation |
|---------|-----------|
| Pad hover | `transform: scale(1.02)`, 150ms ease-out |
| Pad click | Scale 0.95 → 1.0 (bounce), 200ms |
| Pad playing | Glow pulse infinite 1s |
| Folder select | Left border slide-in |
| Settings popup | Scale 0.95 → 1.0 + fade in |
| VU meter | Width transition 50ms |

## Files to Create/Modify

### New Files
- `tailwind.config.js` - Tailwind configuration
- `src/assets/noise.svg` - Grain texture
- `src/app/shared/components/settings-popup/` - Settings popup component
- `src/app/shared/components/vu-meter/` - VU meter component

### Files to Rewrite
- `src/styles.css` - Global styles + Tailwind imports
- `src/app/app.component.ts` - Add grain overlay
- `src/app/features/mixer/mixer.component.ts` - New sidebar + main layout
- `src/app/features/soundboard/soundboard.component.ts` - Grid layout
- `src/app/features/soundboard/sound-pad/sound-pad.component.ts` - Pad redesign
- `src/app/features/devices/device-selector.component.ts` - Move to settings popup
- `src/app/features/mixer/master-control/master-control.component.ts` - Move to settings popup
- `src/app/features/mixer/channel-strip/channel-strip.component.ts` - Move to settings popup
- `src/app/features/mixer/level-meters/level-meters.component.ts` - New VU meter style

## Data Model Changes

### Folder Support

```typescript
interface Folder {
  id: string;
  name: string;
  createdAt: Date;
}

interface SoundboardState {
  folders: Folder[];
  activeFolderId: string;
  padsByFolder: Record<string, SoundPad[]>;
}
```

Default folder created on first launch with id `default`.

## Implementation Notes

1. **Install Tailwind first**: `npm install -D tailwindcss postcss autoprefixer`
2. **Remove all inline styles**: Convert component by component
3. **Test each component**: Ensure visual parity before moving to next
4. **Folder system**: Prepare data model but only implement "Default" folder initially
5. **Settings popup**: Extract device selectors and mixer controls from current layout

## Out of Scope (Phase 3 Future)

- Add/rename/delete folders
- Drag & drop to reorganize pads
- Custom keyboard shortcuts
- Pad images
- Light theme / custom themes
