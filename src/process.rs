use std::time::Instant;

pub mod config;
use config::*;

mod buffer;
pub use buffer::Buffer;

mod band_smoothing;
use band_smoothing::apply_band_smoothing;

mod bands;
use bands::aggregate_bands;

mod magnitudes;
use magnitudes::calculate_magnitudes;

mod peak_smoothing;
use peak_smoothing::apply_peak_smoothing;

mod preprocess;
use preprocess::preprocess;

mod rfft;
use rfft::apply_rfft;

mod scaling;
use scaling::apply_scaling;

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
    frequencies: Vec<Frequency>,
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
            frequencies: vec![bar; 0],
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
            band.frequencies.clear();
            band.frequencies.resize(bands, bar);
        }
    }

    pub fn current_frequencies(&self) -> (&[Frequency], &[Frequency]) {
        (&self.left.frequencies, &self.right.frequencies)
    }

    #[profiling::function]
    pub fn process_samples(&mut self, samples: &[f32; SAMPLE_SIZE]) {
        let current = Instant::now();
        let dt = current.duration_since(self.last_update).as_secs_f32();
        self.last_update = current;

        let (left, right) = (&mut self.left, &mut self.right);

        preprocess(samples, left, right, &self.config.window);

        apply_rfft(left);
        apply_rfft(right);

        calculate_magnitudes(left);
        calculate_magnitudes(right);

        aggregate_bands(left, self.sample_rate, &self.config.binning);
        aggregate_bands(right, self.sample_rate, &self.config.binning);

        apply_band_smoothing(left, &self.config.band_smoothing);
        apply_band_smoothing(right, &self.config.band_smoothing);

        apply_scaling(left, &self.config.scaling);
        apply_scaling(right, &self.config.scaling);

        apply_peak_smoothing(left, current, dt, &self.config.peak_smoothing);
        apply_peak_smoothing(right, current, dt, &self.config.peak_smoothing);

        // TODO silence detection
    }
}
