# Virtual Audio Device Setup Guide

This guide explains how to set up and use virtual audio devices with Voiceboard.

## What is a Virtual Audio Device?

A virtual audio device is a software-based audio driver that creates virtual audio inputs and outputs. These appear in Windows as regular audio devices, but instead of connecting to physical hardware, they route audio between applications.

For Voiceboard, we need a virtual audio device to:
1. Receive mixed audio (mic + sound effects) from Voiceboard
2. Appear as a microphone input in other apps (Discord, Zoom, etc.)

## Recommended Solutions

### Option 1: VB-Audio Virtual Cable (Recommended)

**Pros**:
- Free
- Simple and lightweight
- Stable and reliable
- Low latency

**Cons**:
- Limited to one virtual cable (free version)
- Basic functionality only

**Download**: https://vb-audio.com/Cable/

### Option 2: VoiceMeeter

**Pros**:
- Free
- Advanced mixing capabilities
- Multiple virtual inputs/outputs
- Built-in effects

**Cons**:
- More complex to set up
- Higher CPU usage
- Steeper learning curve

**Download**: https://vb-audio.com/Voicemeeter/

### Option 3: Virtual Audio Cable (VAC)

**Pros**:
- Very stable
- Professional grade
- Multiple virtual cables

**Cons**:
- Paid software (~$25)
- Overkill for this use case

**Download**: https://vac.muzychenko.net/

## Installation

### Installing VB-Audio Virtual Cable

1. **Download** the installer from: https://vb-audio.com/Cable/

2. **Extract** the ZIP file

3. **Run installer as Administrator**:
   - Right-click `VBCABLE_Setup_x64.exe` (or x86 for 32-bit)
   - Select "Run as administrator"

4. **Follow installation wizard**:
   - Click "Install Driver"
   - Accept the license
   - Wait for installation to complete

5. **Restart your computer** (important!)

6. **Verify installation**:
   - Open Windows Sound settings
   - Check "Playback" devices for "CABLE Input"
   - Check "Recording" devices for "CABLE Output"

### Installing VoiceMeeter

1. **Download** from: https://vb-audio.com/Voicemeeter/

2. **Run installer** as Administrator

3. **Select version**:
   - VoiceMeeter: Basic version
   - VoiceMeeter Banana: More inputs/outputs (recommended)
   - VoiceMeeter Potato: Professional version

4. **Install and restart**

5. **Launch VoiceMeeter** after restart

6. **Configure** (see VoiceMeeter setup section below)

## Configuration

### Windows Sound Settings

After installing a virtual cable, configure Windows:

1. **Open Sound Settings**:
   - Right-click speaker icon in taskbar
   - Select "Sound settings" or "Open Sound settings"

2. **Set your default devices**:
   - **Playback**: Your normal speakers/headphones
   - **Recording**: Your normal microphone

   Do NOT set the virtual cable as default (Voiceboard will use it programmatically)

### VB-Audio Cable Setup

**For Voiceboard**:

The virtual cable creates two devices:
- **CABLE Input** (Playback device): Where Voiceboard writes audio
- **CABLE Output** (Recording device): Where other apps read audio

Voiceboard will automatically detect and use CABLE Input as the output device.

**For Other Applications** (Discord, Zoom, etc.):

1. Open the application's audio settings

2. **Set Input Device** to: "CABLE Output" (VB-Audio Virtual Cable)

3. Example for Discord:
   - User Settings → Voice & Video
   - Input Device → Select "CABLE Output (VB-Audio Virtual Cable)"

### VoiceMeeter Setup

VoiceMeeter is more complex but offers more control.

**Basic Configuration**:

1. **Launch VoiceMeeter**

2. **Configure Hardware Inputs**:
   - Click "Hardware Input 1"
   - Select your real microphone

3. **Configure Hardware Out**:
   - Click "A1"
   - Select your speakers/headphones

4. **Configure Virtual Input**:
   - This is where Voiceboard will send audio

5. **Route audio**:
   - Enable "A1" for inputs you want to hear
   - Enable "B1" for inputs you want to send to apps

**Voiceboard Configuration**:

In Voiceboard:
- Select "VoiceMeeter Input" as the output device

In other apps (Discord, etc.):
- Select "VoiceMeeter Output" as the microphone

## Voiceboard Integration

### Automatic Detection

Voiceboard automatically searches for virtual audio cables:

```rust
fn find_virtual_cable(host: &Host) -> Option<Device> {
    let devices = host.output_devices().ok()?;
    for device in devices {
        let name = device.name().ok()?;
        let name_lower = name.to_lowercase();
        if name_lower.contains("cable") ||
           name_lower.contains("voicemeeter") {
            return Some(device);
        }
    }
    None
}
```

### Manual Selection

Users can also manually select the output device via the UI:

1. Open Voiceboard settings
2. Go to "Audio Devices"
3. Select output device from dropdown
4. Choose "CABLE Input" or "VoiceMeeter Input"

### Testing the Connection

**Test in Voiceboard**:

1. Start Voiceboard
2. Enable your microphone
3. Play a sound effect
4. You should see audio levels for both mic and output

**Test in Discord** (or other app):

1. Open Discord voice settings
2. Select "CABLE Output" as input device
3. Do a mic test
4. Speak into your real microphone while Voiceboard is running
5. You should hear yourself
6. Play a sound in Voiceboard
7. You should hear the sound

## Audio Routing Diagrams

### Simple Setup (VB-Cable)

```
Real Microphone
      │
      ▼
  Voiceboard ───► Mix (Mic + Sounds)
                      │
                      ▼
                 CABLE Input (Playback)
                      │
                      ▼
                 CABLE Output (Recording)
                      │
                      ▼
              Discord/Zoom/OBS
```

### Advanced Setup (VoiceMeeter)

```
Real Microphone ──► VoiceMeeter Input 1
                         │
    Voiceboard ─────► VoiceMeeter Virtual Input
        (Sounds)         │
                         ▼
                    VoiceMeeter Mixer
                         │
                    ┌────┴────┐
                    ▼         ▼
              Speakers   VoiceMeeter Output
                           │
                           ▼
                    Discord/Zoom/OBS
```

## Troubleshooting

### Issue: Virtual Cable Not Detected

**Symptoms**: Voiceboard doesn't find virtual cable

**Solutions**:
1. Verify installation in Windows Sound settings
2. Restart Voiceboard
3. Restart computer
4. Reinstall virtual cable driver
5. Manually select device in Voiceboard settings

### Issue: No Audio in Discord/Zoom

**Symptoms**: Others can't hear you in Discord/Zoom

**Checklist**:
- [ ] Virtual cable installed and appears in Windows Sound
- [ ] Voiceboard is running
- [ ] Audio engine started in Voiceboard
- [ ] Discord/Zoom input set to "CABLE Output"
- [ ] Discord/Zoom input volume not muted
- [ ] Windows Sound → Recording → CABLE Output not muted
- [ ] Test with Voiceboard's mic test feature

### Issue: Echo or Feedback

**Symptoms**: Hearing yourself loop back

**Causes**:
- Audio routing loop
- Both real mic and virtual cable enabled

**Solutions**:
1. In Discord: Disable "Listen to this device" for CABLE Output
2. In Windows Sound: Disable "Listen to this device" for CABLE Output
3. Ensure Discord input is ONLY CABLE Output, not your real mic

### Issue: Distorted or Crackling Audio

**Symptoms**: Audio sounds distorted through virtual cable

**Solutions**:
1. Reduce master volume in Voiceboard
2. Reduce effects volume in Voiceboard
3. Check Windows volume mixer (don't set CABLE to max)
4. Increase buffer size in Voiceboard settings
5. Update virtual cable driver

### Issue: High Latency

**Symptoms**: Noticeable delay between speaking and hearing in Discord

**Measurements**:
- Normal: 20-50ms (imperceptible)
- Noticeable: 100-200ms
- Bad: >200ms

**Solutions**:
1. Reduce buffer size in Voiceboard
2. Use VB-Cable instead of VoiceMeeter (lower latency)
3. Close other audio applications
4. Update audio drivers
5. Use wired headset instead of Bluetooth

### Issue: Virtual Cable Doesn't Appear After Restart

**Symptoms**: Virtual cable missing after computer restart

**Solutions**:
1. Reinstall virtual cable driver
2. Run installation as Administrator
3. Check Windows Sound → Show Disabled Devices
4. Enable "CABLE Input" and "CABLE Output" if disabled
5. Set CABLE as default communication device (then reset to your preference)

## Advanced Configuration

### Buffer Size Tuning

For optimal latency with virtual cables:

1. **In Voiceboard** (config.json):
```json
{
  "audio": {
    "buffer_size": 480,  // 10ms @ 48kHz
    "sample_rate": 48000
  }
}
```

2. **In VB-Cable**:
   - Open "VB-Cable Control Panel"
   - Set buffer size: 128-512 samples
   - Lower = less latency, more CPU

### Sample Rate Matching

Ensure all devices use the same sample rate:

1. **Windows Sound Settings**:
   - Right-click CABLE Input → Properties
   - Advanced tab
   - Set to "2 channel, 24 bit, 48000 Hz"

2. **In Voiceboard**:
   - Already configured for 48000 Hz

3. **In Discord/Zoom**:
   - Usually auto-detects and matches

### Multiple Virtual Cables

For advanced setups (requires VoiceMeeter Banana/Potato):

1. **Scenario**: Separate music player + Voiceboard

2. **Setup**:
   - VoiceMeeter Input 1: Real microphone
   - VoiceMeeter Input 2: Music player (e.g., Spotify)
   - VoiceMeeter Input 3: Voiceboard sound effects
   - Output: Mix all to Discord

3. **Configuration**:
   - Each input has individual volume control
   - Can mute music while speaking (ducking)

## Performance Impact

### CPU Usage

- **VB-Cable**: ~0.5% CPU
- **VoiceMeeter**: ~2-5% CPU
- **Voiceboard**: ~5-10% CPU
- **Total**: ~10-15% CPU

### Memory Usage

- **VB-Cable**: ~5 MB
- **VoiceMeeter**: ~20 MB
- **Voiceboard**: ~50 MB
- **Total**: ~75 MB

### Latency Measurements

| Setup | Latency |
|-------|---------|
| VB-Cable + Voiceboard | ~20ms |
| VoiceMeeter + Voiceboard | ~30ms |
| Direct microphone | ~5ms |

## Alternative Approaches

### Option: Windows Audio Loopback API

**Status**: Future enhancement

**Concept**: Use Windows WASAPI loopback mode instead of virtual cable

**Pros**:
- No external software needed
- Lower latency
- More control

**Cons**:
- Complex to implement
- Requires low-level Windows API knowledge
- May need driver signing

### Option: Custom Virtual Audio Driver

**Status**: Advanced / future

**Concept**: Write a custom WDM audio driver

**Pros**:
- Complete control
- No dependencies
- Best performance

**Cons**:
- Very complex
- Requires kernel-mode development
- Driver signing required for Windows
- Maintenance burden

## Resources

### VB-Audio Documentation
- Virtual Cable: https://vb-audio.com/Cable/index.htm
- VoiceMeeter: https://vb-audio.com/Voicemeeter/index.htm
- Forum: https://forum.vb-audio.com/

### Windows Audio
- WASAPI: https://docs.microsoft.com/en-us/windows/win32/coreaudio/wasapi
- Audio Drivers: https://docs.microsoft.com/en-us/windows-hardware/drivers/audio/

### Testing Tools
- Audacity: Record from CABLE Output to verify audio
- VoiceMeeter's built-in meter bridge
- Windows Sound Recorder

## Recommendations

**For Most Users**: VB-Audio Virtual Cable
- Simple, free, works well
- Minimal setup
- Low latency

**For Power Users**: VoiceMeeter Banana
- More control
- Multiple virtual devices
- Built-in mixing and effects

**For Developers/Testing**: Both
- Test with VB-Cable for simplicity
- Test with VoiceMeeter for compatibility

---

**Next Steps**: After setting up your virtual audio device, return to the main README for usage instructions.
