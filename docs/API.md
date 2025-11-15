# Voiceboard API Reference

This document describes all API interfaces between the Angular frontend and Rust backend via Tauri IPC.

## Table of Contents

- [Soundboard Commands](#soundboard-commands)
- [Device Commands](#device-commands)
- [Audio Commands](#audio-commands)
- [Events](#events)
- [Data Types](#data-types)

## Soundboard Commands

### `get_sounds`

Get all sounds from the soundboard.

**Parameters:** None

**Returns:** `Sound[]`

**Example:**
```typescript
const sounds = await invoke<Sound[]>('get_sounds');
```

---

### `add_sound`

Add a new sound to the soundboard.

**Parameters:**
- `name: string` - Display name for the sound
- `filePath: string` - Absolute path to the audio file

**Returns:** `Sound`

**Example:**
```typescript
const sound = await invoke<Sound>('add_sound', {
  name: 'My Sound',
  filePath: 'C:\\sounds\\mysound.mp3'
});
```

---

### `delete_sound`

Delete a sound from the soundboard.

**Parameters:**
- `id: string` - Sound ID

**Returns:** `void`

**Example:**
```typescript
await invoke('delete_sound', { id: 'sound-id-123' });
```

---

### `rename_sound`

Rename a sound.

**Parameters:**
- `id: string` - Sound ID
- `name: string` - New name

**Returns:** `void`

**Example:**
```typescript
await invoke('rename_sound', {
  id: 'sound-id-123',
  name: 'New Name'
});
```

---

### `update_sound_volume`

Update the volume of a sound.

**Parameters:**
- `id: string` - Sound ID
- `volume: number` - Volume (0.0 to 1.0)

**Returns:** `void`

**Example:**
```typescript
await invoke('update_sound_volume', {
  id: 'sound-id-123',
  volume: 0.75
});
```

---

### `reorder_sounds`

Reorder sounds by providing a new order of IDs.

**Parameters:**
- `ids: string[]` - Array of sound IDs in the desired order

**Returns:** `void`

**Example:**
```typescript
await invoke('reorder_sounds', {
  ids: ['sound-1', 'sound-2', 'sound-3']
});
```

---

### `filter_sounds`

Filter sounds by name query.

**Parameters:**
- `query: string` - Search query

**Returns:** `Sound[]`

**Example:**
```typescript
const filtered = await invoke<Sound[]>('filter_sounds', {
  query: 'explosion'
});
```

---

### `get_sound_count`

Get the total count of sounds.

**Parameters:** None

**Returns:** `number`

**Example:**
```typescript
const count = await invoke<number>('get_sound_count');
```

---

## Device Commands

### `get_input_devices`

Get all available input devices (microphones).

**Parameters:** None

**Returns:** `AudioDevice[]`

**Example:**
```typescript
const devices = await invoke<AudioDevice[]>('get_input_devices');
```

---

### `get_output_devices`

Get all available output devices.

**Parameters:** None

**Returns:** `AudioDevice[]`

**Example:**
```typescript
const devices = await invoke<AudioDevice[]>('get_output_devices');
```

---

### `get_default_input_device`

Get the default input device.

**Parameters:** None

**Returns:** `AudioDevice | null`

**Example:**
```typescript
const device = await invoke<AudioDevice | null>('get_default_input_device');
```

---

### `get_default_output_device`

Get the default output device.

**Parameters:** None

**Returns:** `AudioDevice | null`

**Example:**
```typescript
const device = await invoke<AudioDevice | null>('get_default_output_device');
```

---

### `select_input_device`

Select an input device to use for microphone capture.

**Parameters:**
- `deviceId: string` - Device ID

**Returns:** `void`

**Example:**
```typescript
await invoke('select_input_device', {
  deviceId: 'input_0'
});
```

---

### `select_output_device`

Select an output device (virtual cable) for audio output.

**Parameters:**
- `deviceId: string` - Device ID

**Returns:** `void`

**Example:**
```typescript
await invoke('select_output_device', {
  deviceId: 'output_1'
});
```

---

## Audio Commands

### `start_audio_engine`

Start the audio engine.

**Parameters:** None

**Returns:** `void`

**Example:**
```typescript
await invoke('start_audio_engine');
```

---

### `stop_audio_engine`

Stop the audio engine.

**Parameters:** None

**Returns:** `void`

**Example:**
```typescript
await invoke('stop_audio_engine');
```

---

### `play_sound`

Play a sound.

**Parameters:**
- `id: string` - Sound ID
- `filePath: string` - Path to the audio file

**Returns:** `void`

**Example:**
```typescript
await invoke('play_sound', {
  id: 'sound-123',
  filePath: 'C:\\sounds\\effect.mp3'
});
```

---

### `stop_sound`

Stop a currently playing sound.

**Parameters:**
- `id: string` - Sound ID

**Returns:** `void`

**Example:**
```typescript
await invoke('stop_sound', { id: 'sound-123' });
```

---

### `stop_all_sounds`

Stop all currently playing sounds.

**Parameters:** None

**Returns:** `void`

**Example:**
```typescript
await invoke('stop_all_sounds');
```

---

### `set_master_volume`

Set the master output volume.

**Parameters:**
- `volume: number` - Volume (0.0 to 1.0)

**Returns:** `void`

**Example:**
```typescript
await invoke('set_master_volume', { volume: 0.8 });
```

---

### `set_mic_volume`

Set the microphone input volume.

**Parameters:**
- `volume: number` - Volume (0.0 to 1.0)

**Returns:** `void`

**Example:**
```typescript
await invoke('set_mic_volume', { volume: 0.5 });
```

---

### `set_effects_volume`

Set the sound effects volume.

**Parameters:**
- `volume: number` - Volume (0.0 to 1.0)

**Returns:** `void`

**Example:**
```typescript
await invoke('set_effects_volume', { volume: 0.7 });
```

---

## Events

Events are emitted from Rust to Angular using Tauri's event system.

### `audio-level`

Emitted periodically with audio level information for visualization.

**Payload:**
```typescript
{
  mic_level: AudioLevel;
  output_level: AudioLevel;
}
```

**Example:**
```typescript
const unlisten = await listen<AudioLevelEvent>('audio-level', (event) => {
  console.log('Mic level:', event.payload.mic_level);
  console.log('Output level:', event.payload.output_level);
});

// Later: unlisten();
```

---

### `sound-playback-started`

Emitted when a sound starts playing.

**Payload:**
```typescript
{
  soundId: string;
}
```

**Example:**
```typescript
const unlisten = await listen('sound-playback-started', (event) => {
  console.log('Sound started:', event.payload.soundId);
});
```

---

### `sound-playback-stopped`

Emitted when a sound stops playing.

**Payload:**
```typescript
{
  soundId: string;
}
```

**Example:**
```typescript
const unlisten = await listen('sound-playback-stopped', (event) => {
  console.log('Sound stopped:', event.payload.soundId);
});
```

---

### `device-changed`

Emitted when the audio device configuration changes.

**Payload:**
```typescript
{
  deviceId: string;
}
```

**Example:**
```typescript
const unlisten = await listen('device-changed', (event) => {
  console.log('Device changed:', event.payload.deviceId);
});
```

---

### `error`

Emitted when an error occurs in the audio engine.

**Payload:**
```typescript
{
  message: string;
  details: string;
}
```

**Example:**
```typescript
const unlisten = await listen('error', (event) => {
  console.error('Audio error:', event.payload);
});
```

---

## Data Types

### `Sound`

```typescript
interface Sound {
  id: string;
  name: string;
  file_path: string;
  volume: number;        // 0.0 to 1.0
  sort_order: number;
  created_at: string;    // ISO 8601 timestamp
  updated_at: string;    // ISO 8601 timestamp
}
```

---

### `AudioDevice`

```typescript
interface AudioDevice {
  id: string;
  name: string;
  device_type: 'input' | 'output';
  is_default: boolean;
}
```

---

### `AudioLevel`

```typescript
interface AudioLevel {
  rms_db: number;         // RMS level in decibels (-60 to 0)
  peak_db: number;        // Peak level in decibels (-60 to 0)
  rms_linear: number;     // RMS level linear (0.0 to 1.0)
  peak_linear: number;    // Peak level linear (0.0 to 1.0)
}
```

---

### `AudioLevelEvent`

```typescript
interface AudioLevelEvent {
  mic_level: AudioLevel;
  output_level: AudioLevel;
}
```

---

## Error Handling

All Tauri commands can throw errors. Always wrap calls in try-catch blocks:

```typescript
try {
  await invoke('play_sound', { id: soundId, filePath });
} catch (error) {
  console.error('Failed to play sound:', error);
  // Handle error appropriately
}
```

Errors from Rust are serialized as strings containing the error message.

---

## Usage Examples

### Complete Sound Playback Flow

```typescript
// 1. Load sounds
const sounds = await invoke<Sound[]>('get_sounds');

// 2. Play a sound
const sound = sounds[0];
await invoke('play_sound', {
  id: sound.id,
  filePath: sound.file_path
});

// 3. Listen for playback events
const unlisten = await listen('sound-playback-started', (event) => {
  console.log('Playing:', event.payload.soundId);
});

// 4. Later: stop the sound
await invoke('stop_sound', { id: sound.id });
```

### Device Selection Flow

```typescript
// 1. Get available input devices
const devices = await invoke<AudioDevice[]>('get_input_devices');

// 2. Display to user (UI selection)
// ...

// 3. Select a device
await invoke('select_input_device', {
  deviceId: selectedDevice.id
});

// 4. Listen for device changes
const unlisten = await listen('device-changed', (event) => {
  console.log('Device changed to:', event.payload.deviceId);
});
```

### Volume Control Flow

```typescript
// Set all volume levels
await Promise.all([
  invoke('set_master_volume', { volume: 0.8 }),
  invoke('set_mic_volume', { volume: 0.5 }),
  invoke('set_effects_volume', { volume: 0.7 })
]);

// Listen for audio levels
const unlisten = await listen<AudioLevelEvent>('audio-level', (event) => {
  updateVisualization(event.payload);
});
```

---

## Best Practices

1. **Always handle errors**: Wrap Tauri commands in try-catch blocks
2. **Cleanup event listeners**: Call `unlisten()` when components are destroyed
3. **Type safety**: Use TypeScript interfaces for all data structures
4. **Debounce volume changes**: Avoid sending too many volume update commands
5. **Cache sound list**: Use a service to maintain local state and reduce backend calls
6. **Validate inputs**: Check file paths and IDs before sending to backend
7. **Handle offline state**: Check if Tauri context exists before making calls

---

## Performance Considerations

- **Sound loading**: Sounds are decoded on first play, which may cause a small delay
- **Event frequency**: Audio level events are emitted frequently (10-60Hz), use throttling in UI
- **Concurrent playback**: The mixer can handle multiple sounds, but too many may affect performance
- **File size**: Keep sound files under 10MB for optimal load times
- **Device enumeration**: Cache device lists and only refresh when needed

---

## Debugging

Enable debug logging by setting the `RUST_LOG` environment variable:

```
RUST_LOG=voiceboard=debug,tauri=info
```

Check console for detailed logs from both Rust and Angular.
