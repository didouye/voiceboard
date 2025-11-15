/// Robot voice effect using ring modulation and pitch quantization
pub struct RobotEffect {
    sample_rate: u32,
    carrier_freq: f32,
    phase: f32,
}

impl RobotEffect {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            carrier_freq: 30.0, // Carrier frequency for ring modulation
            phase: 0.0,
        }
    }
    
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(input.len());
        let phase_increment = 2.0 * std::f32::consts::PI * self.carrier_freq / self.sample_rate as f32;
        
        for &sample in input {
            // Ring modulation
            let carrier = self.phase.sin();
            let modulated = sample * carrier;
            
            // Hard clip for robotic effect
            let clipped = modulated.max(-0.8).min(0.8);
            
            output.push(clipped);
            
            // Update phase
            self.phase += phase_increment;
            if self.phase >= 2.0 * std::f32::consts::PI {
                self.phase -= 2.0 * std::f32::consts::PI;
            }
        }
        
        output
    }
}
