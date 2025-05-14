use super::{BandSmoothing, Channel};

#[profiling::function]
pub fn apply_band_smoothing(channel: &mut Channel, config: &BandSmoothing) {
    let num_bands = channel.band_magnitudes.len();
    let smoothed = &mut channel.smoothed_band_magnitudes;
    let magnitudes = &mut channel.band_magnitudes;

    match config {
        _ if num_bands == 0 => return,

        BandSmoothing::Exponential { factor } => {
            let factor = factor.clamp(0.0, 1.0);
            for i in 0..num_bands {
                let scale = (1.0 - factor) * smoothed[i.saturating_sub(1)];
                smoothed[i] = factor * magnitudes[i] + scale;
            }
        }

        BandSmoothing::MovingAverage { window_size } => {
            let window_size = window_size.max(1.0).round() as usize;
            for (i, sample) in smoothed.iter_mut().enumerate() {
                let start = i.saturating_sub(window_size / 2);
                let end = (i + window_size / 2 + 1).min(num_bands);
                let count = end - start;
                let mag = magnitudes[start..end].iter().sum::<f32>();
                let norm = (count > 0) as i32 as f32;
                *sample = mag / count as f32 * norm;
            }
        }
    }

    for mag in smoothed {
        *mag = mag.max(0.0)
    }
}
