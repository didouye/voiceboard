import { Injectable } from '@angular/core';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';

export interface YouTubeAudioDto {
  temp_path: string;
  title: string;
  duration: number;
  video_id: string;
}

@Injectable({ providedIn: 'root' })
export class YouTubeService {

  /**
   * Validate YouTube URL format
   */
  isValidUrl(url: string): boolean {
    const pattern = /(?:youtube\.com\/watch\?v=|youtu\.be\/|youtube\.com\/embed\/)([a-zA-Z0-9_-]{11})/;
    return pattern.test(url);
  }

  /**
   * Download audio from YouTube URL
   */
  async download(url: string): Promise<YouTubeAudioDto> {
    return invoke<YouTubeAudioDto>('youtube_download', { url });
  }

  /**
   * Trim audio and import to soundboard
   */
  async trimAndImport(
    tempPath: string,
    startSeconds: number,
    endSeconds: number,
    name: string
  ): Promise<{ hash: string; name: string; path: string; duration: number }> {
    return invoke('youtube_trim_and_import', {
      tempPath,
      startSeconds,
      endSeconds,
      name,
    });
  }

  /**
   * Cancel import and cleanup temp file
   */
  async cancel(tempPath: string): Promise<void> {
    return invoke('youtube_cancel', { tempPath });
  }

  /**
   * Get asset URL for audio file (for wavesurfer)
   */
  getAudioUrl(tempPath: string): string {
    return convertFileSrc(tempPath);
  }
}
