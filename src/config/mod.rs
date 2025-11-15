use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub effects: EffectConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectConfig {
    pub pitch_shift: Option<f32>,
    pub formant_shift: Option<f32>,
    pub reverb_enabled: bool,
    pub robot_enabled: bool,
    pub distortion: Option<f32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            buffer_size: 512,
            effects: EffectConfig {
                pitch_shift: None,
                formant_shift: None,
                reverb_enabled: false,
                robot_enabled: false,
                distortion: None,
            },
        }
    }
}

impl Config {
    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save_to_file(&self, path: &str) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
