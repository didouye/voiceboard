import { ComponentFixture, TestBed } from '@angular/core/testing';
import { SoundPadComponent } from './sound-pad.component';
import { SoundboardService } from '../../../core/services/soundboard.service';
import { ShortcutService } from '../../../core/services/shortcut.service';
import { SoundPad, Sound } from '../../../core/models';

describe('SoundPadComponent', () => {
  let component: SoundPadComponent;
  let fixture: ComponentFixture<SoundPadComponent>;
  let soundboardServiceSpy: jasmine.SpyObj<SoundboardService>;
  let shortcutServiceSpy: jasmine.SpyObj<ShortcutService>;

  const createMockSound = (overrides: Partial<Sound> = {}): Sound => ({
    id: 'hash_abc123',
    name: 'test-sound.mp3',
    path: '/path/to/test-sound.mp3',
    duration: 5.5,
    volume: 1.0,
    speed: 1.0,
    folderIds: [],
    isPlaying: false,
    addedAt: Date.now(),
    ...overrides
  });

  const createMockPad = (overrides: Partial<SoundPad> = {}): SoundPad => ({
    index: 0,
    sound: null,
    color: '#e74c3c',
    ...overrides
  });

  beforeEach(async () => {
    const soundboardSpy = jasmine.createSpyObj('SoundboardService', [
      'formatDuration',
      'pads'
    ]);
    soundboardSpy.formatDuration.and.callFake((seconds: number) => {
      const mins = Math.floor(seconds / 60);
      const secs = Math.floor(seconds % 60);
      return `${mins}:${secs.toString().padStart(2, '0')}`;
    });
    soundboardSpy.pads.and.returnValue([]);

    const shortcutSpy = jasmine.createSpyObj('ShortcutService', [
      'formatEventAsShortcut',
      'checkConflict'
    ]);

    await TestBed.configureTestingModule({
      imports: [SoundPadComponent],
      providers: [
        { provide: SoundboardService, useValue: soundboardSpy },
        { provide: ShortcutService, useValue: shortcutSpy }
      ]
    }).compileComponents();

    fixture = TestBed.createComponent(SoundPadComponent);
    component = fixture.componentInstance;
    soundboardServiceSpy = TestBed.inject(SoundboardService) as jasmine.SpyObj<SoundboardService>;
    shortcutServiceSpy = TestBed.inject(ShortcutService) as jasmine.SpyObj<ShortcutService>;
  });

  describe('display', () => {
    it('should display filename when no custom name', () => {
      component.pad = createMockPad({ sound: createMockSound() });
      fixture.detectChanges();

      const compiled = fixture.nativeElement as HTMLElement;
      const nameElement = compiled.querySelector('.text-xs.font-semibold');

      expect(nameElement?.textContent?.trim()).toBe('test-sound.mp3');
    });

    it('should display custom name when set', () => {
      component.pad = createMockPad({
        sound: createMockSound({ customName: 'My Custom Sound' })
      });
      fixture.detectChanges();

      const compiled = fixture.nativeElement as HTMLElement;
      const nameElement = compiled.querySelector('.text-xs.font-semibold');

      expect(nameElement?.textContent?.trim()).toBe('My Custom Sound');
    });

    it('should display original filename below custom name', () => {
      component.pad = createMockPad({
        sound: createMockSound({ customName: 'My Custom Sound' })
      });
      fixture.detectChanges();

      const compiled = fixture.nativeElement as HTMLElement;
      const filenameElement = compiled.querySelector('.text-\\[9px\\]');

      expect(filenameElement?.textContent?.trim()).toBe('test-sound.mp3');
    });

    it('should not display filename below when no custom name', () => {
      component.pad = createMockPad({ sound: createMockSound() });
      fixture.detectChanges();

      const compiled = fixture.nativeElement as HTMLElement;
      const filenameElement = compiled.querySelector('.text-\\[9px\\]');

      expect(filenameElement).toBeNull();
    });

    it('should display duration', () => {
      component.pad = createMockPad({ sound: createMockSound() });
      fixture.detectChanges();

      const compiled = fixture.nativeElement as HTMLElement;
      const durationElement = compiled.querySelector('.text-\\[10px\\].text-white\\/70');

      expect(durationElement?.textContent?.trim()).toBe('0:05');
    });

    it('should display import prompt when no sound', () => {
      component.pad = createMockPad({ sound: null });
      fixture.detectChanges();

      const compiled = fixture.nativeElement as HTMLElement;
      expect(compiled.textContent).toContain('Import');
    });
  });

  describe('events', () => {
    it('should emit customNameChange when name changes', () => {
      component.pad = createMockPad({ sound: createMockSound() });
      fixture.detectChanges();

      spyOn(component.customNameChange, 'emit');

      component.onCustomNameChange('New Name');

      expect(component.customNameChange.emit).toHaveBeenCalledWith('New Name');
    });

    it('should emit null when name is empty', () => {
      component.pad = createMockPad({ sound: createMockSound() });
      fixture.detectChanges();

      spyOn(component.customNameChange, 'emit');

      component.onCustomNameChange('');

      expect(component.customNameChange.emit).toHaveBeenCalledWith(null);
    });

    it('should emit null when name is whitespace only', () => {
      component.pad = createMockPad({ sound: createMockSound() });
      fixture.detectChanges();

      spyOn(component.customNameChange, 'emit');

      component.onCustomNameChange('   ');

      expect(component.customNameChange.emit).toHaveBeenCalledWith(null);
    });

    it('should emit trimmed name', () => {
      component.pad = createMockPad({ sound: createMockSound() });
      fixture.detectChanges();

      spyOn(component.customNameChange, 'emit');

      component.onCustomNameChange('  Trimmed Name  ');

      expect(component.customNameChange.emit).toHaveBeenCalledWith('Trimmed Name');
    });

    it('should emit play when clicked with sound', () => {
      component.pad = createMockPad({ sound: createMockSound() });
      fixture.detectChanges();

      spyOn(component.play, 'emit');

      component.onClick(new MouseEvent('click'));

      expect(component.play.emit).toHaveBeenCalled();
    });

    it('should emit import when clicked without sound', () => {
      component.pad = createMockPad({ sound: null });
      fixture.detectChanges();

      spyOn(component.import, 'emit');

      component.onClick(new MouseEvent('click'));

      expect(component.import.emit).toHaveBeenCalled();
    });

    it('should emit volumeChange when volume changes', () => {
      component.pad = createMockPad({ sound: createMockSound() });
      fixture.detectChanges();

      spyOn(component.volumeChange, 'emit');

      component.onVolumeChange(1.5);

      expect(component.volumeChange.emit).toHaveBeenCalledWith(1.5);
    });

    it('should emit speedChange when speed changes', () => {
      component.pad = createMockPad({ sound: createMockSound() });
      fixture.detectChanges();

      spyOn(component.speedChange, 'emit');

      component.onSpeedChange(2.0);

      expect(component.speedChange.emit).toHaveBeenCalledWith(2.0);
    });
  });

  describe('resetAll', () => {
    it('should emit reset values for volume, speed and customName', () => {
      component.pad = createMockPad({
        sound: createMockSound({
          customName: 'Custom',
          volume: 1.5,
          speed: 2.0
        })
      });
      fixture.detectChanges();

      spyOn(component.volumeChange, 'emit');
      spyOn(component.speedChange, 'emit');
      spyOn(component.customNameChange, 'emit');

      component.resetAll();

      expect(component.volumeChange.emit).toHaveBeenCalledWith(1.0);
      expect(component.speedChange.emit).toHaveBeenCalledWith(1.0);
      expect(component.customNameChange.emit).toHaveBeenCalledWith(null);
    });
  });

  describe('formatDuration', () => {
    it('should delegate to soundboardService', () => {
      component.pad = createMockPad();
      fixture.detectChanges();

      const result = component.formatDuration(90);

      expect(soundboardServiceSpy.formatDuration).toHaveBeenCalledWith(90);
      expect(result).toBe('1:30');
    });
  });
});
