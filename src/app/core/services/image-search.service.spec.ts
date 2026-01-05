import { TestBed } from '@angular/core/testing';
import { ImageSearchService } from './image-search.service';

describe('ImageSearchService', () => {
  let service: ImageSearchService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(ImageSearchService);
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  describe('extractQueryFromFilename', () => {
    it('should extract words from filename', () => {
      expect(service.extractQueryFromFilename('airhorn.mp3')).toBe('airhorn');
    });

    it('should replace underscores with spaces', () => {
      expect(service.extractQueryFromFilename('funny_airhorn_sound.mp3')).toBe('funny airhorn sound');
    });

    it('should replace hyphens with spaces', () => {
      expect(service.extractQueryFromFilename('funny-airhorn-sound.mp3')).toBe('funny airhorn sound');
    });

    it('should limit to 3 words', () => {
      expect(service.extractQueryFromFilename('one_two_three_four_five.mp3')).toBe('one two three');
    });

    it('should convert to lowercase', () => {
      expect(service.extractQueryFromFilename('LOUD_AIRHORN.mp3')).toBe('loud airhorn');
    });
  });

  describe('API key management', () => {
    it('should start with no API key', () => {
      expect(service.hasApiKey()).toBeFalse();
    });

    it('should save API key to localStorage', () => {
      service.setApiKey('test-key');
      expect(localStorage.getItem('pexels_api_key')).toBe('test-key');
      expect(service.hasApiKey()).toBeTrue();
    });

    it('should remove API key when set to null', () => {
      service.setApiKey('test-key');
      service.setApiKey(null);
      expect(localStorage.getItem('pexels_api_key')).toBeNull();
      expect(service.hasApiKey()).toBeFalse();
    });

    it('should load API key from localStorage on construction', () => {
      localStorage.setItem('pexels_api_key', 'stored-key');
      const newService = new ImageSearchService();
      expect(newService.apiKey()).toBe('stored-key');
    });
  });
});
