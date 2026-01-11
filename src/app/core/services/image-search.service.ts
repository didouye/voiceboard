import { inject, Injectable, signal, computed } from '@angular/core';
import { fetch } from '@tauri-apps/plugin-http';
import { DebugConsoleService } from './debug-console.service';

export interface ImageSearchResult {
  id: string;
  thumbnailUrl: string;
  fullUrl: string;
  title: string;
}

@Injectable({
  providedIn: 'root'
})
export class ImageSearchService {
  private debug = inject(DebugConsoleService);
  private _loading = signal(false);
  private _error = signal<string | null>(null);

  readonly loading = this._loading.asReadonly();
  readonly error = this._error.asReadonly();

  // Always available (no API key needed)
  readonly hasApiKey = computed(() => true);

  /**
   * Get vqd token from DuckDuckGo
   */
  private async getVqdToken(query: string): Promise<string> {
    const url = `https://duckduckgo.com/?q=${encodeURIComponent(query)}&iax=images&ia=images`;
    this.debug.log('debug', '[ImageSearch] Fetching vqd token', { query, url });

    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8',
        'Accept-Language': 'en-US,en;q=0.9'
      }
    });

    this.debug.log('debug', '[ImageSearch] vqd token response', { status: response.status, ok: response.ok });

    if (!response.ok) {
      this.debug.log('error', '[ImageSearch] Failed to get vqd token', { status: response.status });
      throw new Error(`Failed to get search token (status: ${response.status})`);
    }

    const html = await response.text();
    this.debug.log('debug', '[ImageSearch] HTML received', { length: html.length });

    // Extract vqd token from HTML
    const vqdMatch = html.match(/vqd=["']([^"']+)["']/);
    if (!vqdMatch) {
      this.debug.log('error', '[ImageSearch] Could not extract vqd token from HTML');
      throw new Error('Could not extract search token');
    }

    this.debug.log('debug', '[ImageSearch] vqd token extracted', { token: vqdMatch[1].substring(0, 20) + '...' });
    return vqdMatch[1];
  }

  /**
   * Search for images using DuckDuckGo
   */
  async search(query: string, page: number = 1, perPage: number = 12): Promise<ImageSearchResult[]> {
    this.debug.log('info', '[ImageSearch] Starting search', { query, page, perPage });
    this._loading.set(true);
    this._error.set(null);

    try {
      // Get vqd token first
      const vqd = await this.getVqdToken(query);

      // Fetch images
      const url = `https://duckduckgo.com/i.js?l=fr-fr&o=json&q=${encodeURIComponent(query)}&vqd=${vqd}&p=${page}`;
      this.debug.log('debug', '[ImageSearch] Fetching images', { url });

      const response = await fetch(url, {
        method: 'GET',
        headers: {
          'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
          'Accept': 'application/json, text/javascript, */*; q=0.01',
          'Accept-Language': 'en-US,en;q=0.9',
          'Referer': 'https://duckduckgo.com/',
          'X-Requested-With': 'XMLHttpRequest'
        }
      });

      this.debug.log('debug', '[ImageSearch] Images response', { status: response.status, ok: response.ok });

      if (!response.ok) {
        this.debug.log('error', '[ImageSearch] Search request failed', { status: response.status });
        throw new Error(`Search failed (status: ${response.status})`);
      }

      const data = await response.json() as { results?: Array<{ image: string; thumbnail: string; title: string }> };
      this.debug.log('debug', '[ImageSearch] Results received', { count: data.results?.length ?? 0 });

      if (!data.results || data.results.length === 0) {
        this.debug.log('warn', '[ImageSearch] No results found');
        return [];
      }

      // Map to our interface, limit to perPage
      const results = data.results.slice(0, perPage).map((item, index) => ({
        id: `ddg-${index}-${Date.now()}`,
        thumbnailUrl: item.thumbnail,
        fullUrl: item.image,
        title: item.title || 'Image'
      }));

      this.debug.log('info', '[ImageSearch] Search completed', { resultCount: results.length });
      return results;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Search failed';
      this.debug.log('error', '[ImageSearch] Search error', { error: message });
      this._error.set(message);
      throw err;
    } finally {
      this._loading.set(false);
    }
  }

  /**
   * Download an image and return as Uint8Array
   */
  async downloadImage(url: string): Promise<{ data: Uint8Array; extension: string }> {
    this.debug.log('info', '[ImageSearch] Downloading image', { url: url.substring(0, 100) + '...' });

    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        'Accept': 'image/webp,image/apng,image/*,*/*;q=0.8'
      }
    });

    this.debug.log('debug', '[ImageSearch] Download response', { status: response.status, ok: response.ok });

    if (!response.ok) {
      this.debug.log('error', '[ImageSearch] Failed to download image', { status: response.status });
      throw new Error('Failed to download image');
    }

    // Determine extension from URL or content-type
    const contentType = response.headers.get('content-type') || '';
    let extension = 'jpg';
    if (contentType.includes('png') || url.includes('.png')) extension = 'png';
    else if (contentType.includes('webp') || url.includes('.webp')) extension = 'webp';
    else if (contentType.includes('gif') || url.includes('.gif')) extension = 'gif';

    const buffer = await response.arrayBuffer();
    this.debug.log('info', '[ImageSearch] Image downloaded', { extension, size: buffer.byteLength });

    return {
      data: new Uint8Array(buffer),
      extension
    };
  }

  /**
   * Extract search query from filename
   * "funny_airhorn_sound.mp3" -> "funny airhorn sound"
   * Takes first 3 words max
   */
  extractQueryFromFilename(filename: string): string {
    const nameWithoutExt = filename.replace(/\.[^/.]+$/, '');
    const words = nameWithoutExt.replace(/[-_]/g, ' ').split(/\s+/);
    return words.slice(0, 3).join(' ').toLowerCase();
  }

}
