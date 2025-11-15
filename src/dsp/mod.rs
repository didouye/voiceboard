use anyhow::Result;
use realfft::RealFftPlanner;
use num_complex::Complex;

mod pitch_shifter;
mod formant_shifter;
mod reverb;
mod robot_effect;
mod distortion;

pub use pitch_shifter::PitchShifter;
pub use formant_shifter::FormantShifter;
pub use reverb::Reverb;
pub use robot_effect::RobotEffect;
pub use distortion::Distortion;

/// Main effect chain that processes audio through multiple effects
pub struct EffectChain {
    sample_rate: u32,
    pitch_shifter: Option<PitchShifter>,
    formant_shifter: Option<FormantShifter>,
    reverb: Option<Reverb>,
    robot: Option<RobotEffect>,
    distortion: Option<Distortion>,
    
    // FFT processing components
    fft_planner: RealFftPlanner<f32>,
    fft_size: usize,
    scratch_buffer: Vec<Complex<f32>>,
    input_buffer: Vec<f32>,
    output_buffer: Vec<f32>,
}

impl EffectChain {
    pub fn new(sample_rate: u32) -> Result<Self> {
        let fft_size = 2048;
        let fft_planner = RealFftPlanner::new();
        
        Ok(Self {
            sample_rate,
            pitch_shifter: None,
            formant_shifter: None,
            reverb: None,
            robot: None,
            distortion: None,
            fft_planner,
            fft_size,
            scratch_buffer: vec![Complex::new(0.0, 0.0); fft_size / 2 + 1],
            input_buffer: vec![0.0; fft_size],
            output_buffer: vec![0.0; fft_size],
        })
    }
    
    pub fn set_pitch_shift(&mut self, semitones: f32) -> Result<()> {
        self.pitch_shifter = Some(PitchShifter::new(self.sample_rate, semitones));
        Ok(())
    }
    
    pub fn set_formant_shift(&mut self, shift: f32) -> Result<()> {
        self.formant_shifter = Some(FormantShifter::new(self.sample_rate, shift));
        Ok(())
    }
    
    pub fn enable_reverb(&mut self) -> Result<()> {
        self.reverb = Some(Reverb::new(self.sample_rate)?);
        Ok(())
    }
    
    pub fn enable_robot(&mut self) -> Result<()> {
        self.robot = Some(RobotEffect::new(self.sample_rate));
        Ok(())
    }
    
    pub fn set_distortion(&mut self, amount: f32) -> Result<()> {
        self.distortion = Some(Distortion::new(amount));
        Ok(())
    }
    
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output = input.to_vec();
        
        // Apply pitch shifting
        if let Some(ref mut pitch_shifter) = self.pitch_shifter {
            output = pitch_shifter.process(&output);
        }
        
        // Apply formant shifting
        if let Some(ref mut formant_shifter) = self.formant_shifter {
            output = formant_shifter.process(&output);
        }
        
        // Apply robot effect
        if let Some(ref mut robot) = self.robot {
            output = robot.process(&output);
        }
        
        // Apply distortion
        if let Some(ref mut distortion) = self.distortion {
            output = distortion.process(&output);
        }
        
        // Apply reverb (last for natural sound)
        if let Some(ref mut reverb) = self.reverb {
            output = reverb.process(&output);
        }
        
        output
    }
}
