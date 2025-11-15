use anyhow::Result;
use log::info;
use std::sync::Arc;
use parking_lot::Mutex;

mod audio;
mod dsp;
mod config;

use audio::{AudioCapture, AudioRenderer};
use dsp::EffectChain;
use config::Config;

fn main() -> Result<()> {
    env_logger::init();
    
    info!("VoiceBoard - Real-time Voice Changer");
    info!("====================================");
    
    // Initialize COM
    audio::initialize_com()?;
    
    // Load configuration
    let config = Config::default();
    info!("Configuration loaded: buffer_size={}, sample_rate={}", 
          config.buffer_size, config.sample_rate);
    
    // Initialize DSP effect chain
    let effect_chain = Arc::new(Mutex::new(EffectChain::new(config.sample_rate)?));
    info!("Effect chain initialized");
    
    // Apply default effects from config
    {
        let mut chain = effect_chain.lock();
        if let Some(pitch) = config.effects.pitch_shift {
            chain.set_pitch_shift(pitch)?;
        }
        if let Some(formant) = config.effects.formant_shift {
            chain.set_formant_shift(formant)?;
        }
        if config.effects.reverb_enabled {
            chain.enable_reverb()?;
        }
        if config.effects.robot_enabled {
            chain.enable_robot()?;
        }
        if let Some(distortion) = config.effects.distortion {
            chain.set_distortion(distortion)?;
        }
    }
    
    // Initialize audio capture
    let mut capture = AudioCapture::new(config.sample_rate, config.buffer_size)?;
    info!("Audio capture initialized");
    
    // Initialize audio renderer
    let renderer = Arc::new(Mutex::new(AudioRenderer::new(config.sample_rate, config.buffer_size)?));
    info!("Audio renderer initialized");
    
    info!("Starting real-time audio processing...");
    info!("Press Ctrl+C to stop");
    
    // Main processing loop
    let renderer_clone = renderer.clone();
    let result = capture.start(move |input_buffer| {
        let mut chain = effect_chain.lock();
        
        // Process audio through effect chain
        let output_buffer = chain.process(input_buffer);
        
        // Render to output device
        let mut renderer = renderer_clone.lock();
        renderer.render(&output_buffer);
    });
    
    // Cleanup COM
    audio::uninitialize_com();
    
    result
}
