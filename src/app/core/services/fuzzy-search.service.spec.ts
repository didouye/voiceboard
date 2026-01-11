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
});
