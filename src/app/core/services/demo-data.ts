import { AudioDevice, AppSettings, MixerConfig, SoundFile } from '../models';

/**
 * Mock data for demo mode - used for screenshots and testing
 */

export const DEMO_INPUT_DEVICES: AudioDevice[] = [
  { id: 'mic-1', name: 'Blue Yeti X', deviceType: 'input', isDefault: true, isVirtual: false },
  { id: 'mic-2', name: 'MacBook Pro Microphone', deviceType: 'input', isDefault: false, isVirtual: false },
];

export const DEMO_OUTPUT_DEVICES: AudioDevice[] = [
  { id: 'out-1', name: 'VB-Cable', deviceType: 'output', isDefault: false, isVirtual: true },
  { id: 'out-2', name: 'BlackHole 2ch', deviceType: 'output', isDefault: false, isVirtual: true },
];

export const DEMO_PREVIEW_DEVICES: AudioDevice[] = [
  { id: 'preview-1', name: 'MacBook Pro Speakers', deviceType: 'output', isDefault: true, isVirtual: false },
  { id: 'preview-2', name: 'AirPods Pro', deviceType: 'output', isDefault: false, isVirtual: false },
];

export const DEMO_SETTINGS: AppSettings = {
  audio: {
    inputDeviceId: 'mic-1',
    outputDeviceId: 'out-1',
    previewDeviceId: 'preview-1',
    masterVolume: 0.8,
    sampleRate: 48000,
    bufferSize: 1024,
    micMonitoring: false,
  },
  startMinimized: false,
  autoStartMixing: true,
};

export const DEMO_MIXER_CONFIG: MixerConfig = {
  masterVolume: 0.8,
  channels: [
    { id: 'mic', name: 'Microphone', channelType: 'Microphone', volume: 1.0, muted: false, solo: false },
  ],
  sampleRate: 48000,
  bufferSize: 1024,
};

export const DEMO_SOUNDBOARD_PADS = [
  { id: 'pad-1', name: 'Air Horn', path: '/sounds/airhorn.mp3', shortcut: '1', volume: 1.0, speed: 1.0 },
  { id: 'pad-2', name: 'Sad Trombone', path: '/sounds/sad-trombone.mp3', shortcut: '2', volume: 0.8, speed: 1.0 },
  { id: 'pad-3', name: 'Applause', path: '/sounds/applause.mp3', shortcut: '3', volume: 1.0, speed: 1.0 },
  { id: 'pad-4', name: 'Drum Roll', path: '/sounds/drumroll.mp3', shortcut: '4', volume: 0.9, speed: 1.0 },
  { id: 'pad-5', name: 'MLG Horn', path: '/sounds/mlg.mp3', shortcut: '5', volume: 1.0, speed: 1.0 },
  { id: 'pad-6', name: 'Bruh', path: '/sounds/bruh.mp3', shortcut: '6', volume: 0.7, speed: 1.0 },
  { id: 'pad-7', name: 'Ba Dum Tss', path: '/sounds/badumtss.mp3', shortcut: '7', volume: 1.0, speed: 1.0 },
  { id: 'pad-8', name: 'Victory', path: '/sounds/victory.mp3', shortcut: '8', volume: 1.0, speed: 1.0 },
  { id: 'pad-9', name: 'Fail', path: '/sounds/fail.mp3', shortcut: '9', volume: 0.8, speed: 1.0 },
  { id: 'pad-10', name: 'Laugh Track', path: '/sounds/laugh.mp3', shortcut: '0', volume: 1.0, speed: 1.0 },
  { id: 'pad-11', name: 'Suspense', path: '/sounds/suspense.mp3', shortcut: '-', volume: 0.9, speed: 1.0 },
  { id: 'pad-12', name: 'Ding', path: '/sounds/ding.mp3', shortcut: '=', volume: 1.0, speed: 1.0 },
];

export function createDemoSoundFile(path: string): SoundFile {
  const name = path.split('/').pop()?.replace(/\.[^.]+$/, '') || 'Sound';
  return {
    id: `demo-${Date.now()}`,
    name,
    path,
    duration: 2.5,
    sampleRate: 48000,
    channels: 2,
  };
}
