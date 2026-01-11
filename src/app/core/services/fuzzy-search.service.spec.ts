// src/app/core/services/fuzzy-search.service.spec.ts
import { TestBed } from '@angular/core/testing';
import { FuzzySearchService, SearchResult } from './fuzzy-search.service';
import { Sound } from '../models';

describe('FuzzySearchService', () => {
  let service: FuzzySearchService;

  const createMockSound = (overrides: Partial<Sound> = {}): Sound => ({
    id: 'hash_abc123',
    name: 'test-sound',
    path: '/path/to/test-sound.mp3',
    duration: 5.0,
    volume: 1.0,
    speed: 1.0,
    folderIds: [],
    isPlaying: false,
    addedAt: Date.now(),
    ...overrides
  });

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [FuzzySearchService]
    });
    service = TestBed.inject(FuzzySearchService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });

  describe('exact match', () => {
    it('should find exact substring match', () => {
      const sounds = [createMockSound({ id: '1', name: 'barbe bleue' })];
      const results = service.search('barbe', sounds);

      expect(results.length).toBe(1);
      expect(results[0].sound.id).toBe('1');
      expect(results[0].score).toBe(100);
    });

    it('should be case insensitive', () => {
      const sounds = [createMockSound({ id: '1', name: 'Barbe Bleue' })];
      const results = service.search('BARBE', sounds);

      expect(results.length).toBe(1);
      expect(results[0].score).toBe(100);
    });

    it('should return matched indices for exact match', () => {
      const sounds = [createMockSound({ id: '1', name: 'barbe bleue' })];
      const results = service.search('barbe', sounds);

      expect(results[0].matchedIndices).toEqual([0, 1, 2, 3, 4]);
    });

    it('should use customName when available', () => {
      const sounds = [createMockSound({ id: '1', name: 'original', customName: 'barbe' })];
      const results = service.search('barbe', sounds);

      expect(results.length).toBe(1);
    });

    it('should return empty array for no matches', () => {
      const sounds = [createMockSound({ id: '1', name: 'test' })];
      const results = service.search('xyz', sounds);

      expect(results.length).toBe(0);
    });
  });

  describe('subsequence match', () => {
    it('should find subsequence match', () => {
      const sounds = [createMockSound({ id: '1', name: 'barbe bleue' })];
      const results = service.search('bbe', sounds);

      expect(results.length).toBe(1);
      expect(results[0].score).toBeGreaterThan(0);
      expect(results[0].score).toBeLessThan(100);
    });

    it('should return correct indices for subsequence', () => {
      const sounds = [createMockSound({ id: '1', name: 'barbe bleue' })];
      const results = service.search('bbe', sounds);

      // b(0), b(3 or 6), e(4 or 10) - depending on implementation
      expect(results[0].matchedIndices.length).toBe(3);
    });

    it('should score denser subsequences higher', () => {
      const sounds = [
        createMockSound({ id: '1', name: 'abc' }),      // dense: a(0)b(1)c(2)
        createMockSound({ id: '2', name: 'a---b---c' }) // sparse
      ];
      const results = service.search('abc', sounds);

      // First should have exact match (score 100), second subsequence
      const denseResult = results.find(r => r.sound.id === '1');
      const sparseResult = results.find(r => r.sound.id === '2');

      expect(denseResult!.score).toBeGreaterThan(sparseResult!.score);
    });

    it('should prefer exact match over subsequence', () => {
      const sounds = [createMockSound({ id: '1', name: 'barbe' })];
      const results = service.search('barbe', sounds);

      expect(results[0].score).toBe(100); // Exact, not subsequence
    });
  });

  describe('levenshtein match', () => {
    it('should find match with typo (substitution)', () => {
      const sounds = [createMockSound({ id: '1', name: 'barbe' })];
      const results = service.search('babre', sounds);

      expect(results.length).toBe(1);
      expect(results[0].score).toBeGreaterThan(0);
      expect(results[0].score).toBeLessThan(60); // Lower than subsequence
    });

    it('should find match with missing character', () => {
      const sounds = [createMockSound({ id: '1', name: 'barbe' })];
      const results = service.search('barb', sounds);

      expect(results.length).toBe(1);
    });

    it('should find match with extra character', () => {
      const sounds = [createMockSound({ id: '1', name: 'barbe' })];
      const results = service.search('barbee', sounds);

      expect(results.length).toBe(1);
    });

    it('should not match if too many errors', () => {
      const sounds = [createMockSound({ id: '1', name: 'barbe' })];
      const results = service.search('xxxxx', sounds);

      expect(results.length).toBe(0);
    });

    it('should not use levenshtein for queries under 3 characters', () => {
      const sounds = [createMockSound({ id: '1', name: 'ab' })];
      const results = service.search('ax', sounds);

      // 'ax' vs 'ab' is 1 edit, but query is too short
      expect(results.length).toBe(0);
    });

    it('should not return matched indices for levenshtein', () => {
      const sounds = [createMockSound({ id: '1', name: 'barbe' })];
      const results = service.search('babre', sounds);

      expect(results[0].matchedIndices).toEqual([]);
    });

    it('should skip levenshtein for names longer than 50 characters', () => {
      const longName = 'a'.repeat(51);
      const sounds = [createMockSound({ id: '1', name: longName })];
      // Query that would match via levenshtein but name is too long
      const results = service.search('aaa', sounds);

      // Should not match because name > 50 chars and 'aaa' is not exact or subsequence of 51 a's
      // Actually 'aaa' is a subsequence of 'aaa...a', so let's use a different query
      const sounds2 = [createMockSound({ id: '1', name: longName })];
      const results2 = service.search('baa', sounds2);

      // 'baa' vs 51 a's - not exact, not subsequence (b not in string), levenshtein skipped
      expect(results2.length).toBe(0);
    });
  });
});
