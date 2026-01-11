// src/app/core/services/fuzzy-search.service.ts
import { Injectable } from '@angular/core';
import { Sound } from '../models';

export interface SearchResult {
  sound: Sound;
  score: number;
  matchedIndices: number[];
}

@Injectable({
  providedIn: 'root'
})
export class FuzzySearchService {

  search(query: string, sounds: Sound[]): SearchResult[] {
    const normalizedQuery = this.normalizeText(query);
    if (!normalizedQuery) return [];

    const results: SearchResult[] = [];

    for (const sound of sounds) {
      const displayName = sound.customName || sound.name;
      const normalizedName = this.normalizeText(displayName);

      const exactMatch = this.exactMatch(normalizedQuery, normalizedName, displayName);
      if (exactMatch) {
        results.push({ sound, ...exactMatch });
      }
    }

    return results.sort((a, b) => b.score - a.score);
  }

  private exactMatch(query: string, normalizedText: string, originalText: string): { score: number; matchedIndices: number[] } | null {
    const index = normalizedText.indexOf(query);
    if (index === -1) return null;

    const matchedIndices: number[] = [];
    for (let i = index; i < index + query.length; i++) {
      matchedIndices.push(i);
    }

    return { score: 100, matchedIndices };
  }

  private normalizeText(text: string): string {
    return text
      .toLowerCase()
      .normalize('NFD')
      .replace(/[\u0300-\u036f]/g, ''); // Remove accents
  }
}
