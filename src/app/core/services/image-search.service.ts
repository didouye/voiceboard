import { Injectable, signal, computed } from '@angular/core';

export interface ImageSearchResult {
  id: string;
  thumbnailUrl: string;
  fullUrl: string;
  attribution: string;
  photographer: string;
}

interface PexelsPhoto {
  id: number;
  photographer: string;
  src: {
    tiny: string;
    small: string;
    medium: string;
  };
}

interface PexelsResponse {
  photos: PexelsPhoto[];
  next_page?: string;
}

@Injectable({
  providedIn: 'root'
})
export class ImageSearchService {
  private _apiKey = signal<string | null>(null);
  private _loading = signal(false);
  private _error = signal<string | null>(null);

  readonly apiKey = this._apiKey.asReadonly();
  readonly loading = this._loading.asReadonly();
  readonly error = this._error.asReadonly();
  readonly hasApiKey = computed(() => !!this._apiKey());

  private readonly PEXELS_API_URL = 'https://api.pexels.com/v1';
  private readonly STORAGE_KEY = 'pexels_api_key';

  constructor() {
    this.loadApiKey();
  }

  /**
   * Load API key from localStorage
   */
  private loadApiKey(): void {
    const key = localStorage.getItem(this.STORAGE_KEY);
    if (key) {
      this._apiKey.set(key);
    }
  }

  /**
   * Set and persist the Pexels API key
   */
  setApiKey(key: string | null): void {
    if (key) {
      localStorage.setItem(this.STORAGE_KEY, key);
      this._apiKey.set(key);
    } else {
      localStorage.removeItem(this.STORAGE_KEY);
      this._apiKey.set(null);
    }
  }

  /**
   * Test if the API key is valid
   */
  async testApiKey(key: string): Promise<boolean> {
    try {
      const response = await fetch(`${this.PEXELS_API_URL}/search?query=test&per_page=1`, {
        headers: { Authorization: key }
      });
      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * Search for images
   */
  async search(query: string, page: number = 1, perPage: number = 12): Promise<ImageSearchResult[]> {
    const apiKey = this._apiKey();
    if (!apiKey) {
      throw new Error('Pexels API key not configured');
    }

    this._loading.set(true);
    this._error.set(null);

    try {
      const url = `${this.PEXELS_API_URL}/search?query=${encodeURIComponent(query)}&page=${page}&per_page=${perPage}`;
      const response = await fetch(url, {
        headers: { Authorization: apiKey }
      });

      if (!response.ok) {
        if (response.status === 401) {
          throw new Error('Invalid API key');
        }
        if (response.status === 429) {
          throw new Error('Rate limit reached, try again in 1 hour');
        }
        throw new Error(`Search failed: ${response.statusText}`);
      }

      const data: PexelsResponse = await response.json();

      return data.photos.map(photo => ({
        id: photo.id.toString(),
        thumbnailUrl: photo.src.tiny,
        fullUrl: photo.src.medium,
        attribution: `Photo by ${photo.photographer} on Pexels`,
        photographer: photo.photographer
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
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error('Failed to download image');
    }

    const contentType = response.headers.get('content-type') || '';
    let extension = 'jpg';
    if (contentType.includes('png')) extension = 'png';
    else if (contentType.includes('webp')) extension = 'webp';
    else if (contentType.includes('gif')) extension = 'gif';

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
    // Remove extension
    const nameWithoutExt = filename.replace(/\.[^/.]+$/, '');
    // Replace separators with spaces
    const words = nameWithoutExt.replace(/[-_]/g, ' ').split(/\s+/);
    // Take first 3 words
    return words.slice(0, 3).join(' ').toLowerCase();
  }
}
