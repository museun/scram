use crate::process::FrequencyScale;

use super::{Banding, Channel};

#[profiling::function]
pub fn aggregate_bands(channel: &mut Channel, sample_rate: u32, banding: &Banding) {
    channel.band_magnitudes.fill(0.0);

    let opts = Opts {
        num_bands: channel.band_magnitudes.len(),
        min_freq: banding.frequency_cutoff.low,
        max_freq: banding.frequency_cutoff.high,
        hz_per: (sample_rate as f32 / 2.0) / (channel.fft_magnitudes.len() as f32 - 1.0),
    };

    match banding.scale {
        FrequencyScale::Linear => apply_linear(channel, opts),
        FrequencyScale::Logarithmic => apply_logarithmic(channel, opts),
        FrequencyScale::Bark => apply_bark(channel, opts),
        FrequencyScale::Mel => apply_mel(channel, opts),
    }
}

struct Opts {
    num_bands: usize,
    min_freq: f32,
    max_freq: f32,
    hz_per: f32,
}

#[profiling::function]
fn apply_linear(channel: &mut Channel, opts: Opts) {
    let total = channel.fft_magnitudes.len();
    let ratio = (opts.max_freq - opts.min_freq) / opts.num_bands as f32;

    for sample in 0..opts.num_bands {
        // let start_hz = (opts.min_freq + sample as f32 * opts.hz_per) / opts.hz_per;
        // let end_hz = (opts.min_freq + (sample + 1) as f32 * opts.hz_per) / opts.hz_per;

        let start_hz = opts.min_freq + sample as f32 * ratio;
        let end_hz = opts.min_freq + (sample + 1) as f32 * ratio;

        let start_hz = start_hz / opts.hz_per;
        let end_hz = end_hz / opts.hz_per;

        let start = (start_hz.floor() as usize).clamp(0, total);
        let end = (end_hz.ceil() as usize).min(total);

        let count = end.saturating_sub(start);
        let mag = channel.fft_magnitudes[start..end].iter().sum::<f32>();

        let norm = if count > 0 { mag / total as f32 } else { 0.0 };
        channel.band_magnitudes[sample] = norm;
    }
}

#[profiling::function]
fn apply_logarithmic(channel: &mut Channel, opts: Opts) {
    let total = channel.fft_magnitudes.len();

    let base = (opts.max_freq / opts.min_freq).powf(1.0 / opts.num_bands as f32);
    for sample in 0..opts.num_bands - 1 {
        let start_hz = opts.min_freq * base.powf(sample as f32);
        let end_hz = opts.min_freq * base.powf((sample + 1) as f32);

        let start_freq = start_hz / opts.hz_per;
        let end_freq = end_hz / opts.hz_per;

        let start = (start_freq.floor() as usize).clamp(0, total);
        let end = (end_freq.ceil() as usize).min(total);

        let count = end.saturating_sub(start);
        let mag = if count > 0 {
            channel.fft_magnitudes[start..end].iter().sum::<f32>()
        } else {
            0.0
        };

        let norm = if count > 0 { mag / total as f32 } else { 0.0 };
        channel.band_magnitudes[sample] = norm;
    }

    let sample = opts.num_bands.saturating_sub(1);
    let start_hz = opts.min_freq * base.powf(sample as f32);
    let end_hz = opts.max_freq;

    let start_freq = start_hz / opts.hz_per;
    let end_freq = end_hz / opts.hz_per;
    let start = (start_freq.floor() as usize).clamp(0, total);
    let end = (end_freq.ceil() as usize).min(total);

    let count = end.saturating_sub(start);
    let mag = if count > 0 {
        channel.fft_magnitudes[start..end].iter().sum::<f32>()
    } else {
        0.0
    };

    let norm = if count > 0 { mag / total as f32 } else { 0.0 };
    channel.band_magnitudes[sample] = norm;
}

#[profiling::function]
fn apply_mel(channel: &mut Channel, opts: Opts) {
    fn hz_to_mel(hz: f32) -> f32 {
        2595.0 * (1.0 + hz / 700.0).log10()
    }
    fn mel_to_hz(mel: f32) -> f32 {
        700.0 * (10.0_f32.powf(mel / 2595.0) - 1.0)
    }

    let total = channel.fft_magnitudes.len();

    let min_mel = hz_to_mel(opts.min_freq);
    let max_mel = hz_to_mel(opts.max_freq);
    let mel_per = (max_mel - min_mel) / opts.num_bands as f32;

    for sample in 0..opts.num_bands {
        let start_hz = mel_to_hz(min_mel + sample as f32 * mel_per) / opts.hz_per;
        let end_hz = mel_to_hz(min_mel + (sample + 1) as f32 * mel_per) / opts.hz_per;

        let start = (start_hz.floor() as usize).clamp(0, total);
        let end = (end_hz.ceil() as usize).min(total);

        let count = end.saturating_sub(start);
        let mag = channel.fft_magnitudes[start..end].iter().sum::<f32>();
        let norm = if count > 0 { 1.0 } else { 0.0 };
        channel.band_magnitudes[sample] = mag / total as f32 * norm;
    }
}

#[profiling::function]
fn apply_bark(channel: &mut Channel, opts: Opts) {
    fn hz_to_bark(hz: f32) -> f32 {
        7.0 * ((hz / 600.0) + ((hz / 600.0).powi(2) + 1.0).sqrt()).ln()
    }
    fn bark_to_hz(bark: f32) -> f32 {
        600.0 * (bark / 7.0).sinh()
    }

    let total = channel.fft_magnitudes.len();
    let min_bark = hz_to_bark(opts.min_freq);
    let max_bark = hz_to_bark(opts.max_freq);
    let bark_per = (max_bark - min_bark) / opts.num_bands as f32;

    for sample in 0..opts.num_bands {
        let start_hz = bark_to_hz(min_bark + sample as f32 * bark_per) / opts.hz_per;
        let end_hz = bark_to_hz(min_bark + (sample + 1) as f32 * bark_per) / opts.hz_per;
        let start = (start_hz.floor() as usize).clamp(0, total);
        let end = (end_hz.ceil() as usize).min(total);

        let count = end.saturating_sub(start);
        let mag = channel.fft_magnitudes[start..end].iter().sum::<f32>();
        let norm = if count > 0 { 1.0 } else { 0.0 };
        channel.band_magnitudes[sample] = mag / total as f32 * norm;
    }
}
