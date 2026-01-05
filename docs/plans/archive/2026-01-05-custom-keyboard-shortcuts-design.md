# Custom Keyboard Shortcuts - Design

> **Date:** 2026-01-05
> **Status:** Approved

## Overview

Allow users to configure custom keyboard shortcuts for each pad, with modifier key support (Ctrl, Alt, Shift, Meta) and global hotkeys (work even when app is not focused).

## Key Decisions

| Decision | Choice |
|----------|--------|
| Global hotkeys | Enabled by default, toggle in Settings to disable |
| Key recording | Capture first key combination pressed (immediate) |
| Conflict detection | Block and warn, user must confirm replacement |
| Recorder UI | Inline button in pad settings popup |
| Global toggle location | Main Settings popup |

## Shortcut Format

```
[Modifier+]*Key

Examples:
- "1"
- "Ctrl+1"
- "Ctrl+Shift+A"
- "Alt+F1"
- "Meta+Space"
```

**Supported modifiers:** Ctrl, Alt, Shift, Meta (Cmd on macOS, Win on Windows)

## Data Model

### Shortcut Type

```typescript
interface KeyboardShortcut {
  key: string;           // "1", "A", "F1", "Space", etc.
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;         // Cmd on macOS, Win on Windows
}

// Utility functions
function parseShortcut(str: string): KeyboardShortcut;
function formatShortcut(shortcut: KeyboardShortcut): string;
function shortcutMatches(event: KeyboardEvent, shortcut: KeyboardShortcut): boolean;
```

### Settings Change

```typescript
interface AudioSettings {
  // ... existing fields ...
  globalHotkeysEnabled: boolean;  // default: true
}
```

### Persistence

- `soundboard.json`: existing `hotkey` field per pad (now supports "Ctrl+1" format)
- `settings.json`: new `globalHotkeysEnabled` field

## Backend (Tauri)

### Plugin

Use `tauri-plugin-global-shortcut` (official Tauri v2 plugin).

### New Commands

```rust
/// Register a global shortcut for a pad
#[tauri::command]
fn register_global_shortcut(
    pad_id: String,
    shortcut: String,  // "Ctrl+Shift+1"
) -> Result<(), String>;

/// Unregister a shortcut
#[tauri::command]
fn unregister_global_shortcut(shortcut: String) -> Result<(), String>;

/// Unregister all shortcuts
#[tauri::command]
fn unregister_all_shortcuts() -> Result<(), String>;

/// Enable/disable global hotkeys
#[tauri::command]
fn set_global_hotkeys_enabled(enabled: bool) -> Result<(), String>;
```

### Event Emitted to Frontend

```rust
// When a global shortcut is pressed
app.emit("global-shortcut-triggered", json!({
    "padId": "pad-0",
    "shortcut": "Ctrl+1"
}));
```

### Flow

1. On startup, frontend sends all shortcuts to register
2. When user changes a shortcut: unregister old + register new
3. When global hotkeys disabled: `unregister_all_shortcuts()`
4. Backend emits `global-shortcut-triggered` → frontend plays sound

## Frontend

### New Service: ShortcutService

```typescript
@Injectable({ providedIn: 'root' })
export class ShortcutService {
  // Map shortcut string → pad id
  private registry = new Map<string, string>();

  // Check if a shortcut is already used
  checkConflict(shortcut: string, excludePadId?: string): string | null;

  // Register a shortcut (returns old pad id if conflict)
  register(padId: string, shortcut: string): string | null;

  // Unregister a shortcut
  unregister(shortcut: string): void;

  // Sync with backend (global hotkeys)
  syncWithBackend(): Promise<void>;
}
```

### Key Recorder UI (SoundPadComponent)

Add to settings popup after Speed section:

```html
<!-- Shortcut -->
<div class="pt-3 border-t border-border">
  <div class="flex justify-between items-center mb-2 text-xs">
    <span class="text-text-secondary">Shortcut</span>
  </div>
  <div class="flex gap-1">
    <button
      class="flex-1 px-3 py-1.5 text-xs rounded border transition-colors"
      [class]="isRecording
        ? 'bg-accent border-accent text-white animate-pulse'
        : 'bg-surface-hover border-border text-text-primary'"
      (click)="startRecording($event)"
      (keydown)="onRecordKeydown($event)"
    >
      {{ isRecording ? 'Press keys...' : (pad.hotkey || 'Click to set') }}
    </button>
    @if (pad.hotkey) {
      <button
        class="px-2 text-text-muted hover:text-status-error"
        (click)="clearShortcut($event)"
      >x</button>
    }
  </div>
</div>
```

### Conflict Detection Flow

1. User presses "Ctrl+1" in recorder
2. `SoundPadComponent` calls `shortcutService.checkConflict("Ctrl+1", pad.id)`
3. If conflict detected:
   - Show dialog: "Ctrl+1 is assigned to 'Airhorn'. Replace?"
   - [Replace] → old pad loses shortcut, new pad gets it
   - [Cancel] → cancel, keep old shortcut
4. If no conflict:
   - Register directly
5. Call backend `register_global_shortcut()`
6. Save to `soundboard.json`

### Settings Popup Toggle

Add to Settings popup (after device selectors):

```html
<!-- Keyboard section -->
<div class="pt-4 border-t border-border">
  <h4 class="text-xs font-semibold text-text-secondary uppercase mb-3">
    Keyboard
  </h4>

  <div class="flex items-center justify-between">
    <div>
      <span class="text-sm text-text-primary">Global Hotkeys</span>
      <p class="text-xs text-text-muted">
        Trigger sounds even when app is in background
      </p>
    </div>
    <button
      class="w-10 h-5 rounded-full transition-colors relative"
      [class]="globalHotkeysEnabled ? 'bg-accent' : 'bg-surface-hover'"
      (click)="toggleGlobalHotkeys()"
    >
      <span
        class="absolute top-0.5 w-4 h-4 bg-white rounded-full transition-transform"
        [class]="globalHotkeysEnabled ? 'left-5' : 'left-0.5'"
      ></span>
    </button>
  </div>
</div>
```

**Behavior:**
- Toggle ON → `register_global_shortcut()` for all pads with shortcuts
- Toggle OFF → `unregister_all_shortcuts()`, local shortcuts still work when app focused

## Files to Create

- `src/app/core/services/shortcut.service.ts` - Shortcut management and conflicts
- `src/app/core/models/shortcut.model.ts` - Types and utilities

## Files to Modify

| File | Changes |
|------|---------|
| `src/app/core/models/audio-device.model.ts` | Add `globalHotkeysEnabled` to `AudioSettings` |
| `src/app/features/soundboard/sound-pad/sound-pad.component.ts` | Add key recorder UI |
| `src/app/features/soundboard/soundboard.component.ts` | Integrate `ShortcutService`, listen to backend events |
| `src/app/features/settings/settings-popup.component.ts` | Add Global Hotkeys toggle |
| `src-tauri/Cargo.toml` | Add `tauri-plugin-global-shortcut` |
| `src-tauri/src/lib.rs` | Initialize plugin, add commands |
| `src-tauri/src/application/commands.rs` | New shortcut commands |

## Dependencies

- `tauri-plugin-global-shortcut = "2"` (Cargo.toml)

## Testing

- Unit tests for `ShortcutService` (parse, format, conflict detection)
- Unit tests for shortcut utility functions
- Unit tests for Rust commands
