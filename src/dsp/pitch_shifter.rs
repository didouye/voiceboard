use num_complex::Complex;
use realfft::{RealFftPlanner, RealToComplex, ComplexToReal};
use std::sync::Arc;

/// Pitch shifter using phase vocoder algorithm
pub struct PitchShifter {
    sample_rate: u32,
    pitch_ratio: f32,
    fft_size: usize,
    hop_size: usize,
    
    // FFT components
    r2c: Arc<dyn RealToComplex<f32>>,
    c2r: Arc<dyn ComplexToReal<f32>>,
    
    // Buffers
    input_buffer: Vec<f32>,
    output_buffer: Vec<f32>,
    fft_buffer: Vec<Complex<f32>>,
    last_phase: Vec<f32>,
    sum_phase: Vec<f32>,
    
    // Windowing
    window: Vec<f32>,
    
    // Overlap-add
    overlap_buffer: Vec<f32>,
    input_position: usize,
    output_position: usize,
}

impl PitchShifter {
    pub fn new(sample_rate: u32, semitones: f32) -> Self {
        let fft_size = 2048;
        let hop_size = fft_size / 4;
        let pitch_ratio = 2.0_f32.powf(semitones / 12.0);
        
        let mut planner = RealFftPlanner::new();
        let r2c = planner.plan_fft_forward(fft_size);
        let c2r = planner.plan_fft_inverse(fft_size);
        
        // Create Hann window
        let window: Vec<f32> = (0..fft_size)
            .map(|i| {
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / fft_size as f32).cos())
            })
            .collect();
        
        Self {
            sample_rate,
            pitch_ratio,
            fft_size,
            hop_size,
            r2c,
            c2r,
            input_buffer: vec![0.0; fft_size],
            output_buffer: vec![0.0; fft_size],
            fft_buffer: vec![Complex::new(0.0, 0.0); fft_size / 2 + 1],
            last_phase: vec![0.0; fft_size / 2 + 1],
            sum_phase: vec![0.0; fft_size / 2 + 1],
            window,
            overlap_buffer: vec![0.0; fft_size],
            input_position: 0,
            output_position: 0,
        }
    }
    
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(input.len());
        
        for &sample in input {
            self.input_buffer[self.input_position] = sample;
            self.input_position += 1;
            
            if self.input_position >= self.hop_size {
                self.process_frame();
                self.input_position = 0;
            }
            
            // Output sample
            output.push(self.overlap_buffer[0]);
            
            // Shift overlap buffer
            self.overlap_buffer.rotate_left(1);
            self.overlap_buffer[self.fft_size - 1] = 0.0;
        }
        
        output
    }
    
    fn process_frame(&mut self) {
        // Apply window
        let mut windowed: Vec<f32> = self.input_buffer
            .iter()
            .zip(self.window.iter())
            .map(|(s, w)| s * w)
            .collect();
        
        // Forward FFT
        self.r2c.process(&mut windowed, &mut self.fft_buffer).unwrap();
        
        // Phase vocoder processing
        for i in 0..self.fft_buffer.len() {
            let magnitude = self.fft_buffer[i].norm();
            let phase = self.fft_buffer[i].arg();
            
            // Calculate phase difference
            let phase_diff = phase - self.last_phase[i];
            self.last_phase[i] = phase;
            
            // Calculate true frequency
            let bin_center_freq = 2.0 * std::f32::consts::PI * (i as f32) / (self.fft_size as f32);
            let phase_deviation = phase_diff - bin_center_freq * (self.hop_size as f32);
            
            // Unwrap phase
            let wrapped_deviation = ((phase_deviation + std::f32::consts::PI) % (2.0 * std::f32::consts::PI)) - std::f32::consts::PI;
            
            // Calculate instantaneous frequency
            let true_freq = bin_center_freq + wrapped_deviation / (self.hop_size as f32);
            
            // Update accumulated phase with pitch shift
            self.sum_phase[i] += true_freq * (self.hop_size as f32) * self.pitch_ratio;
            
            // Reconstruct complex number
            self.fft_buffer[i] = Complex::new(
                magnitude * self.sum_phase[i].cos(),
                magnitude * self.sum_phase[i].sin(),
            );
        }
        
        // Inverse FFT
        self.c2r.process(&mut self.fft_buffer, &mut self.output_buffer).unwrap();
        
        // Normalize
        let norm_factor = 1.0 / (self.fft_size as f32);
        for sample in self.output_buffer.iter_mut() {
            *sample *= norm_factor;
        }
        
        // Apply window and overlap-add
        for i in 0..self.fft_size {
            self.overlap_buffer[i] += self.output_buffer[i] * self.window[i];
        }
    }
}
