import { TestBed } from "@angular/core/testing";
import { ImageSearchService } from "./image-search.service";

describe("ImageSearchService", () => {
  let service: ImageSearchService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(ImageSearchService);
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  describe("extractQueryFromFilename", () => {
    it("should extract words from filename", () => {
      expect(service.extractQueryFromFilename("airhorn.mp3")).toBe("airhorn");
    });

    it("should replace underscores with spaces", () => {
      expect(service.extractQueryFromFilename("funny_airhorn_sound.mp3")).toBe(
        "funny airhorn sound",
      );
    });

    it("should replace hyphens with spaces", () => {
      expect(service.extractQueryFromFilename("funny-airhorn-sound.mp3")).toBe(
        "funny airhorn sound",
      );
    });

    it("should limit to 3 words", () => {
      expect(
        service.extractQueryFromFilename("one_two_three_four_five.mp3"),
      ).toBe("one two three");
    });

    it("should convert to lowercase", () => {
      expect(service.extractQueryFromFilename("LOUD_AIRHORN.mp3")).toBe(
        "loud airhorn",
      );
    });
  });

  describe("hasApiKey", () => {
    it("should always return true (DuckDuckGo needs no key)", () => {
      expect(service.hasApiKey()).toBeTrue();
    });
  });

  describe("initial state", () => {
    it("should not be loading initially", () => {
      expect(service.loading()).toBeFalse();
    });

    it("should have no error initially", () => {
      expect(service.error()).toBeNull();
    });
  });
});
