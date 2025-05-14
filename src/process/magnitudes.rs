use super::{Channel, SAMPLE_SIZE};

#[profiling::function]
pub fn calculate_magnitudes(channel: &mut Channel) {
    const NYQUIST: usize = SAMPLE_SIZE / 2;
    const BINS: usize = NYQUIST / 2 + 1;

    channel.fft_magnitudes[0] = channel.fft_input[0].abs();
    for i in 0..BINS - 2 {
        let (re, im) = (
            channel.fft_input[1 + i * 2],
            channel.fft_input[1 + i * 2 + 1],
        );
        channel.fft_magnitudes[i + 1] = (re.abs().powi(2) + im.abs().powi(2)).sqrt();
    }
    channel.fft_magnitudes[BINS - 1] = channel.fft_input[NYQUIST - 1].abs();
}
