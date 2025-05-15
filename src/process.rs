use std::time::Instant;

pub mod config;
use config::*;

mod buffer;
pub use buffer::{Buffer, Source};

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Frequency {
    pub value: f32,
    pub peak: f32,
    pub ts: Instant,
}

impl Frequency {
    fn empty() -> Self {
        Self {
            value: 0.0,
            peak: 0.0,
            ts: Instant::now(),
        }
    }
}

#[derive(Clone)]
struct Channel {
    fft_input: Box<[f32]>,
    fft_magnitudes: Box<[f32]>,

    band_magnitudes: Vec<f32>,
    smoothed_band_magnitudes: Vec<f32>,
    frequencies: Vec<Frequency>,
}

impl Channel {
    fn empty(size: usize) -> Self {
        Self {
            fft_input: vec![0.0; size].into_boxed_slice(),
            fft_magnitudes: vec![0.0; size / 2 + 1].into_boxed_slice(),
            band_magnitudes: Vec::new(),
            smoothed_band_magnitudes: Vec::new(),
            frequencies: Vec::new(),
        }
    }
}

pub struct Processor {
    config: Config,
    sample_rate: u32,

    left: Channel,
    right: Channel,

    last_update: Instant,
    sample_size: usize,
}

impl Processor {
    pub const MIN_SAMPLE_SIZE: usize = 32;
    pub const MAX_SAMPLE_SIZE: usize = 4096;

    pub fn new(sample_rate: u32, sample_size: usize, config: Config) -> anyhow::Result<Self> {
        let sample_size = sample_size
            .clamp(Self::MIN_SAMPLE_SIZE, Self::MAX_SAMPLE_SIZE)
            .next_power_of_two();

        Ok(Self {
            config,
            sample_rate,
            left: Channel::empty(sample_size / 2),
            right: Channel::empty(sample_size / 2),
            last_update: Instant::now(),
            sample_size,
        })
    }

    #[profiling::function]
    pub fn update(&mut self, buffer: &mut dyn Buffer) -> bool {
        let samples = {
            profiling::scope!("read samples");
            match buffer.read_samples(self.sample_size) {
                Some(samples) => samples,
                None => return false,
            }
        };

        if samples.len() != self.sample_size {
            return false;
        }

        self.process_samples(samples);
        true
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn set_bands(&mut self, bands: usize) {
        let bar = Frequency::empty();
        for channel in [&mut self.left, &mut self.right] {
            channel.band_magnitudes.resize(bands, 0.0);
            channel.smoothed_band_magnitudes.resize(bands, 0.0);
            channel.smoothed_band_magnitudes.fill(0.0);
            channel.frequencies.clear();
            channel.frequencies.resize(bands, bar);
        }
    }

    pub fn current_frequencies(&self) -> [&[Frequency]; 2] {
        [&self.left.frequencies, &self.right.frequencies]
    }

    #[profiling::function]
    pub fn process_samples(&mut self, samples: &[f32]) {
        let current = Instant::now();
        let dt = current.duration_since(self.last_update).as_secs_f32();
        self.last_update = current;

        let (left, right) = (&mut self.left, &mut self.right);

        preprocess(samples, left, right, &self.config.window, self.sample_size);

        apply_rfft(&mut left.fft_input);
        apply_rfft(&mut right.fft_input);

        calculate_magnitudes(&mut left.fft_magnitudes, &left.fft_input);
        calculate_magnitudes(&mut right.fft_magnitudes, &right.fft_input);

        aggregate_bands(left, self.sample_rate, &self.config.banding);
        aggregate_bands(right, self.sample_rate, &self.config.banding);

        apply_band_smoothing(left, &self.config.band_smoothing);
        apply_band_smoothing(right, &self.config.band_smoothing);

        apply_scaling(left, &self.config.scaling);
        apply_scaling(right, &self.config.scaling);

        apply_peak_smoothing(left, current, dt, &self.config.peak_smoothing);
        apply_peak_smoothing(right, current, dt, &self.config.peak_smoothing);

        // TODO silence detection
    }
}
