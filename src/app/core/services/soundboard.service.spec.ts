import { TestBed } from '@angular/core/testing';
import { SoundboardService } from './soundboard.service';
import { TauriService } from './tauri.service';
import { PadImage, Sound } from '../models';
import { signal } from '@angular/core';

describe('SoundboardService', () => {
  let service: SoundboardService;
  let tauriServiceSpy: jasmine.SpyObj<TauriService>;

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
    const spy = jasmine.createSpyObj('TauriService', [
      'loadSoundboardState',
      'saveSoundboardState',
      'loadFolders',
      'saveFolders',
      'loadSettings',
      'loadSoundFile',
      'playSound',
      'stopSound',
      'previewSound',
      'stopPreview',
      'setPreviewDevice',
      'listenPreviewStarted',
      'listenPreviewStopped',
      'importSoundWithHash',
      'importMultipleSoundsWithHash',
      'hashFile'
    ]);

    // Default mock implementations
    spy.loadSoundboardState.and.returnValue(Promise.resolve(null));
    spy.saveSoundboardState.and.returnValue(Promise.resolve());
    spy.loadFolders.and.returnValue(Promise.resolve([]));
    spy.saveFolders.and.returnValue(Promise.resolve());
    spy.loadSettings.and.returnValue(Promise.resolve({
      audio: { previewDeviceId: null }
    }));
    spy.listenPreviewStarted.and.returnValue(Promise.resolve(() => {}));
    spy.listenPreviewStopped.and.returnValue(Promise.resolve(() => {}));

    TestBed.configureTestingModule({
      providers: [
        SoundboardService,
        { provide: TauriService, useValue: spy }
      ]
    });

    service = TestBed.inject(SoundboardService);
    tauriServiceSpy = TestBed.inject(TauriService) as jasmine.SpyObj<TauriService>;
  });

  describe('initialization', () => {
    it('should be created', () => {
      expect(service).toBeTruthy();
    });

    it('should initialize with 12 empty pads', () => {
      expect(service.pads().length).toBe(12);
      expect(service.pads().every(p => p.sound === null)).toBeTrue();
    });

    it('should initialize pads with index and color', () => {
      const pad = service.pads()[0];
      expect(pad.index).toBe(0);
      expect(pad.color).toBeDefined();
      expect(pad.sound).toBeNull();
    });
  });

  describe('setSoundCustomName', () => {
    beforeEach(() => {
      // Add a sound to work with
      const sound = createMockSound({ id: 'sound-1' });
      (service as any)._sounds.set(new Map([['sound-1', sound]]));
    });

    it('should set custom name on a sound', () => {
      service.setSoundCustomName('sound-1', 'My Custom Name');

      const sound = service.getSound('sound-1');
      expect(sound?.customName).toBe('My Custom Name');
    });

    it('should clear custom name when passed null', () => {
      service.setSoundCustomName('sound-1', 'My Custom Name');
      service.setSoundCustomName('sound-1', null);

      const sound = service.getSound('sound-1');
      expect(sound?.customName).toBeUndefined();
    });

    it('should clear custom name when passed empty string', () => {
      service.setSoundCustomName('sound-1', 'My Custom Name');
      service.setSoundCustomName('sound-1', '');

      const sound = service.getSound('sound-1');
      expect(sound?.customName).toBeUndefined();
    });

    it('should not affect other sounds when setting custom name', () => {
      const sound2 = createMockSound({ id: 'sound-2', name: 'sound-2' });
      (service as any)._sounds.update((sounds: Map<string, Sound>) => {
        const updated = new Map(sounds);
        updated.set('sound-2', sound2);
        return updated;
      });

      service.setSoundCustomName('sound-1', 'Name for Sound 1');

      const otherSound = service.getSound('sound-2');
      expect(otherSound?.customName).toBeUndefined();
    });
  });

  describe('setSoundVolume', () => {
    beforeEach(() => {
      const sound = createMockSound({ id: 'sound-1' });
      (service as any)._sounds.set(new Map([['sound-1', sound]]));
    });

    it('should set volume on a sound', () => {
      service.setSoundVolume('sound-1', 1.5);

      const sound = service.getSound('sound-1');
      expect(sound?.volume).toBe(1.5);
    });

    it('should clamp volume to max 2.0', () => {
      service.setSoundVolume('sound-1', 3.0);

      const sound = service.getSound('sound-1');
      expect(sound?.volume).toBe(2.0);
    });

    it('should clamp volume to min 0', () => {
      service.setSoundVolume('sound-1', -0.5);

      const sound = service.getSound('sound-1');
      expect(sound?.volume).toBe(0);
    });
  });

  describe('setSoundSpeed', () => {
    beforeEach(() => {
      const sound = createMockSound({ id: 'sound-1' });
      (service as any)._sounds.set(new Map([['sound-1', sound]]));
    });

    it('should set speed on a sound', () => {
      service.setSoundSpeed('sound-1', 1.5);

      const sound = service.getSound('sound-1');
      expect(sound?.speed).toBe(1.5);
    });

    it('should clamp speed to max 2.0', () => {
      service.setSoundSpeed('sound-1', 3.0);

      const sound = service.getSound('sound-1');
      expect(sound?.speed).toBe(2.0);
    });

    it('should clamp speed to min 0.5', () => {
      service.setSoundSpeed('sound-1', 0.1);

      const sound = service.getSound('sound-1');
      expect(sound?.speed).toBe(0.5);
    });
  });

  describe('virtual pads', () => {
    it('should generate pads with sounds in them', () => {
      const sound = createMockSound({ id: 'sound-1', name: 'Test Sound' });
      (service as any)._sounds.set(new Map([['sound-1', sound]]));

      const pads = service.pads();
      expect(pads.length).toBeGreaterThanOrEqual(12);
      expect(pads[0].sound?.id).toBe('sound-1');
    });

    it('should leave remaining pads empty', () => {
      const sound = createMockSound({ id: 'sound-1' });
      (service as any)._sounds.set(new Map([['sound-1', sound]]));

      const pads = service.pads();
      expect(pads[1].sound).toBeNull();
    });
  });

  describe('formatDuration', () => {
    it('should format seconds as mm:ss', () => {
      expect(service.formatDuration(0)).toBe('0:00');
      expect(service.formatDuration(30)).toBe('0:30');
      expect(service.formatDuration(60)).toBe('1:00');
      expect(service.formatDuration(90)).toBe('1:30');
      expect(service.formatDuration(125)).toBe('2:05');
    });
  });

  describe('setSoundImage', () => {
    beforeEach(() => {
      const sound = createMockSound({ id: 'sound-1' });
      (service as any)._sounds.set(new Map([['sound-1', sound]]));
    });

    it('should set image on sound', () => {
      const image: PadImage = { localPath: 'sound-1-abc123.jpg' };
      service.setSoundImage('sound-1', image);
      expect(service.getSound('sound-1')?.image).toEqual(image);
    });

    it('should clear image when set to null', () => {
      const image: PadImage = { localPath: 'sound-1-abc123.jpg' };
      service.setSoundImage('sound-1', image);
      service.setSoundImage('sound-1', null);
      expect(service.getSound('sound-1')?.image).toBeUndefined();
    });
  });

  describe('removeSound', () => {
    beforeEach(() => {
      const sound = createMockSound({ id: 'sound-1' });
      (service as any)._sounds.set(new Map([['sound-1', sound]]));
    });

    it('should remove sound from store', () => {
      service.removeSound('sound-1');
      expect(service.getSound('sound-1')).toBeUndefined();
    });
  });
});
