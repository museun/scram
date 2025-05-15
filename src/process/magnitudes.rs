#[profiling::function]
pub fn calculate_magnitudes(magnitudes: &mut [f32], raw: &[f32]) {
    let nyquist = raw.len();
    let bins = nyquist / 2 + 1;

    magnitudes[0] = raw[0].abs();
    for i in 0..bins - 2 {
        let (re, im) = (raw[1 + i * 2], raw[1 + i * 2 + 1]);
        magnitudes[i + 1] = (re.abs().powi(2) + im.abs().powi(2)).sqrt();
    }
    magnitudes[bins - 1] = raw[nyquist - 1].abs();
}
