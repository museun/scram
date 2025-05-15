#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Config {
    pub banding: Banding,
    pub window: Window,
    pub scaling: VolumeScale,
    pub band_smoothing: BandSmoothing,
    pub peak_smoothing: PeakSmoothing,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Banding {
    pub frequency_cutoff: FrequencyCutoff,
    pub scale: FrequencyScale,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum FrequencyScale {
    Linear,
    Logarithmic,
    Bark,
    #[default]
    Mel,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum Window {
    None,
    Hann,
    Hamming,
    #[default]
    Blackman,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
#[non_exhaustive]
pub enum VolumeScale {
    Linear,
    #[default]
    Logarithimic,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PeakSmoothing {
    pub attack_rate: f32,
    pub decay_rate: f32,
    pub decay_limit: f32,
    pub peak_threshold: f32,
}

impl Default for PeakSmoothing {
    fn default() -> Self {
        Self {
            attack_rate: 20.0,
            decay_rate: 0.5,
            decay_limit: 1.0,
            peak_threshold: 1e-4,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum BandSmoothing {
    None,
    Exponential { factor: f32 },
    MovingAverage { window_size: usize },
}

impl Default for BandSmoothing {
    fn default() -> Self {
        Self::Exponential { factor: 0.5 }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FrequencyCutoff {
    pub low: f32,
    pub high: f32,
}

impl Default for FrequencyCutoff {
    fn default() -> Self {
        Self {
            low: 20.0,
            high: 18000.0,
        }
    }
}
