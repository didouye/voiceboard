import { Injectable, signal, OnDestroy, effect } from '@angular/core';
import { TauriService } from './tauri.service';
import { SoundboardService } from './soundboard.service';
import { formatShortcut, shortcutFromEvent, isModifierKey } from '../models';

@Injectable({
  providedIn: 'root'
})
export class ShortcutService implements OnDestroy {
  // Map shortcut string -> sound id
  private registry = new Map<string, string>();

  // Enabled state
  private _enabled = signal(true);
  readonly enabled = this._enabled.asReadonly();

  private unlistenGlobalShortcut?: () => void;
  private initialized = false;
  private lastHotkeySnapshot = '';

  constructor(
    private tauri: TauriService,
    private soundboard: SoundboardService
  ) {
    this.init();

    // Watch for sound changes and resync shortcuts when hotkey associations change
    effect(() => {
      const sounds = Array.from(this.soundboard.sounds().values());
      // Create a snapshot of hotkey -> soundId associations
      const snapshot = sounds
        .filter(s => s.hotkey)
        .map(s => `${s.hotkey}:${s.id}`)
        .sort()
        .join('|');

      // Only sync if initialized and associations actually changed
      if (this.initialized && snapshot !== this.lastHotkeySnapshot) {
        this.lastHotkeySnapshot = snapshot;
        this.syncFromSoundboard();
      }
    });
  }

  private async init(): Promise<void> {
    // Load initial state
    try {
      const settings = await this.tauri.loadSettings();
      this._enabled.set(settings.audio.globalHotkeysEnabled);
    } catch (err) {
      console.error('Failed to load global hotkeys setting:', err);
    }

    // Listen for global shortcut events from backend
    this.unlistenGlobalShortcut = await this.tauri.listenGlobalShortcut((data) => {
      console.log('[ShortcutService] Global shortcut triggered:', data);
      this.soundboard.playSound(data.soundId);
    });

    // Register all existing shortcuts from soundboard
    await this.syncFromSoundboard();

    // Capture initial snapshot and mark as initialized
    this.updateHotkeySnapshot();
    this.initialized = true;
  }

  private updateHotkeySnapshot(): void {
    const sounds = Array.from(this.soundboard.sounds().values());
    this.lastHotkeySnapshot = sounds
      .filter(s => s.hotkey)
      .map(s => `${s.hotkey}:${s.id}`)
      .sort()
      .join('|');
  }

  ngOnDestroy(): void {
    this.unlistenGlobalShortcut?.();
  }

  /**
   * Sync shortcuts from soundboard sounds to backend
   */
  async syncFromSoundboard(): Promise<void> {
    const sounds = Array.from(this.soundboard.sounds().values());

    // Clear existing
    await this.tauri.unregisterAllShortcuts();
    this.registry.clear();

    // Register all sound shortcuts
    for (const sound of sounds) {
      if (sound.hotkey) {
        try {
          await this.tauri.registerGlobalShortcut(sound.id, sound.hotkey);
          this.registry.set(sound.hotkey, sound.id);
        } catch (err) {
          console.error(`Failed to register shortcut ${sound.hotkey} for ${sound.id}:`, err);
        }
      }
    }
  }

  /**
   * Check if a shortcut is already used by another sound
   * Returns the sound ID if conflict exists, null otherwise
   */
  checkConflict(shortcut: string, excludeSoundId?: string): string | null {
    const existingSoundId = this.registry.get(shortcut);
    if (existingSoundId && existingSoundId !== excludeSoundId) {
      return existingSoundId;
    }
    return null;
  }

  /**
   * Register a shortcut for a sound
   */
  async register(soundId: string, shortcut: string): Promise<void> {
    // Unregister old shortcut for this sound if exists
    const oldShortcut = this.getShortcutForSound(soundId);
    if (oldShortcut) {
      await this.unregister(oldShortcut);
    }

    // Register new shortcut
    try {
      await this.tauri.registerGlobalShortcut(soundId, shortcut);
      this.registry.set(shortcut, soundId);
    } catch (err) {
      console.error(`Failed to register shortcut ${shortcut}:`, err);
      throw err;
    }
  }

  /**
   * Unregister a shortcut
   */
  async unregister(shortcut: string): Promise<void> {
    try {
      await this.tauri.unregisterGlobalShortcut(shortcut);
      this.registry.delete(shortcut);
    } catch (err) {
      console.error(`Failed to unregister shortcut ${shortcut}:`, err);
    }
  }

  /**
   * Get shortcut assigned to a sound
   */
  getShortcutForSound(soundId: string): string | null {
    for (const [shortcut, id] of this.registry.entries()) {
      if (id === soundId) return shortcut;
    }
    return null;
  }

  /**
   * Enable or disable global hotkeys
   */
  async setEnabled(enabled: boolean): Promise<void> {
    try {
      await this.tauri.setGlobalHotkeysEnabled(enabled);
      this._enabled.set(enabled);
    } catch (err) {
      console.error('Failed to set global hotkeys enabled:', err);
      throw err;
    }
  }

  /**
   * Format a keyboard event as a shortcut string for recording.
   * Requires at least one modifier key (Ctrl, Alt, Shift, Meta/Cmd).
   */
  formatEventAsShortcut(event: KeyboardEvent): string | null {
    if (isModifierKey(event.key)) {
      return null; // Don't record modifier-only presses
    }

    // Require at least one modifier key
    if (!event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) {
      return null; // No modifiers pressed - invalid shortcut
    }

    const shortcut = shortcutFromEvent(event);
    return formatShortcut(shortcut);
  }
}
