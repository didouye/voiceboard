use anyhow::{Result, Context};
use log::{info, debug};
use windows::Win32::Media::Audio::{
    IAudioClient, IAudioCaptureClient, IMMDeviceEnumerator, MMDeviceEnumerator,
    eCapture, eConsole, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
    WAVEFORMATEX, WAVE_FORMAT_PCM,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CLSCTX_ALL,
};
use windows::Win32::Foundation::{HANDLE, CloseHandle};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject, INFINITE};
use windows::core::PCWSTR;
use std::ptr;

pub struct AudioCapture {
    sample_rate: u32,
    buffer_size: usize,
    audio_client: Option<IAudioClient>,
    capture_client: Option<IAudioCaptureClient>,
    event_handle: HANDLE,
}

impl AudioCapture {
    pub fn new(sample_rate: u32, buffer_size: usize) -> Result<Self> {
        info!("Initializing audio capture at {} Hz, buffer size: {}", sample_rate, buffer_size);
        
        unsafe {
            // Create device enumerator
            let enumerator: IMMDeviceEnumerator = 
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                    .context("Failed to create device enumerator")?;
            
            // Get default capture device
            let device = enumerator.GetDefaultAudioEndpoint(eCapture, eConsole)
                .context("Failed to get default capture device")?;
            
            debug!("Got default capture device");
            
            // Activate audio client
            let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)
                .context("Failed to activate audio client")?;
            
            // Set up wave format
            let mut wave_format = WAVEFORMATEX {
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
                AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                buffer_duration,
                0,
                &wave_format,
                None,
            ).context("Failed to initialize audio client")?;
            
            debug!("Audio client initialized");
            
            // Create event for callbacks
            let event_handle = CreateEventW(None, false, false, PCWSTR::null())
                .context("Failed to create event")?;
            
            audio_client.SetEventHandle(event_handle)
                .context("Failed to set event handle")?;
            
            // Get capture client
            let capture_client: IAudioCaptureClient = audio_client.GetService()
                .context("Failed to get capture client")?;
            
            info!("Audio capture initialized successfully");
            
            Ok(Self {
                sample_rate,
                buffer_size,
                audio_client: Some(audio_client),
                capture_client: Some(capture_client),
                event_handle,
            })
        }
    }
    
    pub fn start<F>(&mut self, mut callback: F) -> Result<()>
    where
        F: FnMut(&[f32]),
    {
        info!("Starting audio capture");
        
        unsafe {
            if let Some(ref audio_client) = self.audio_client {
                audio_client.Start()
                    .context("Failed to start audio client")?;
            }
            
            loop {
                // Wait for buffer to be ready
                WaitForSingleObject(self.event_handle, INFINITE);
                
                if let Some(ref capture_client) = self.capture_client {
                    // Get next packet size
                    let packet_size = match capture_client.GetNextPacketSize() {
                        Ok(size) => size,
                        Err(_) => continue,
                    };
                    
                    if packet_size > 0 {
                        let mut data_ptr = ptr::null_mut();
                        let mut num_frames = 0u32;
                        let mut flags = 0u32;
                        
                        // Get buffer
                        if let Ok(()) = capture_client.GetBuffer(
                            &mut data_ptr,
                            &mut num_frames,
                            &mut flags,
                            None,
                            None,
                        ) {
                            if num_frames > 0 && !data_ptr.is_null() {
                                // Convert to f32 slice
                                let samples = std::slice::from_raw_parts(
                                    data_ptr as *const f32,
                                    num_frames as usize,
                                );
                                
                                // Call processing callback
                                callback(samples);
                            }
                            
                            // Release buffer
                            let _ = capture_client.ReleaseBuffer(num_frames);
                        }
                    }
                }
            }
        }
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        unsafe {
            if let Some(ref audio_client) = self.audio_client {
                let _ = audio_client.Stop();
            }
            
            if !self.event_handle.is_invalid() {
                let _ = CloseHandle(self.event_handle);
            }
        }
        
        info!("Audio capture cleaned up");
    }
}
