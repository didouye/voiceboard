# Frequently Asked Questions (FAQ)

## General Questions

### Q: What is VoiceBoard?
A: VoiceBoard is a real-time voice changer for Windows that captures your microphone input, applies digital signal processing (DSP) effects, and outputs the modified audio to applications like Discord, Zoom, or OBS.

### Q: Is VoiceBoard free?
A: Yes, VoiceBoard is open-source and free to use under the MIT license.

### Q: What makes VoiceBoard different from other voice changers?
A: VoiceBoard is built in Rust for performance and reliability, uses professional WASAPI audio APIs, and is designed for low latency. It's also open-source so you can see exactly how it works and customize it.

### Q: Can I use VoiceBoard commercially?
A: Yes, the MIT license allows commercial use.

## Platform and Compatibility

### Q: Does VoiceBoard work on Mac or Linux?
A: No, VoiceBoard is Windows-only because it uses Windows Audio Session API (WASAPI). There are no plans for Mac or Linux versions currently.

### Q: What versions of Windows are supported?
A: Windows 10 and Windows 11 (64-bit) are officially supported. Windows 8.1 may work but is not tested.

### Q: Can I run VoiceBoard on Windows 7?
A: Not officially supported. The WASAPI features used may not be available on Windows 7.

### Q: Does VoiceBoard work with Windows on ARM?
A: Not tested. It should work if you have the ARM64 Rust toolchain, but this hasn't been verified.

## Installation and Setup

### Q: Do I need to install anything besides VoiceBoard?
A: Yes, you need:
1. Rust toolchain (to build from source)
2. Virtual audio cable software (VB-CABLE or VoiceMeeter)
3. A microphone
4. Headphones or speakers

### Q: The build fails with "linker error". What's wrong?
A: You need Visual Studio Build Tools with the MSVC compiler. Download from https://visualstudio.microsoft.com/downloads/ and install "Desktop development with C++".

### Q: Can I use the pre-built executable?
A: Currently, you need to build from source. Pre-built releases may be added in the future.

### Q: How do I update VoiceBoard?
A:
```bash
git pull
cargo build --release
```

## Virtual Audio Devices

### Q: Why do I need a virtual audio cable?
A: VoiceBoard processes your voice and outputs it to an audio device. A virtual cable lets other applications (Discord, Zoom, etc.) receive this processed audio as if it were a microphone.

### Q: Which virtual audio software is best?
A: For simplicity: VB-CABLE (free, simple setup). For advanced features: VoiceMeeter (free, more control).

### Q: Can I use VoiceBoard without a virtual cable?
A: Only for monitoring yourself. To use in applications, you need a virtual cable.

### Q: I installed VB-CABLE but can't see it in Windows.
A: Make sure you:
1. Ran the installer as Administrator
2. Rebooted your computer
3. Check Sound Settings for "CABLE Input" and "CABLE Output"

### Q: Do I need to pay for virtual audio cables?
A: No, both VB-CABLE and VoiceMeeter are free (donations appreciated by their developers).

## Usage and Configuration

### Q: How do I change effects in real-time?
A: Currently, you need to edit `config.toml` and restart VoiceBoard. Real-time GUI control is planned for future versions.

### Q: Can I save multiple presets?
A: Create multiple config files (e.g., `discord.toml`, `gaming.toml`) and copy the one you want to `config.toml` before running.

### Q: What happens if I don't create a config.toml?
A: VoiceBoard uses default settings (pass-through mode with no effects).

### Q: How do I reset to default settings?
A: Delete or rename `config.toml` and VoiceBoard will use defaults.

### Q: Can I use multiple effects at once?
A: Yes! Enable as many effects as you want in the config file.

## Performance

### Q: How much CPU does VoiceBoard use?
A: Typically 5-15% on modern CPUs with all effects enabled. Single effects use 2-5%.

### Q: My CPU usage is very high. What can I do?
A: 
1. Increase buffer_size in config.toml (e.g., to 1024 or 2048)
2. Disable heavy effects (reverb is the most CPU-intensive)
3. Lower sample rate to 44100 Hz
4. Close other applications

### Q: I hear crackling in the audio. Why?
A: This usually means buffer underruns. Solutions:
1. Increase buffer_size in config.toml
2. Close other CPU-intensive applications
3. Disable some effects
4. Update audio drivers

### Q: How much latency does VoiceBoard add?
A: Depends on buffer size:
- 256 samples @ 48kHz: ~21ms total
- 512 samples @ 48kHz: ~32ms total
- 1024 samples @ 48kHz: ~57ms total

### Q: Can I reduce latency?
A: Yes:
1. Decrease buffer_size (e.g., to 256)
2. Use fewer effects
3. Ensure no other applications are using audio devices
4. Use wired headphones (Bluetooth adds latency)

## Audio Quality

### Q: Why does my voice sound robotic?
A: If robot_enabled = true, that's intentional. Otherwise, try increasing buffer_size to reduce artifacts.

### Q: The pitch shift sounds unnatural. Can I improve it?
A: Phase vocoder pitch shifting has inherent limitations. Use smaller pitch shifts (±3 semitones) for more natural results.

### Q: Why is there echo in my audio?
A: Check that you don't have "Listen to this device" enabled on both your physical microphone AND the virtual cable. You only need it on the virtual cable.

### Q: Can I improve audio quality?
A: Yes:
1. Use a better microphone
2. Use higher sample rate (48000 Hz)
3. Use larger buffer size
4. Use fewer effects (less processing = less artifacts)

## Application Integration

### Q: How do I use VoiceBoard with Discord?
A:
1. Run VoiceBoard
2. In Discord: Settings → Voice & Video
3. Input Device: CABLE Output
4. Test audio

### Q: VoiceBoard works but Discord doesn't hear me.
A: Make sure:
1. Discord input is set to "CABLE Output" (not "Default")
2. VoiceBoard is running
3. Your Windows default playback is "CABLE Input"
4. You're not muted in Discord

### Q: Can I use VoiceBoard with Zoom?
A: Yes! Set Zoom's microphone to "CABLE Output".

### Q: Does VoiceBoard work with games?
A: Yes, if the game has voice chat settings, set the microphone to "CABLE Output".

### Q: Can I stream with VoiceBoard in OBS?
A: Yes! Add an "Audio Input Capture" source in OBS and select "CABLE Output".

### Q: Can I use VoiceBoard in multiple applications simultaneously?
A: Yes! All applications can read from "CABLE Output" at the same time.

## Troubleshooting

### Q: I can't hear myself. What's wrong?
A: Enable "Listen to this device":
1. Sound Settings → Recording
2. Right-click CABLE Output → Properties
3. Listen tab → Check "Listen to this device"
4. Select your headphones

### Q: I hear myself twice (echo).
A: You have monitoring enabled in two places. Disable "Listen to this device" or the application's monitoring feature.

### Q: VoiceBoard starts but no audio processes.
A: Check:
1. Your microphone is set as Windows default recording device
2. CABLE Input is set as Windows default playback device
3. VoiceBoard has permission to access microphone
4. Check logs with `RUST_LOG=debug cargo run --release`

### Q: VoiceBoard crashes on startup.
A: Common causes:
1. No microphone connected
2. Audio device in use by another application
3. Virtual cable not installed
4. Check logs for specific error

### Q: I get "Access Denied" errors.
A: Run Command Prompt as Administrator when building/running.

### Q: Audio cuts in and out.
A: Increase buffer_size to 1024 or 2048 to reduce dropouts.

## Effects

### Q: How much can I change my pitch?
A: Technically ±24 semitones (2 octaves), but ±12 semitones sounds more natural.

### Q: What's the difference between pitch shift and formant shift?
A: 
- Pitch shift: Changes the frequency of your voice (higher/lower)
- Formant shift: Changes vocal characteristics (younger/older, male/female)

### Q: Can I make myself sound like a specific person?
A: No, voice cloning requires different technology (neural networks). VoiceBoard only applies DSP effects.

### Q: What is the robot effect?
A: Ring modulation that creates a synthetic, robotic sound quality.

### Q: Why does reverb use more CPU than other effects?
A: Reverb uses multiple delay lines and filters, requiring more processing.

### Q: Can I add my own effects?
A: Yes! VoiceBoard is open-source. Add effects in `src/dsp/` and integrate them into the effect chain.

## Privacy and Security

### Q: Does VoiceBoard collect any data?
A: No. VoiceBoard runs entirely on your computer and doesn't send data anywhere.

### Q: Does VoiceBoard record my voice?
A: No. It processes audio in real-time and doesn't save anything to disk.

### Q: Is my voice data secure?
A: Yes. All processing happens locally on your computer.

### Q: Can VoiceBoard be used for voice anonymization?
A: It can disguise your voice with effects, but it's not designed for security/anonymity purposes.

## Development and Contributing

### Q: Can I contribute to VoiceBoard?
A: Yes! Pull requests are welcome. See the code on GitHub.

### Q: What programming language is VoiceBoard written in?
A: Rust, for performance and memory safety.

### Q: Can I use VoiceBoard code in my own project?
A: Yes, under the terms of the MIT license.

### Q: How can I report bugs?
A: Open an issue on the GitHub repository with:
- OS version
- VoiceBoard version
- Steps to reproduce
- Error messages or logs

### Q: Are there plans for a GUI?
A: Yes! A graphical interface for real-time control is planned for future versions.

### Q: Will there be VST plugin support?
A: It's on the roadmap but not currently in development.

## Advanced Usage

### Q: Can I use ASIO instead of WASAPI?
A: Not currently. ASIO support is planned for future versions.

### Q: Can I route VoiceBoard output to multiple virtual cables?
A: Not directly, but you can use VoiceMeeter to split the output to multiple destinations.

### Q: Can I automate effect changes?
A: Not currently. Future versions may support automation via MIDI or OSC.

### Q: Can I use VoiceBoard as a VST plugin in my DAW?
A: Not currently. It's a standalone application.

### Q: Does VoiceBoard support stereo input?
A: Currently mono only. Stereo support may be added in the future.

## Comparison to Other Software

### Q: How does VoiceBoard compare to Voicemod?
A: 
- VoiceBoard: Free, open-source, manual config, lower latency
- Voicemod: Commercial, GUI, presets, more effects

### Q: Is VoiceBoard better than Clownfish?
A: Different goals:
- VoiceBoard: Modern, Rust-based, WASAPI, actively developed
- Clownfish: Older, system-wide, simpler setup

### Q: Should I use VoiceBoard or MorphVOX?
A: 
- VoiceBoard: Free, open-source, modern tech
- MorphVOX: Commercial, more features, better voice morphing

## Licensing

### Q: Can I use VoiceBoard for YouTube videos?
A: Yes, the MIT license allows this.

### Q: Can I sell software that includes VoiceBoard?
A: Yes, but you must include the MIT license and give credit.

### Q: Can I modify VoiceBoard and sell it?
A: Yes, under the MIT license terms.

### Q: Do I need to credit VoiceBoard?
A: Not required, but appreciated!

## Still Have Questions?

- Check the [README.md](README.md) for general information
- Read [BUILD.md](BUILD.md) for build instructions
- See [EXAMPLES.md](EXAMPLES.md) for usage examples
- Review [VIRTUAL_AUDIO_SETUP.md](VIRTUAL_AUDIO_SETUP.md) for setup help
- Open an issue on GitHub for bugs or feature requests
