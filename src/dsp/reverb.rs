use anyhow::Result;

/// Reverb effect using Schroeder reverberator algorithm
pub struct Reverb {
    sample_rate: u32,
    
    // Comb filters
    comb_delays: Vec<Vec<f32>>,
    comb_indices: Vec<usize>,
    comb_gains: Vec<f32>,
    
    // All-pass filters
    allpass_delays: Vec<Vec<f32>>,
    allpass_indices: Vec<usize>,
    allpass_gains: Vec<f32>,
    
    // Mix parameters
    wet_mix: f32,
    dry_mix: f32,
}

impl Reverb {
    pub fn new(sample_rate: u32) -> Result<Self> {
        // Delay times in milliseconds (Schroeder's recommended values)
        let comb_delay_times = vec![29.7, 37.1, 41.1, 43.7];
        let allpass_delay_times = vec![5.0, 1.7];
        
        // Convert to samples
        let comb_delays: Vec<Vec<f32>> = comb_delay_times
            .iter()
            .map(|&time| {
                let samples = (time * sample_rate as f32 / 1000.0) as usize;
                vec![0.0; samples]
            })
            .collect();
        
        let allpass_delays: Vec<Vec<f32>> = allpass_delay_times
            .iter()
            .map(|&time| {
                let samples = (time * sample_rate as f32 / 1000.0) as usize;
                vec![0.0; samples]
            })
            .collect();
        
        let num_combs = comb_delays.len();
        let num_allpass = allpass_delays.len();
        
        Ok(Self {
            sample_rate,
            comb_delays,
            comb_indices: vec![0; num_combs],
            comb_gains: vec![0.742; num_combs],
            allpass_delays,
            allpass_indices: vec![0; num_allpass],
            allpass_gains: vec![0.7; num_allpass],
            wet_mix: 0.3,
            dry_mix: 0.7,
        })
    }
    
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(input.len());
        
        for &sample in input {
            // Process through parallel comb filters
            let mut comb_sum = 0.0;
            for i in 0..self.comb_delays.len() {
                let delay_length = self.comb_delays[i].len();
                let delayed = self.comb_delays[i][self.comb_indices[i]];
                
                // Comb filter output
                let comb_out = delayed;
                comb_sum += comb_out;
                
                // Update delay line
                self.comb_delays[i][self.comb_indices[i]] = 
                    sample + delayed * self.comb_gains[i];
                
                self.comb_indices[i] = (self.comb_indices[i] + 1) % delay_length;
            }
            
            // Average the comb outputs
            let mut processed = comb_sum / self.comb_delays.len() as f32;
            
            // Process through series all-pass filters
            for i in 0..self.allpass_delays.len() {
                let delay_length = self.allpass_delays[i].len();
                let delayed = self.allpass_delays[i][self.allpass_indices[i]];
                
                // All-pass filter
                let allpass_out = -processed + delayed;
                self.allpass_delays[i][self.allpass_indices[i]] = 
                    processed + delayed * self.allpass_gains[i];
                
                processed = allpass_out;
                self.allpass_indices[i] = (self.allpass_indices[i] + 1) % delay_length;
            }
            
            // Mix dry and wet signals
            output.push(sample * self.dry_mix + processed * self.wet_mix);
        }
        
        output
    }
}
