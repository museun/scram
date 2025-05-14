#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Config {
    pub binning: Binning,
    pub window: Window,
    pub scaling: Scaling,
    pub band_smoothing: BandSmoothing,
    pub peak_smoothing: PeakSmoothing,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Binning {
    pub frequency_cutoff: FrequencyCutoff,
    pub banding: Banding,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum Banding {
    Linear,
    Logarithmic,
    Bark,
    #[default]
    Mel,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum Window {
    Hann,
    Hamming,
    #[default]
    Blackman,
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
#[non_exhaustive]
pub enum Scaling {
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
    Exponential { factor: f32 },
    MovingAverage { window_size: f32 },
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
