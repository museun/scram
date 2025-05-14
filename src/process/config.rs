use std::f32::consts::TAU;

const N: f32 = (super::SAMPLE_SIZE / 2) as f32;

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

impl Window {
    #[inline]
    pub(crate) const fn apply(&self) -> fn(f32) -> f32 {
        #[inline(always)]
        fn hann(d: f32) -> f32 {
            0.5 * (1.0 - (TAU * d / N - 1.0).cos())
        }

        #[inline(always)]
        fn hamming(d: f32) -> f32 {
            0.54 - 0.46 * (TAU * d / N - 1.0).cos()
        }

        #[inline(always)]
        fn blackman(d: f32) -> f32 {
            0.42 - 0.5 * (TAU * d / N - 1.0).cos() + 0.08 * (TAU * 2.0 * d / N - 1.0).cos()
        }

        match self {
            Self::Hann => hann,
            Self::Hamming => hamming,
            Self::Blackman => blackman,
        }
    }
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
