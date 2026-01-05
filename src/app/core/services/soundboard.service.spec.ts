import { TestBed } from '@angular/core/testing';
import { SoundboardService } from './soundboard.service';
import { TauriService } from './tauri.service';
import { PadImage } from '../models';

describe('SoundboardService', () => {
  let service: SoundboardService;
  let tauriServiceSpy: jasmine.SpyObj<TauriService>;

  beforeEach(() => {
    const spy = jasmine.createSpyObj('TauriService', [
      'loadSoundboardState',
      'saveSoundboardState',
      'loadSettings',
      'loadSoundFile',
      'playSound',
      'stopSound',
      'previewSound',
      'stopPreview',
      'setPreviewDevice',
      'listenPreviewStarted',
      'listenPreviewStopped'
    ]);

    // Default mock implementations
    spy.loadSoundboardState.and.returnValue(Promise.resolve([]));
    spy.saveSoundboardState.and.returnValue(Promise.resolve());
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

    it('should initialize pads with default values', () => {
      const pad = service.pads()[0];
      expect(pad.volume).toBe(1.0);
      expect(pad.speed).toBe(1.0);
      expect(pad.isPlaying).toBeFalse();
      expect(pad.customName).toBeUndefined();
    });
  });

  describe('setPadCustomName', () => {
    it('should set custom name on a pad', () => {
      const padId = service.pads()[0].id;

      service.setPadCustomName(padId, 'My Custom Name');

      const updatedPad = service.pads().find(p => p.id === padId);
      expect(updatedPad?.customName).toBe('My Custom Name');
    });

    it('should clear custom name when passed null', () => {
      const padId = service.pads()[0].id;

      service.setPadCustomName(padId, 'My Custom Name');
      service.setPadCustomName(padId, null);

      const updatedPad = service.pads().find(p => p.id === padId);
      expect(updatedPad?.customName).toBeUndefined();
    });

    it('should clear custom name when passed empty string', () => {
      const padId = service.pads()[0].id;

      service.setPadCustomName(padId, 'My Custom Name');
      service.setPadCustomName(padId, '');

      const updatedPad = service.pads().find(p => p.id === padId);
      expect(updatedPad?.customName).toBeUndefined();
    });

    // Note: Testing that saveState is called requires complex async mocking
    // due to the service's initialization flow. The behavior (state persistence)
    // is verified through integration tests. Unit tests focus on state changes.

    it('should not affect other pads when setting custom name', () => {
      const pad0Id = service.pads()[0].id;
      const pad1Id = service.pads()[1].id;

      service.setPadCustomName(pad0Id, 'Name for Pad 0');

      const pad1 = service.pads().find(p => p.id === pad1Id);
      expect(pad1?.customName).toBeUndefined();
    });
  });

  describe('setPadVolume', () => {
    it('should set volume on a pad', () => {
      const padId = service.pads()[0].id;

      service.setPadVolume(padId, 1.5);

      const updatedPad = service.pads().find(p => p.id === padId);
      expect(updatedPad?.volume).toBe(1.5);
    });

    it('should clamp volume to max 2.0', () => {
      const padId = service.pads()[0].id;

      service.setPadVolume(padId, 3.0);

      const updatedPad = service.pads().find(p => p.id === padId);
      expect(updatedPad?.volume).toBe(2.0);
    });

    it('should clamp volume to min 0', () => {
      const padId = service.pads()[0].id;

      service.setPadVolume(padId, -0.5);

      const updatedPad = service.pads().find(p => p.id === padId);
      expect(updatedPad?.volume).toBe(0);
    });
  });

  describe('setPadSpeed', () => {
    it('should set speed on a pad', () => {
      const padId = service.pads()[0].id;

      service.setPadSpeed(padId, 1.5);

      const updatedPad = service.pads().find(p => p.id === padId);
      expect(updatedPad?.speed).toBe(1.5);
    });

    it('should clamp speed to max 2.0', () => {
      const padId = service.pads()[0].id;

      service.setPadSpeed(padId, 3.0);

      const updatedPad = service.pads().find(p => p.id === padId);
      expect(updatedPad?.speed).toBe(2.0);
    });

    it('should clamp speed to min 0.5', () => {
      const padId = service.pads()[0].id;

      service.setPadSpeed(padId, 0.1);

      const updatedPad = service.pads().find(p => p.id === padId);
      expect(updatedPad?.speed).toBe(0.5);
    });
  });

  describe('addPads', () => {
    it('should add 4 pads by default', () => {
      const initialCount = service.pads().length;

      service.addPads();

      expect(service.pads().length).toBe(initialCount + 4);
    });

    it('should add specified number of pads', () => {
      const initialCount = service.pads().length;

      service.addPads(8);

      expect(service.pads().length).toBe(initialCount + 8);
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

  describe('setPadImage', () => {
    it('should set image on pad', () => {
      const image: PadImage = { localPath: 'pad-0-abc123.jpg' };
      service.setPadImage('pad-0', image);
      expect(service.pads()[0].image).toEqual(image);
    });

    it('should clear image when set to null', () => {
      const image: PadImage = { localPath: 'pad-0-abc123.jpg' };
      service.setPadImage('pad-0', image);
      service.setPadImage('pad-0', null);
      expect(service.pads()[0].image).toBeUndefined();
    });
  });
});
