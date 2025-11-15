use num_complex::Complex;
use realfft::{RealFftPlanner, RealToComplex, ComplexToReal};
use std::sync::Arc;

/// Formant shifter using spectral envelope manipulation
pub struct FormantShifter {
    sample_rate: u32,
    shift_ratio: f32,
    fft_size: usize,
    hop_size: usize,
    
    // FFT components
    r2c: Arc<dyn RealToComplex<f32>>,
    c2r: Arc<dyn ComplexToReal<f32>>,
    
    // Buffers
    input_buffer: Vec<f32>,
    output_buffer: Vec<f32>,
    fft_buffer: Vec<Complex<f32>>,
    window: Vec<f32>,
    overlap_buffer: Vec<f32>,
    input_position: usize,
}

impl FormantShifter {
    pub fn new(sample_rate: u32, shift: f32) -> Self {
        let fft_size = 2048;
        let hop_size = fft_size / 4;
        let shift_ratio = 2.0_f32.powf(shift / 12.0);
        
        let mut planner = RealFftPlanner::new();
        let r2c = planner.plan_fft_forward(fft_size);
        let c2r = planner.plan_fft_inverse(fft_size);
        
        // Hann window
        let window: Vec<f32> = (0..fft_size)
            .map(|i| {
                0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / fft_size as f32).cos())
            })
            .collect();
        
        Self {
            sample_rate,
            shift_ratio,
            fft_size,
            hop_size,
            r2c,
            c2r,
            input_buffer: vec![0.0; fft_size],
            output_buffer: vec![0.0; fft_size],
            fft_buffer: vec![Complex::new(0.0, 0.0); fft_size / 2 + 1],
            window,
            overlap_buffer: vec![0.0; fft_size],
            input_position: 0,
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
            
            output.push(self.overlap_buffer[0]);
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
        
        // Shift formants by warping frequency axis
        let mut shifted_buffer = vec![Complex::new(0.0, 0.0); self.fft_buffer.len()];
        
        for i in 0..self.fft_buffer.len() {
            let source_idx = (i as f32 / self.shift_ratio) as usize;
            if source_idx < self.fft_buffer.len() {
                shifted_buffer[i] = self.fft_buffer[source_idx];
            }
        }
        
        self.fft_buffer.copy_from_slice(&shifted_buffer);
        
        // Inverse FFT
        self.c2r.process(&mut self.fft_buffer, &mut self.output_buffer).unwrap();
        
        // Normalize
        let norm_factor = 1.0 / (self.fft_size as f32);
        for sample in self.output_buffer.iter_mut() {
            *sample *= norm_factor;
        }
        
        // Overlap-add
        for i in 0..self.fft_size {
            self.overlap_buffer[i] += self.output_buffer[i] * self.window[i];
        }
    }
}
