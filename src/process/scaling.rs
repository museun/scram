use super::{Channel, Config, Scaling};

#[profiling::function]
pub fn apply_scaling(channel: &mut Channel, config: &Config) {
    match config.scaling {
        Scaling::Logarithimic => {
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
