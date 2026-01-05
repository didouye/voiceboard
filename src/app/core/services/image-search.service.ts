import { Injectable, signal, computed } from '@angular/core';
import { fetch } from '@tauri-apps/plugin-http';

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

    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'
      }
    });

    if (!response.ok) {
      throw new Error('Failed to get search token');
    }

    const html = await response.text();

    // Extract vqd token from HTML
    const vqdMatch = html.match(/vqd=["']([^"']+)["']/);
    if (!vqdMatch) {
      throw new Error('Could not extract search token');
    }

    return vqdMatch[1];
  }

  /**
   * Search for images using DuckDuckGo
   */
  async search(query: string, page: number = 1, perPage: number = 12): Promise<ImageSearchResult[]> {
    this._loading.set(true);
    this._error.set(null);

    try {
      // Get vqd token first
      const vqd = await this.getVqdToken(query);

      // Fetch images
      const url = `https://duckduckgo.com/i.js?l=fr-fr&o=json&q=${encodeURIComponent(query)}&vqd=${vqd}&p=${page}`;

      const response = await fetch(url, {
        method: 'GET',
        headers: {
          'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36',
          'Accept': 'application/json'
        }
      });

      if (!response.ok) {
        throw new Error('Search failed');
      }

      const data = await response.json() as { results?: Array<{ image: string; thumbnail: string; title: string }> };

      if (!data.results || data.results.length === 0) {
        return [];
      }

      // Map to our interface, limit to perPage
      return data.results.slice(0, perPage).map((item, index) => ({
        id: `ddg-${index}-${Date.now()}`,
        thumbnailUrl: item.thumbnail,
        fullUrl: item.image,
        title: item.title || 'Image'
      }));
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Search failed';
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
    const response = await fetch(url, {
      method: 'GET',
      headers: {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'
      }
    });

    if (!response.ok) {
      throw new Error('Failed to download image');
    }

    // Determine extension from URL or content-type
    const contentType = response.headers.get('content-type') || '';
    let extension = 'jpg';
    if (contentType.includes('png') || url.includes('.png')) extension = 'png';
    else if (contentType.includes('webp') || url.includes('.webp')) extension = 'webp';
    else if (contentType.includes('gif') || url.includes('.gif')) extension = 'gif';

    const buffer = await response.arrayBuffer();
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

  // Removed Pexels-specific methods:
  // - setApiKey()
  // - testApiKey()
  // - apiKey signal
}
