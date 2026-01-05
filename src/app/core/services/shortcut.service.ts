import { Injectable, signal, OnDestroy } from '@angular/core';
import { TauriService } from './tauri.service';
import { SoundboardService } from './soundboard.service';
import { formatShortcut, shortcutFromEvent, isModifierKey } from '../models';

@Injectable({
  providedIn: 'root'
})
export class ShortcutService implements OnDestroy {
  // Map shortcut string -> pad id
  private registry = new Map<string, string>();

  // Enabled state
  private _enabled = signal(true);
  readonly enabled = this._enabled.asReadonly();

  private unlistenGlobalShortcut?: () => void;

  constructor(
    private tauri: TauriService,
    private soundboard: SoundboardService
  ) {
    this.init();
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
      this.soundboard.playSound(data.padId);
    });

    // Register all existing shortcuts from soundboard
    await this.syncFromSoundboard();
  }

  ngOnDestroy(): void {
    this.unlistenGlobalShortcut?.();
  }

  /**
   * Sync shortcuts from soundboard pads to backend
   */
  async syncFromSoundboard(): Promise<void> {
    const pads = this.soundboard.pads();

    // Clear existing
    await this.tauri.unregisterAllShortcuts();
    this.registry.clear();

    // Register all pad shortcuts
    for (const pad of pads) {
      if (pad.hotkey) {
        try {
          await this.tauri.registerGlobalShortcut(pad.id, pad.hotkey);
          this.registry.set(pad.hotkey, pad.id);
        } catch (err) {
          console.error(`Failed to register shortcut ${pad.hotkey} for ${pad.id}:`, err);
        }
      }
    }
  }

  /**
   * Check if a shortcut is already used by another pad
   * Returns the pad ID if conflict exists, null otherwise
   */
  checkConflict(shortcut: string, excludePadId?: string): string | null {
    const existingPadId = this.registry.get(shortcut);
    if (existingPadId && existingPadId !== excludePadId) {
      return existingPadId;
    }
    return null;
  }

  /**
   * Register a shortcut for a pad
   */
  async register(padId: string, shortcut: string): Promise<void> {
    // Unregister old shortcut for this pad if exists
    const oldShortcut = this.getShortcutForPad(padId);
    if (oldShortcut) {
      await this.unregister(oldShortcut);
    }

    // Register new shortcut
    try {
      await this.tauri.registerGlobalShortcut(padId, shortcut);
      this.registry.set(shortcut, padId);
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
   * Get shortcut assigned to a pad
   */
  getShortcutForPad(padId: string): string | null {
    for (const [shortcut, id] of this.registry.entries()) {
      if (id === padId) return shortcut;
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
