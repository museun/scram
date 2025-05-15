use super::{BandSmoothing, Channel};

#[profiling::function]
pub fn apply_band_smoothing(channel: &mut Channel, config: &BandSmoothing) {
    let num_bands = channel.band_magnitudes.len();
    let smoothed = &mut channel.smoothed_band_magnitudes;
    let magnitudes = &mut channel.band_magnitudes;

    match config {
        _ if num_bands == 0 => return,

        BandSmoothing::None => {
            smoothed[0..num_bands].copy_from_slice(&magnitudes[0..num_bands]);
        }

        BandSmoothing::Exponential { factor } => {
            let factor = factor.clamp(0.0, 1.0);
            for i in 0..num_bands {
                let scale = (1.0 - factor) * smoothed[i.saturating_sub(1)];
                smoothed[i] = factor * magnitudes[i] + scale;
            }
        }

        &BandSmoothing::MovingAverage { window_size } => {
            let window_size = window_size.max(1);
            for (i, sample) in smoothed.iter_mut().enumerate() {
                let start = i.saturating_sub(window_size / 2);
                let end = (i + window_size / 2 + 1).min(num_bands);

                let count = end.saturating_sub(start);
                let mag = magnitudes[start..end].iter().sum::<f32>();

                let norm = if count > 0 { 1.0 } else { 0.0 };
                *sample = mag / count as f32 * norm;
            }
        }
    }

    for mag in smoothed {
        *mag = mag.max(0.0)
    }
}
