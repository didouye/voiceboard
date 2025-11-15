use anyhow::{Result, Context};
use log::{info, debug};
use windows::Win32::Media::Audio::{
    IAudioClient, IAudioRenderClient, IMMDeviceEnumerator, MMDeviceEnumerator,
    eRender, eConsole, AUDCLNT_SHAREMODE_SHARED,
    WAVEFORMATEX, WAVE_FORMAT_PCM,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CLSCTX_ALL,
};
use std::ptr;

pub struct AudioRenderer {
    sample_rate: u32,
    buffer_size: usize,
    audio_client: Option<IAudioClient>,
    render_client: Option<IAudioRenderClient>,
}

impl AudioRenderer {
    pub fn new(sample_rate: u32, buffer_size: usize) -> Result<Self> {
        info!("Initializing audio renderer at {} Hz, buffer size: {}", sample_rate, buffer_size);
        
        unsafe {
            // Create device enumerator
            let enumerator: IMMDeviceEnumerator = 
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                    .context("Failed to create device enumerator")?;
            
            // Get default render device
            let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)
                .context("Failed to get default render device")?;
            
            debug!("Got default render device");
            
            // Activate audio client
            let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)
                .context("Failed to activate audio client")?;
            
            // Set up wave format
            let wave_format = WAVEFORMATEX {
                wFormatTag: WAVE_FORMAT_PCM as u16,
                nChannels: 1, // Mono
                nSamplesPerSec: sample_rate,
                nAvgBytesPerSec: sample_rate * 4, // 32-bit float = 4 bytes
                nBlockAlign: 4,
                wBitsPerSample: 32,
                cbSize: 0,
            };
            
            // Initialize audio client
            let buffer_duration = (buffer_size as f64 / sample_rate as f64 * 10_000_000.0) as i64;
            
            audio_client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                0,
                buffer_duration,
                0,
                &wave_format,
                None,
            ).context("Failed to initialize audio client")?;
            
            debug!("Audio renderer client initialized");
            
            // Get render client
            let render_client: IAudioRenderClient = audio_client.GetService()
                .context("Failed to get render client")?;
            
            // Start the audio client
            audio_client.Start()
                .context("Failed to start audio client")?;
            
            info!("Audio renderer initialized successfully");
            
            Ok(Self {
                sample_rate,
                buffer_size,
                audio_client: Some(audio_client),
                render_client: Some(render_client),
            })
        }
    }
    
    pub fn render(&mut self, samples: &[f32]) {
        unsafe {
            if let Some(ref render_client) = self.render_client {
                if let Some(ref audio_client) = self.audio_client {
                    // Get buffer size
                    if let Ok(buffer_size) = audio_client.GetBufferSize() {
                        // Get current padding
                        if let Ok(padding) = audio_client.GetCurrentPadding() {
                            let available = buffer_size - padding;
                            let frames_to_write = available.min(samples.len() as u32);
                            
                            if frames_to_write > 0 {
                                // Get buffer
                                if let Ok(data_ptr) = render_client.GetBuffer(frames_to_write) {
                                    if !data_ptr.is_null() {
                                        // Copy samples to buffer
                                        let buffer = std::slice::from_raw_parts_mut(
                                            data_ptr as *mut f32,
                                            frames_to_write as usize,
                                        );
                                        
                                        let copy_len = frames_to_write.min(samples.len() as u32) as usize;
                                        buffer[..copy_len].copy_from_slice(&samples[..copy_len]);
                                        
                                        // Release buffer
                                        let _ = render_client.ReleaseBuffer(frames_to_write, 0);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Drop for AudioRenderer {
    fn drop(&mut self) {
        unsafe {
            if let Some(ref audio_client) = self.audio_client {
                let _ = audio_client.Stop();
            }
        }
        
        info!("Audio renderer cleaned up");
    }
}
