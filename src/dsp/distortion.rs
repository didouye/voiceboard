/// Distortion effect using wave shaping
pub struct Distortion {
    amount: f32,
    mix: f32,
}

impl Distortion {
    pub fn new(amount: f32) -> Self {
        Self {
            amount: amount.clamp(0.0, 1.0),
            mix: 0.5,
        }
    }
    
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output = Vec::with_capacity(input.len());
        
        for &sample in input {
            // Soft clipping distortion using tanh
            let gain = 1.0 + self.amount * 9.0; // 1x to 10x gain
            let distorted = (sample * gain).tanh();
            
            // Mix dry and wet
            let mixed = sample * (1.0 - self.mix) + distorted * self.mix;
            
            output.push(mixed);
        }
        
        output
    }
}
