# Virtual Audio Device Setup Guide

This guide explains how to set up virtual audio devices on Windows to route VoiceBoard's output to other applications (Discord, Zoom, OBS, games, etc.).

## Why Do You Need a Virtual Audio Device?

VoiceBoard processes your microphone input and outputs modified audio. To use this in other applications, you need a "virtual audio cable" that acts as a bridge between VoiceBoard and other software.

```
Physical Mic → VoiceBoard → Virtual Cable → Other Apps
```

## Recommended Solutions

### Option 1: VB-CABLE (Free, Recommended for Beginners)

**Best for**: Simple setup, single application use

#### Installation Steps

1. **Download VB-CABLE**
   - Visit: https://vb-audio.com/Cable/
   - Click "Download" for VB-CABLE Virtual Audio Device
   - Extract the ZIP file

2. **Install the Driver**
   ```
   - Right-click on VBCABLE_Setup_x64.exe (or x86 for 32-bit)
   - Select "Run as Administrator"
   - Click "Install Driver"
   - Wait for installation to complete
   ```

3. **Reboot Your Computer**
   - Required for the driver to load properly

4. **Verify Installation**
   - Open Windows Sound Settings (Win + I → System → Sound)
   - You should see "CABLE Input" in Playback devices
   - You should see "CABLE Output" in Recording devices

#### Configuration

**Step 1: Configure VoiceBoard Output**
```
1. Right-click speaker icon in system tray
2. Select "Sounds" or "Sound settings"
3. Go to "Playback" tab
4. Set "CABLE Input" as Default Device
5. Click "OK"
```

**Step 2: Configure Application Input**
```
In your target application (Discord, Zoom, etc.):
1. Open audio settings
2. Set Input/Microphone to "CABLE Output"
3. Test audio
```

**Step 3: Monitor Your Audio (Optional)**
```
To hear yourself:
1. Open Sound Settings
2. Go to Recording devices
3. Right-click "CABLE Output"
4. Select "Properties"
5. Go to "Listen" tab
6. Check "Listen to this device"
7. Select your headphones/speakers
8. Click "OK"
```

### Option 2: VoiceMeeter (Free, Advanced)

**Best for**: Complex routing, multiple applications, mixing

#### Installation Steps

1. **Download VoiceMeeter**
   - Visit: https://vb-audio.com/Voicemeeter/
   - Choose:
     - VoiceMeeter: Basic (2 inputs)
     - VoiceMeeter Banana: Advanced (3 inputs)
     - VoiceMeeter Potato: Professional (5 inputs)
   - Recommended: VoiceMeeter Banana

2. **Install**
   ```
   - Run installer as Administrator
   - Follow installation wizard
   - Reboot when prompted
   ```

3. **Configure VoiceMeeter**
   ```
   - Launch VoiceMeeter
   - Hardware Input 1: Select your physical microphone
   - Hardware Out A1: Select your speakers/headphones
   - Virtual Inputs (VoiceMeeter Output/AUX) are now available
   ```

#### VoiceMeeter Setup for VoiceBoard

**Basic Configuration**:
```
┌────────────────┐
│ Physical Mic   │──→ VoiceBoard ──→ VoiceMeeter VAIO
└────────────────┘                         ↓
                                    Applications
                                           ↓
                                    Your Speakers
```

**Steps**:
1. Set VoiceBoard output to "VoiceMeeter Input (VB-Audio VoiceMeeter VAIO)"
2. In applications, set microphone to "VoiceMeeter Output (VB-Audio VoiceMeeter VAIO)"
3. In VoiceMeeter, route VAIO to A1 (your speakers) to monitor

### Option 3: Synchronous Audio Router (SAR) (Free, Lightweight)

**Best for**: Minimal overhead, advanced users

#### Installation

1. **Download SAR**
   - Visit: https://github.com/eiz/SynchronousAudioRouter
   - Download latest release
   - Extract to a folder

2. **Run SAR**
   ```
   - Run SarAsio.exe or SarWdm.exe as Administrator
   - Configure routing in the GUI
   ```

## Common Setup Scenarios

### Scenario 1: Discord Voice Chat

```
Configuration:
1. Windows Default Playback: CABLE Input
2. VoiceBoard outputs to Default (CABLE Input)
3. Discord Voice Input: CABLE Output
4. Discord Voice Output: Your headphones
5. Monitor via "Listen to this device" on CABLE Output
```

### Scenario 2: OBS Streaming

```
Configuration:
1. VoiceBoard outputs to CABLE Input
2. OBS Studio:
   - Add Audio Input Capture source
   - Select "CABLE Output"
   - Adjust volume in OBS mixer
3. Use separate physical output for monitoring
```

### Scenario 3: Gaming

```
Configuration:
1. VoiceBoard outputs to CABLE Input
2. Game voice chat input: CABLE Output
3. Game audio output: Your headphones
4. Monitor VoiceBoard via Listen feature
```

### Scenario 4: Zoom/Teams Meeting

```
Configuration:
1. VoiceBoard outputs to CABLE Input
2. Zoom/Teams Settings:
   - Microphone: CABLE Output
   - Speaker: Your headphones
3. Enable "Listen to this device" for monitoring
```

## Troubleshooting

### No Audio in Applications

**Problem**: Applications can't hear VoiceBoard output

**Solutions**:
1. Check Windows Sound Settings:
   - Ensure CABLE Input is default playback
   - Ensure CABLE Output is available as recording device
2. In application settings:
   - Manually select CABLE Output as microphone
   - Don't rely on "Default" device
3. Restart the application after changing settings

### Can't Hear Yourself

**Problem**: No monitoring of processed voice

**Solutions**:
1. Enable "Listen to this device":
   - Sound Settings → Recording → CABLE Output → Properties
   - Listen tab → Check "Listen to this device"
   - Select your headphones
2. Use VoiceMeeter for better monitoring control
3. Adjust levels in Windows mixer

### Crackling or Distorted Audio

**Problem**: Audio quality issues through virtual cable

**Solutions**:
1. Increase buffer size in VoiceBoard config:
   ```toml
   buffer_size = 1024  # or higher
   ```
2. Disable audio enhancements:
   - Sound Settings → CABLE Output → Properties
   - Enhancements tab → Disable all
3. Update virtual cable drivers
4. Close other audio applications

### Latency/Delay

**Problem**: Noticeable delay between speaking and hearing

**Solutions**:
1. Decrease VoiceBoard buffer size:
   ```toml
   buffer_size = 256
   ```
2. Use ASIO if available (VoiceMeeter supports ASIO)
3. Disable "Listen to this device" and monitor another way
4. Use wired headphones (Bluetooth adds latency)

### Feedback Loop

**Problem**: Echo or feedback

**Solutions**:
1. Don't output VoiceBoard to the same device you're monitoring
2. Use headphones, not speakers
3. Disable "Listen to this device" if not needed
4. Mute your microphone in the application when not speaking

## Advanced Routing

### Multiple Virtual Devices

For complex setups, chain multiple virtual cables:

```
Mic → VoiceBoard → CABLE 1 → OBS
                            ↘ CABLE 2 → Discord
```

1. Install VB-CABLE A+B (adds CABLE A and B)
2. Use audio routing software to split output
3. Route different processed streams to different apps

### Mixing Multiple Sources

Use VoiceMeeter to mix VoiceBoard with other audio:

```
Mic → VoiceBoard ────┐
Music Player ────────┼─→ VoiceMeeter ─→ Output
Game Audio ──────────┘
```

### Application-Specific Effects

Create multiple VoiceBoard instances with different configs:

```
Instance 1 (Discord): Heavy effects
Instance 2 (Zoom):    Minimal effects
Instance 3 (OBS):     Maximum quality
```

## Best Practices

1. **Always Use Wired Headphones**: Eliminates Bluetooth latency
2. **Set Consistent Sample Rates**: Use 48000 Hz everywhere
3. **Monitor CPU Usage**: Close unnecessary applications
4. **Test Before Going Live**: Verify setup in test calls
5. **Keep Drivers Updated**: Check for virtual cable updates
6. **Have a Backup**: Know how to quickly disable effects

## Uninstallation

### VB-CABLE
```
1. Run VBCABLE_Setup_x64.exe as Administrator
2. Click "Remove Driver"
3. Reboot
```

### VoiceMeeter
```
1. Use Windows "Add or Remove Programs"
2. Find VoiceMeeter
3. Click Uninstall
4. Reboot
```

## Additional Resources

- VB-Audio Forum: https://forum.vb-audio.com/
- VoiceMeeter Tutorial Videos: YouTube "VoiceMeeter Tutorial"
- Discord Audio Setup: https://support.discord.com/hc/en-us/articles/360045138471

## Quick Reference

### Essential Keyboard Shortcuts

**VoiceMeeter**:
- `Ctrl+M`: Mute all
- `A1`: Toggle hardware output 1
- `B1`: Toggle virtual output 1

### Port Names

- **VB-CABLE**: 
  - Output: "CABLE Input" (what you send TO)
  - Input: "CABLE Output" (what apps read FROM)

- **VoiceMeeter**:
  - Output: "VoiceMeeter Input (VAIO)"
  - Input: "VoiceMeeter Output (VAIO)"

## Need Help?

If you encounter issues:
1. Check this guide's troubleshooting section
2. Verify all drivers are installed correctly
3. Test with a simple app first (Windows Voice Recorder)
4. Consult VB-Audio documentation
5. Open an issue on the VoiceBoard GitHub repository
