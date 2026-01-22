// src/app/core/services/fuzzy-search.service.ts
import { Injectable } from "@angular/core";
import { Sound } from "../models";

export interface SearchResult {
  sound: Sound;
  score: number;
  matchedIndices: number[];
}

@Injectable({
  providedIn: "root",
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
      const subseqMatch = this.subsequenceMatch(
        normalizedQuery,
        normalizedName,
      );
      if (subseqMatch) {
        results.push({ sound, ...subseqMatch });
        continue;
      }

      // Try levenshtein match (only for queries >= 3 chars)
      if (normalizedQuery.length >= 3) {
        const levMatch = this.levenshteinMatch(normalizedQuery, normalizedName);
        if (levMatch) {
          results.push({ sound, ...levMatch });
        }
      }
    }

    return results.sort((a, b) => b.score - a.score);
  }

  private exactMatch(
    query: string,
    normalizedText: string,
  ): { score: number; matchedIndices: number[] } | null {
    const index = normalizedText.indexOf(query);
    if (index === -1) return null;

    const matchedIndices: number[] = [];
    for (let i = index; i < index + query.length; i++) {
      matchedIndices.push(i);
    }

    return { score: 100, matchedIndices };
  }

  private subsequenceMatch(
    query: string,
    text: string,
  ): { score: number; matchedIndices: number[] } | null {
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
    const score = Math.round(60 + density * 30); // 60-90 range

    return { score: Math.min(score, 90), matchedIndices: indices };
  }

  private levenshteinMatch(
    query: string,
    text: string,
  ): { score: number; matchedIndices: number[] } | null {
    // Skip very long texts for performance
    if (text.length > 50) return null;

    const distance = this.levenshteinDistance(query, text);

    // Max allowed errors: roughly 1 error per 3 characters, max 3
    const maxErrors = Math.min(3, Math.floor(query.length / 3) + 1);

    if (distance > maxErrors) return null;

    // Score inversely proportional to distance (10-50 range)
    const score = Math.max(10, 50 - distance * 15);

    return { score, matchedIndices: [] }; // No highlighting for levenshtein
  }

  private levenshteinDistance(a: string, b: string): number {
    const matrix: number[][] = [];

    for (let i = 0; i <= a.length; i++) {
      matrix[i] = [i];
    }
    for (let j = 0; j <= b.length; j++) {
      matrix[0][j] = j;
    }

    for (let i = 1; i <= a.length; i++) {
      for (let j = 1; j <= b.length; j++) {
        const cost = a[i - 1] === b[j - 1] ? 0 : 1;
        matrix[i][j] = Math.min(
          matrix[i - 1][j] + 1, // deletion
          matrix[i][j - 1] + 1, // insertion
          matrix[i - 1][j - 1] + cost, // substitution
        );
      }
    }

    return matrix[a.length][b.length];
  }

  private normalizeText(text: string): string {
    return text
      .toLowerCase()
      .normalize("NFD")
      .replace(/[\u0300-\u036f]/g, ""); // Remove accents
  }
}
