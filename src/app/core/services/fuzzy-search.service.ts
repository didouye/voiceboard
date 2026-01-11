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

      // Try exact match first (highest priority)
      const exactMatch = this.exactMatch(normalizedQuery, normalizedName);
      if (exactMatch) {
        results.push({ sound, ...exactMatch });
        continue;
      }

      // Try subsequence match
      const subseqMatch = this.subsequenceMatch(normalizedQuery, normalizedName);
      if (subseqMatch) {
        results.push({ sound, ...subseqMatch });
      }
    }

    return results.sort((a, b) => b.score - a.score);
  }

  private exactMatch(query: string, normalizedText: string): { score: number; matchedIndices: number[] } | null {
    const index = normalizedText.indexOf(query);
    if (index === -1) return null;

    const matchedIndices: number[] = [];
    for (let i = index; i < index + query.length; i++) {
      matchedIndices.push(i);
    }

    return { score: 100, matchedIndices };
  }

  private subsequenceMatch(query: string, text: string): { score: number; matchedIndices: number[] } | null {
    const indices: number[] = [];
    let queryIdx = 0;

    for (let i = 0; i < text.length && queryIdx < query.length; i++) {
      if (text[i] === query[queryIdx]) {
        indices.push(i);
        queryIdx++;
      }
    }

    // All characters must be found
    if (queryIdx !== query.length) return null;

    // Score based on density (how close together the matches are)
    const span = indices[indices.length - 1] - indices[0] + 1;
    const density = query.length / span;
    const score = Math.round(60 + (density * 30)); // 60-90 range

    return { score: Math.min(score, 90), matchedIndices: indices };
  }

  private normalizeText(text: string): string {
    return text
      .toLowerCase()
      .normalize('NFD')
      .replace(/[\u0300-\u036f]/g, ''); // Remove accents
  }
}
