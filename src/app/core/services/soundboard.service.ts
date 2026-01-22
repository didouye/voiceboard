import { Injectable, signal, computed } from "@angular/core";
import { TauriService } from "./tauri.service";
import { FuzzySearchService } from "./fuzzy-search.service";
import { MixerService } from "./mixer.service";
import { Sound, SoundPad, Folder, PadImage } from "../models";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";

const PAD_COLORS = [
  "#e74c3c",
  "#e67e22",
  "#f1c40f",
  "#2ecc71",
  "#1abc9c",
  "#3498db",
  "#9b59b6",
  "#e91e63",
  "#00bcd4",
  "#8bc34a",
  "#ff5722",
  "#795548",
];

@Injectable({
  providedIn: "root",
})
export class SoundboardService {
  // Source of truth
  private _sounds = signal<Map<string, Sound>>(new Map());
  private _folders = signal<Folder[]>([
    { id: "all", name: "Tous", createdAt: Date.now() },
  ]);
  private _activeFolderId = signal<string>("all");
  private _loading = signal(false);
  private _error = signal<string | null>(null);
  private _searchQuery = signal<string>("");
  private _initialized = false;

  // Preview state
  private _previewingSoundId = signal<string | null>(null);
  private _previewDeviceId = signal<string | null>(null);
  readonly previewingSoundId = this._previewingSoundId.asReadonly();
  readonly previewDeviceId = this._previewDeviceId.asReadonly();

  private unlistenPreviewStarted?: () => void;
  private unlistenPreviewStopped?: () => void;

  // Public readonly signals
  readonly sounds = this._sounds.asReadonly();
  readonly folders = this._folders.asReadonly();
  readonly activeFolderId = this._activeFolderId.asReadonly();
  readonly loading = this._loading.asReadonly();
  readonly error = this._error.asReadonly();
  readonly searchQuery = this._searchQuery.asReadonly();

  readonly activeFolder = computed(
    () =>
      this._folders().find((f) => f.id === this._activeFolderId()) ||
      this._folders()[0],
  );

  readonly activeSounds = computed(() => Array.from(this._sounds().values()));

  readonly playingCount = computed(
    () => Array.from(this._sounds().values()).filter((s) => s.isPlaying).length,
  );

  /**
   * Virtual pad grid computed from sounds and active folder
   */
  readonly pads = computed(() => {
    const sounds = this._sounds();
    const activeFolderId = this._activeFolderId();
    const searchQuery = this._searchQuery();

    // Filter sounds by folder
    let filteredSounds = Array.from(sounds.values());
    if (activeFolderId !== "all") {
      filteredSounds = filteredSounds.filter((s) =>
        s.folderIds.includes(activeFolderId),
      );
    }

    // Apply search filter
    let sortedSounds: Array<{ sound: Sound; matchedIndices: number[] }>;

    if (searchQuery.trim()) {
      const searchResults = this.fuzzySearch.search(
        searchQuery,
        filteredSounds,
      );
      sortedSounds = searchResults.map((r) => ({
        sound: r.sound,
        matchedIndices: r.matchedIndices,
      }));
    } else {
      // No search: sort alphabetically
      filteredSounds.sort((a, b) =>
        (a.customName || a.name)
          .toLowerCase()
          .localeCompare((b.customName || b.name).toLowerCase()),
      );
      sortedSounds = filteredSounds.map((s) => ({
        sound: s,
        matchedIndices: [],
      }));
    }

    // Generate virtual grid
    const minPads = Math.max(12, Math.ceil(sortedSounds.length / 4) * 4 + 4);
    const pads: SoundPad[] = [];

    for (let i = 0; i < minPads; i++) {
      const item = sortedSounds[i];
      pads.push({
        index: i,
        sound: item?.sound || null,
        color: PAD_COLORS[i % PAD_COLORS.length],
        matchedIndices: item?.matchedIndices || [],
      });
    }

    return pads;
  });

  constructor(
    private tauri: TauriService,
    private fuzzySearch: FuzzySearchService,
    private mixer: MixerService,
  ) {
    this.loadState();
    this.initPreviewListeners();
    this.initSoundFinishedListener();
  }

  // =========================================================================
  // Sound Operations
  // =========================================================================

  /**
   * Get a sound by ID
   */
  getSound(soundId: string): Sound | undefined {
    return this._sounds().get(soundId);
  }

  /**
   * Add a sound to the store
   */
  private addSound(sound: Sound): void {
    this._sounds.update((sounds) => {
      const updated = new Map(sounds);
      updated.set(sound.id, sound);
      return updated;
    });
  }

  /**
   * Update a sound in the store
   */
  private updateSound(soundId: string, updates: Partial<Sound>): void {
    this._sounds.update((sounds) => {
      const sound = sounds.get(soundId);
      if (!sound) return sounds;

      const updated = new Map(sounds);
      updated.set(soundId, { ...sound, ...updates });
      return updated;
    });
  }

  /**
   * Remove a sound from the store
   */
  removeSound(soundId: string): void {
    this._sounds.update((sounds) => {
      const updated = new Map(sounds);
      updated.delete(soundId);
      return updated;
    });
    this.saveState();
  }

  /**
   * Play a sound
   */
  async playSound(soundId: string): Promise<void> {
    const sound = this._sounds().get(soundId);
    if (!sound) return;

    try {
      // If already playing, stop it first
      if (sound.isPlaying) {
        await this.stopSound(soundId);
        return;
      }

      // Calculate effective volume:
      // If sound volume was modified (not 1.0), use it
      // Otherwise, use global soundboard volume
      const effectiveVolume =
        sound.volume !== 1.0 ? sound.volume : this.mixer.soundboardVolume();

      await this.tauri.playSound(
        soundId,
        sound.path,
        effectiveVolume,
        sound.speed,
      );
      this.updateSound(soundId, { isPlaying: true });
    } catch (err) {
      console.error("Failed to play sound:", err);
      this._error.set("Failed to play sound");
    }
  }

  /**
   * Stop a specific sound
   */
  async stopSound(soundId: string): Promise<void> {
    const sound = this._sounds().get(soundId);
    if (!sound) return;

    try {
      await this.tauri.stopSound(soundId);
      this.updateSound(soundId, { isPlaying: false });
    } catch (err) {
      console.error("Failed to stop sound:", err);
    }
  }

  /**
   * Stop all playing sounds
   */
  async stopAll(): Promise<void> {
    try {
      await this.tauri.stopAllSounds();
      this._sounds.update((sounds) => {
        const updated = new Map(sounds);
        for (const [id, sound] of updated) {
          if (sound.isPlaying) {
            updated.set(id, { ...sound, isPlaying: false });
          }
        }
        return updated;
      });
    } catch (err) {
      console.error("Failed to stop all sounds:", err);
    }
  }

  /**
   * Preview a sound on the preview output device
   */
  async previewSound(soundId: string): Promise<void> {
    const sound = this._sounds().get(soundId);
    if (!sound) return;

    try {
      if (this._previewingSoundId() === soundId) {
        await this.tauri.stopPreview();
        this._previewingSoundId.set(null);
      } else {
        const previewDeviceId = this._previewDeviceId() || "default";
        await this.tauri.previewSound(sound.path, previewDeviceId, soundId);
        this._previewingSoundId.set(soundId);
      }
    } catch (err) {
      console.error("Failed to preview sound:", err);
    }
  }

  /**
   * Stop the currently playing preview
   */
  async stopPreview(): Promise<void> {
    try {
      await this.tauri.stopPreview();
      this._previewingSoundId.set(null);
    } catch (err) {
      this._error.set(
        err instanceof Error ? err.message : "Failed to stop preview",
      );
    }
  }

  // =========================================================================
  // Sound Property Setters
  // =========================================================================

  setSoundVolume(soundId: string, volume: number): void {
    const clampedVolume = Math.max(0, Math.min(2, volume));
    this.updateSound(soundId, { volume: clampedVolume });
    this.saveState();
  }

  setSoundSpeed(soundId: string, speed: number): void {
    const clampedSpeed = Math.max(0.5, Math.min(2, speed));
    this.updateSound(soundId, { speed: clampedSpeed });
    this.saveState();
  }

  setSoundHotkey(soundId: string, hotkey: string | null): void {
    // If setting a new hotkey, clear it from any other sound first
    if (hotkey) {
      this._sounds.update((sounds) => {
        const updated = new Map(sounds);
        for (const [id, sound] of updated) {
          if (sound.hotkey === hotkey && id !== soundId) {
            updated.set(id, { ...sound, hotkey: undefined });
          }
        }
        return updated;
      });
    }
    this.updateSound(soundId, { hotkey: hotkey || undefined });
    this.saveState();
  }

  setSoundCustomName(soundId: string, customName: string | null): void {
    this.updateSound(soundId, { customName: customName || undefined });
    this.saveState();
  }

  setSoundImage(soundId: string, image: PadImage | null): void {
    this.updateSound(soundId, { image: image || undefined });
    this.saveState();
  }

  // =========================================================================
  // Import Operations
  // =========================================================================

  /**
   * Import a single sound file
   */
  async importSound(): Promise<void> {
    try {
      this._loading.set(true);
      this._error.set(null);

      const selected = await open({
        multiple: false,
        filters: [{ name: "Audio", extensions: ["mp3", "ogg", "wav", "flac"] }],
      });

      if (!selected) return;

      const path = selected as string;
      const imported = await this.tauri.importAndNormalizeSound(path);

      // Check for duplicate
      if (this._sounds().has(imported.hash)) {
        this._error.set("This sound already exists in your library");
        return;
      }

      const sound: Sound = {
        id: imported.hash,
        name: imported.name,
        path: imported.path,
        duration: imported.duration,
        volume: 1.0,
        speed: 1.0,
        folderIds: [],
        isPlaying: false,
        addedAt: Date.now(),
      };

      this.addSound(sound);
      await this.saveState();
    } catch (err) {
      console.error("Failed to import sound:", err);
      this._error.set("Failed to import sound");
    } finally {
      this._loading.set(false);
    }
  }

  /**
   * Import multiple sound files
   */
  async importMultipleSounds(): Promise<{
    imported: number;
    skippedDuplicates: number;
    errors: string[];
  }> {
    const result = {
      imported: 0,
      skippedDuplicates: 0,
      errors: [] as string[],
    };

    try {
      this._loading.set(true);
      this._error.set(null);

      const selected = await open({
        multiple: true,
        filters: [{ name: "Audio", extensions: ["mp3", "ogg", "wav", "flac"] }],
      });

      if (!selected || (Array.isArray(selected) && selected.length === 0)) {
        return result;
      }

      const paths = Array.isArray(selected) ? selected : [selected];
      const importResults = await this.tauri.importMultipleAndNormalize(paths);

      for (const res of importResults) {
        if ("err" in res) {
          result.errors.push(res.err);
          continue;
        }

        const imported = res.ok;

        // Check for duplicate
        if (this._sounds().has(imported.hash)) {
          result.skippedDuplicates++;
          continue;
        }

        const sound: Sound = {
          id: imported.hash,
          name: imported.name,
          path: imported.path,
          duration: imported.duration,
          volume: 1.0,
          speed: 1.0,
          folderIds: [],
          isPlaying: false,
          addedAt: Date.now(),
        };

        this.addSound(sound);
        result.imported++;
      }

      if (result.imported > 0) {
        await this.saveState();
      }
    } catch (err) {
      console.error("Failed to import sounds:", err);
      result.errors.push(String(err));
    } finally {
      this._loading.set(false);
    }

    return result;
  }

  /**
   * Import sounds from file paths (for drag & drop)
   */
  async importSoundsFromPaths(paths: string[]): Promise<{
    imported: number;
    skippedDuplicates: number;
    errors: string[];
  }> {
    const result = {
      imported: 0,
      skippedDuplicates: 0,
      errors: [] as string[],
    };

    try {
      this._loading.set(true);
      const importResults = await this.tauri.importMultipleAndNormalize(paths);

      for (const res of importResults) {
        if ("err" in res) {
          result.errors.push(res.err);
          continue;
        }

        const imported = res.ok;

        if (this._sounds().has(imported.hash)) {
          result.skippedDuplicates++;
          continue;
        }

        const sound: Sound = {
          id: imported.hash,
          name: imported.name,
          path: imported.path,
          duration: imported.duration,
          volume: 1.0,
          speed: 1.0,
          folderIds: [],
          isPlaying: false,
          addedAt: Date.now(),
        };

        this.addSound(sound);
        result.imported++;
      }

      if (result.imported > 0) {
        await this.saveState();
      }
    } catch (err) {
      result.errors.push(String(err));
    } finally {
      this._loading.set(false);
    }

    return result;
  }

  // =========================================================================
  // Folder Operations
  // =========================================================================

  setActiveFolder(folderId: string): void {
    if (this._folders().some((f) => f.id === folderId)) {
      this._activeFolderId.set(folderId);
      this._searchQuery.set("");
    }
  }

  setSearchQuery(query: string): void {
    this._searchQuery.set(query);
  }

  createFolder(name: string): void {
    const trimmedName = name.trim();
    if (!trimmedName) return;

    if (
      this._folders().some(
        (f) => f.name.toLowerCase() === trimmedName.toLowerCase(),
      )
    ) {
      return;
    }

    const id = `folder-${Date.now()}`;
    const newFolder: Folder = { id, name: trimmedName, createdAt: Date.now() };
    this._folders.update((folders) => [...folders, newFolder]);
    this.saveFolders();
  }

  renameFolder(folderId: string, newName: string): void {
    if (folderId === "all") return;

    const trimmedName = newName.trim();
    if (!trimmedName) return;

    if (
      this._folders().some(
        (f) =>
          f.id !== folderId &&
          f.name.toLowerCase() === trimmedName.toLowerCase(),
      )
    ) {
      return;
    }

    this._folders.update((folders) =>
      folders.map((f) => (f.id === folderId ? { ...f, name: trimmedName } : f)),
    );
    this.saveFolders();
  }

  deleteFolder(folderId: string): void {
    if (folderId === "all") return;

    // Remove folder from all sounds
    this._sounds.update((sounds) => {
      const updated = new Map(sounds);
      for (const [id, sound] of updated) {
        if (sound.folderIds.includes(folderId)) {
          updated.set(id, {
            ...sound,
            folderIds: sound.folderIds.filter((f) => f !== folderId),
          });
        }
      }
      return updated;
    });

    this._folders.update((folders) => folders.filter((f) => f.id !== folderId));

    if (this._activeFolderId() === folderId) {
      this._activeFolderId.set("all");
    }

    this.saveFolders();
    this.saveState();
  }

  toggleSoundFolder(soundId: string, folderId: string): void {
    if (folderId === "all") return;

    const sound = this._sounds().get(soundId);
    if (!sound) return;

    const hasFolder = sound.folderIds.includes(folderId);
    this.updateSound(soundId, {
      folderIds: hasFolder
        ? sound.folderIds.filter((f) => f !== folderId)
        : [...sound.folderIds, folderId],
    });
    this.saveState();
  }

  addSoundToFolder(soundId: string, folderId: string): void {
    if (folderId === "all") return;

    const sound = this._sounds().get(soundId);
    if (!sound || sound.folderIds.includes(folderId)) return;

    this.updateSound(soundId, {
      folderIds: [...sound.folderIds, folderId],
    });
    this.saveState();
  }

  // =========================================================================
  // Persistence
  // =========================================================================

  private async loadState(): Promise<void> {
    try {
      // Load folders
      const savedFolders = await this.tauri.loadFolders();
      if (savedFolders && savedFolders.length > 0) {
        const hasAll = savedFolders.some((f) => f.id === "all");
        if (!hasAll) {
          savedFolders.unshift({ id: "all", name: "Tous", createdAt: 0 });
        }
        this._folders.set(savedFolders);
      }

      // Load sounds (new format)
      const data = await this.tauri.loadSoundboardState();

      if (
        data &&
        typeof data === "object" &&
        !Array.isArray(data) &&
        (data as any).sounds
      ) {
        // New format: sounds as object
        const soundsObj = (data as any).sounds as Record<string, any>;
        const soundsMap = new Map<string, Sound>();
        for (const [id, soundData] of Object.entries(soundsObj)) {
          soundsMap.set(id, {
            ...soundData,
            isPlaying: false, // Reset runtime state
          } as Sound);
        }
        this._sounds.set(soundsMap);
      } else if (data && Array.isArray(data)) {
        // Old format: pads array - migrate
        await this.migrateFromOldFormat(data);
      }

      this._initialized = true;

      // Migrate any sounds that aren't in AppData yet
      await this.migrateExistingSoundsToNormalized();
    } catch (err) {
      console.error("Failed to load state:", err);
      this._initialized = true;
    }

    // Also load preview device setting
    this.loadPreviewDevice();
  }

  /**
   * Migrate existing sounds to normalized WAV format in AppData
   * This handles sounds imported before normalization was added
   */
  private async migrateExistingSoundsToNormalized(): Promise<void> {
    try {
      const soundsDir = await this.tauri.getSoundsDir();
      const sounds = this._sounds();
      let migratedCount = 0;

      for (const [id, sound] of sounds) {
        // Check if sound is already in AppData sounds directory
        if (sound.path.startsWith(soundsDir)) {
          continue;
        }

        // Check if original file still exists by trying to migrate
        try {
          console.log(`Migrating sound: ${sound.name} (${sound.path})`);
          const newPath = await this.tauri.migrateSoundToNormalized(
            sound.path,
            id,
          );

          // Update sound with new path
          this.updateSound(id, { path: newPath });
          migratedCount++;
        } catch (err) {
          console.error(`Failed to migrate sound ${sound.name}:`, err);
          // Keep the old path - sound might still work or user can re-import
        }
      }

      if (migratedCount > 0) {
        console.log(`Migrated ${migratedCount} sounds to normalized format`);
        await this.saveState();
      }
    } catch (err) {
      console.error("Failed to migrate sounds:", err);
    }
  }

  private async migrateFromOldFormat(pads: any[]): Promise<void> {
    console.log("Migrating from old pad format to new sound format...");
    const sounds = new Map<string, Sound>();

    for (const pad of pads) {
      if (!pad.sound) continue;

      try {
        // Calculate hash for existing file
        const hash = await this.tauri.hashFile(pad.sound.path);

        // Skip if already migrated (duplicate)
        if (sounds.has(hash)) continue;

        const sound: Sound = {
          id: hash,
          name: pad.sound.name,
          path: pad.sound.path,
          duration: pad.sound.duration || 0,
          volume: pad.volume ?? 1.0,
          speed: pad.speed ?? 1.0,
          hotkey: pad.hotkey,
          customName: pad.customName,
          image: pad.image,
          folderIds: pad.folderIds ?? [],
          isPlaying: false,
          addedAt: Date.now(),
        };

        sounds.set(hash, sound);
      } catch (err) {
        console.error(`Failed to migrate sound ${pad.sound?.name}:`, err);
      }
    }

    this._sounds.set(sounds);
    await this.saveState();
    console.log(`Migration complete: ${sounds.size} sounds migrated`);
  }

  private async saveState(): Promise<void> {
    if (!this._initialized) return;

    try {
      // Convert Map to object for JSON serialization
      const soundsObj: Record<string, Omit<Sound, "isPlaying">> = {};
      for (const [id, sound] of this._sounds()) {
        const { isPlaying, ...rest } = sound;
        soundsObj[id] = rest;
      }

      await this.tauri.saveSoundboardState({ sounds: soundsObj } as any);
    } catch (err) {
      console.error("Failed to save state:", err);
    }
  }

  private async saveFolders(): Promise<void> {
    try {
      await this.tauri.saveFolders(this._folders());
    } catch (err) {
      console.error("Failed to save folders:", err);
    }
  }

  // =========================================================================
  // Preview Device
  // =========================================================================

  /**
   * Set the preview output device
   */
  async setPreviewDevice(deviceId: string | null): Promise<void> {
    this._previewDeviceId.set(deviceId);
    try {
      await this.tauri.setPreviewDevice(deviceId);
    } catch (err) {
      this._error.set(
        err instanceof Error ? err.message : "Failed to set preview device",
      );
    }
  }

  /**
   * Load preview device from settings
   */
  async loadPreviewDevice(): Promise<void> {
    try {
      const settings = await this.tauri.loadSettings();
      this._previewDeviceId.set(settings.audio.previewDeviceId);
    } catch (err) {
      console.error("Failed to load preview device:", err);
    }
  }

  // =========================================================================
  // Listeners
  // =========================================================================

  private async initSoundFinishedListener(): Promise<void> {
    try {
      await listen<{ id: string }>("sound-finished", (event) => {
        // The backend sends soundId as the id
        const soundId = event.payload.id;
        if (this._sounds().has(soundId)) {
          this.updateSound(soundId, { isPlaying: false });
        }
      });
    } catch (e) {
      console.error("Failed to initialize sound-finished listener:", e);
    }
  }

  private async initPreviewListeners(): Promise<void> {
    this.unlistenPreviewStarted = await this.tauri.listenPreviewStarted(
      (soundId) => {
        this._previewingSoundId.set(soundId);
      },
    );

    this.unlistenPreviewStopped = await this.tauri.listenPreviewStopped(() => {
      this._previewingSoundId.set(null);
    });
  }

  // =========================================================================
  // Utilities
  // =========================================================================

  formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, "0")}`;
  }

  clearError(): void {
    this._error.set(null);
  }
}
