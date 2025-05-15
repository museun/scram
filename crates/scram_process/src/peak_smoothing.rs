use std::time::Instant;

use super::{Channel, PeakSmoothing};

#[profiling::function]
pub fn apply_peak_smoothing(
    channel: &mut Channel,
    current: Instant,
    dt: f32,
    config: &PeakSmoothing,
) {
    for (bar, &band) in channel.frequencies.iter_mut().zip(&channel.band_magnitudes) {
        if band > bar.value {
            let attack = config.attack_rate * dt;
            bar.value = (bar.value + attack).min(band);
            bar.peak = bar.value;
            bar.ts = current;
        } else if band < bar.value {
            let decay = config.decay_rate * dt;
            bar.value = (bar.value - decay).max(band);

            let elapsed = current.duration_since(bar.ts).as_secs_f32();
            let progress = (elapsed / config.decay_limit).min(1.0);
            let target = bar.peak * (1.0 - progress);
            bar.value = bar.value.max(target).max(band).clamp(0.0, 1.0);

            if bar.value < config.peak_threshold {
                bar.value = 0.0;
                bar.peak = 0.0;
            }
        }

        bar.value = bar.value.clamp(0.0, 1.0);
        bar.peak = bar.peak.clamp(0.0, 1.0)
    }
}
