/**
 * Tauri service - wrapper around Tauri's invoke and listen APIs
 */
import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

@Injectable({
  providedIn: 'root'
})
export class TauriService {
  /**
   * Invoke a Tauri command
   */
  async invoke<T>(command: string, args?: Record<string, any>): Promise<T> {
    try {
      return await invoke<T>(command, args);
    } catch (error) {
      console.error(`Tauri command '${command}' failed:`, error);
      throw error;
    }
  }

  /**
   * Listen to a Tauri event
   */
  async listen<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
    return await listen<T>(event, (event) => {
      handler(event.payload);
    });
  }

  /**
   * Check if running in Tauri context
   */
  isTauri(): boolean {
    return typeof window !== 'undefined' && '__TAURI__' in window;
  }
}
