use core::f32;
use std::time::Instant;

pub mod config;
pub use config::*;

mod buffer;
pub use buffer::Buffer;

mod band_smoothing;
mod bands;
mod magnitudes;
mod peak_smoothing;
mod rfft;
mod scaling;

pub const SAMPLE_SIZE: usize = 4096;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Frequency {
    pub value: f32,
    pub peak: f32,
    pub ts: Instant,
}

#[derive(Clone)]
struct Channel {
    fft_input: Box<[f32; SAMPLE_SIZE / 2]>,
    fft_magnitudes: Box<[f32; SAMPLE_SIZE / 2 + 1]>,

    band_magnitudes: Vec<f32>,
    smoothed_band_magnitudes: Vec<f32>,
    bars: Vec<Frequency>,
}

pub struct Processor {
    config: Config,
    sample_rate: u32,

    left: Channel,
    right: Channel,

    last_update: Instant,
}

impl Processor {
    pub fn new(sample_rate: u32, config: Config) -> Self {
        let bar = Frequency {
            value: 0.0,
            peak: 0.0,
            ts: Instant::now(),
        };

        let left = Channel {
            fft_input: Box::from([0.0; SAMPLE_SIZE / 2]),
            fft_magnitudes: Box::from([0.0; SAMPLE_SIZE / 2 + 1]),
            band_magnitudes: vec![0.0; 0],
            smoothed_band_magnitudes: vec![0.0; 0],
            bars: vec![bar; 0],
        };
        let right = left.clone();

        Self {
            config,
            sample_rate,
            left,
            right,
            last_update: Instant::now(),
        }
    }

    #[profiling::function]
    pub fn update(&mut self, buffer: &mut dyn Buffer<SAMPLE_SIZE>) {
        let Some(samples) = buffer.read_samples() else {
            // not enough samples, yet
            return;
        };

        self.process_samples(samples);
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn set_bands(&mut self, bands: usize) {
        let bar = Frequency {
            value: 0.0,
            peak: 0.0,
            ts: Instant::now(),
        };

        for band in [&mut self.left, &mut self.right] {
            band.band_magnitudes.resize(bands, 0.0);
            band.smoothed_band_magnitudes.resize(bands, 0.0);
            band.smoothed_band_magnitudes.fill(0.0);
            band.bars.clear();
            band.bars.resize(bands, bar);
        }
    }

    pub fn current_frequencies(&self) -> (&[Frequency], &[Frequency]) {
        (&self.left.bars, &self.right.bars)
    }

    #[profiling::function]
    pub fn process_samples(&mut self, samples: &[f32; SAMPLE_SIZE]) {
        let current = Instant::now();
        let dt = current.duration_since(self.last_update).as_secs_f32();
        self.last_update = current;

        let (left, right) = (&mut self.left, &mut self.right);
        Self::preprocess(samples, left, right, &self.config);

        Self::apply_rfft(left);
        Self::apply_rfft(right);

        Self::calculate_magnitudes(left);
        Self::calculate_magnitudes(right);

        Self::aggregate_bands(left, self.sample_rate, &self.config.binning);
        Self::aggregate_bands(right, self.sample_rate, &self.config.binning);

        Self::apply_band_smoothing(left, &self.config.band_smoothing);
        Self::apply_band_smoothing(right, &self.config.band_smoothing);

        Self::apply_scaling(left, &self.config.scaling);
        Self::apply_scaling(right, &self.config.scaling);

        Self::apply_peak_smoothing(left, current, dt, &self.config.peak_smoothing);
        Self::apply_peak_smoothing(right, current, dt, &self.config.peak_smoothing);

        // TODO silence detection
    }

    #[profiling::function]
    fn preprocess(
        samples: &[f32; SAMPLE_SIZE],
        left: &mut Channel,
        right: &mut Channel,
        config: &Config,
    ) {
        let f = config.window.apply();
        for (i, chunk) in samples.chunks_exact(2).enumerate() {
            let t = f(i as f32);
            let &[l, r] = chunk else { unreachable!() };
            left.fft_input[i] = l * t;
            right.fft_input[i] = r * t;
        }
    }

    fn apply_rfft(channel: &mut Channel) {
        rfft::apply_rfft(channel);
    }

    fn calculate_magnitudes(channel: &mut Channel) {
        magnitudes::calculate_magnitudes(channel);
    }

    fn aggregate_bands(channel: &mut Channel, sample_rate: u32, binning: &Binning) {
        bands::aggregate_bands(channel, sample_rate, binning);
    }

    fn apply_band_smoothing(channel: &mut Channel, config: &BandSmoothing) {
        band_smoothing::apply_band_smoothing(channel, config);
    }

    fn apply_scaling(channel: &mut Channel, config: &Scaling) {
        scaling::apply_scaling(channel, config);
    }

    fn apply_peak_smoothing(
        channel: &mut Channel,
        current: Instant,
        dt: f32,
        config: &PeakSmoothing,
    ) {
        peak_smoothing::apply_peak_smoothing(channel, current, dt, config);
    }
}
