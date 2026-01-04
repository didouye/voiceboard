import { Injectable } from '@angular/core';

/**
 * Service to detect and manage demo mode.
 * Demo mode is activated when:
 * - URL contains ?demo or ?demo=true
 * - Or when running outside of Tauri (no __TAURI_INTERNALS__)
 */
@Injectable({
  providedIn: 'root'
})
export class DemoService {
  private readonly _isDemoMode: boolean;

  constructor() {
    this._isDemoMode = this.detectDemoMode();

    if (this._isDemoMode) {
      console.log('[DemoService] Running in demo mode');
    }
  }

  get isDemoMode(): boolean {
    return this._isDemoMode;
  }

  private detectDemoMode(): boolean {
    // Check URL param
    const urlParams = new URLSearchParams(window.location.search);
    if (urlParams.has('demo')) {
      return true;
    }

    // Check if Tauri is available
    const hasTauri = !!(window as any).__TAURI_INTERNALS__;
    return !hasTauri;
  }
}
