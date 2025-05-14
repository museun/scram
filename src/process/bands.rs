use crate::process::Banding;

use super::{Binning, Channel};

#[profiling::function]
pub fn aggregate_bands(channel: &mut Channel, sample_rate: u32, binning: &Binning) {
    channel.band_magnitudes.fill(0.0);

    let num_bins = channel.fft_magnitudes.len();
    let nyquist_freq = sample_rate as f32 / 2.0;
    let hz_per = nyquist_freq / (num_bins as f32 - 1.0);

    let opts = Opts {
        num_bins,
        num_bands: channel.frequencies.len(),
        min_freq: binning.frequency_cutoff.low,
        max_freq: binning.frequency_cutoff.high,
        hz_per,
    };

    match binning.banding {
        Banding::Linear => apply_linear(channel, opts),
        Banding::Logarithmic => apply_logarithmic(channel, opts),
        Banding::Bark => apply_bark(channel, opts),
        Banding::Mel => apply_mel(channel, opts),
    }
}

fn apply_sample(channel: &mut Channel, sample: usize, start_hz: f32, end_hz: f32, total: usize) {
    let start = (start_hz.floor() as usize).max(1).min(total);
    let end = (end_hz.ceil() as usize).min(total);

    let count = end - start;
    let mag = channel.fft_magnitudes[start..end].iter().sum::<f32>();
    let norm = (count > 0) as i32 as f32;
    channel.band_magnitudes[sample] = mag / total as f32 * norm;
}

struct Opts {
    num_bins: usize,
    num_bands: usize,
    min_freq: f32,
    max_freq: f32,
    hz_per: f32,
}

#[profiling::function]
fn apply_linear(channel: &mut Channel, opts: Opts) {
    for sample in 0..opts.num_bands {
        apply_sample(
            channel,
            sample,
            (opts.min_freq + sample as f32 * opts.hz_per) / opts.hz_per,
            (opts.min_freq + (sample + 1) as f32 * opts.hz_per) / opts.hz_per,
            opts.num_bins,
        );
    }
}

#[profiling::function]
fn apply_logarithmic(channel: &mut Channel, opts: Opts) {
    let base = (opts.max_freq / opts.min_freq).powf(1.0 / opts.num_bands as f32);
    for sample in 0..opts.num_bands {
        apply_sample(
            channel,
            sample,
            (opts.min_freq * base.powf(sample as f32)) / opts.hz_per,
            (opts.min_freq * base.powf((sample + 1) as f32)) / opts.hz_per,
            opts.num_bins,
        );
    }
}

#[profiling::function]
fn apply_mel(channel: &mut Channel, opts: Opts) {
    let hz_to_mel = |hz: f32| 2595.0 * (1.0 + hz / 700.0).log10();
    let mel_to_hz = |mel: f32| 700.0 * (10.0_f32.powf(mel / 2595.0) - 1.0);

    let min_mel = hz_to_mel(opts.min_freq);
    let max_mel = hz_to_mel(opts.max_freq);
    let mel_per = (max_mel - min_mel) / opts.num_bands as f32;

    for sample in 0..opts.num_bands {
        apply_sample(
            channel,
            sample,
            mel_to_hz(min_mel + sample as f32 * mel_per) / opts.hz_per,
            mel_to_hz(min_mel + (sample + 1) as f32 * mel_per) / opts.hz_per,
            opts.num_bins,
        );
    }
}

#[profiling::function]
fn apply_bark(channel: &mut Channel, opts: Opts) {
    let hz_to_bark = |hz: f32| 7.0 * ((hz / 600.0) + ((hz / 600.0).powi(2) + 1.0).sqrt()).ln();
    let bark_to_hz = |bark: f32| 600.0 * (bark / 7.0).sinh();

    let min_bark = hz_to_bark(opts.min_freq);
    let max_bark = hz_to_bark(opts.max_freq);
    let bark_per = (max_bark - min_bark) / opts.num_bands as f32;

    for sample in 0..opts.num_bands {
        apply_sample(
            channel,
            sample,
            bark_to_hz(min_bark + sample as f32 * bark_per) / opts.hz_per,
            bark_to_hz(min_bark + (sample + 1) as f32 * bark_per) / opts.hz_per,
            opts.num_bins,
        );
    }
}
