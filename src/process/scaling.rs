use super::{Channel, VolumeScale};

#[profiling::function]
pub fn apply_scaling(channel: &mut Channel, config: &VolumeScale) {
    match config {
        VolumeScale::Linear => {
            let max = channel
                .smoothed_band_magnitudes
                .iter()
                .fold(f32::MIN, |a, &c| a.max(c))
                .clamp(0.0, 1.0)
                * 2.0;

            for (smoothed, mag) in channel
                .smoothed_band_magnitudes
                .iter_mut()
                .zip(channel.band_magnitudes.iter_mut())
            {
                *mag = (*smoothed / max).clamp(0.0, 1.0)
            }
        }
        VolumeScale::Logarithimic => {
            let reference = 1.0; // TODO config this

            let min = -60.0;
            let max = 0.0;

            for (smoothed, mag) in channel
                .smoothed_band_magnitudes
                .iter_mut()
                .zip(channel.band_magnitudes.iter_mut())
            {
                let db = if *smoothed > 0.0 {
                    20.0 * (*smoothed / reference).log10()
                } else {
                    min
                };
                let scaled = (db - min) / (max - min);
                *mag = scaled.clamp(0.0, 1.0);
            }
        }
    }
}
