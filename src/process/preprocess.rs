use super::{Channel, SAMPLE_SIZE, Window};

#[profiling::function]
pub fn preprocess(
    samples: &[f32; SAMPLE_SIZE],
    left: &mut Channel,
    right: &mut Channel,
    config: &Window,
) {
    let f = config.apply();
    for (i, chunk) in samples.chunks_exact(2).enumerate() {
        let t = f(i as f32);
        let &[l, r] = chunk else { unreachable!() };
        left.fft_input[i] = l * t;
        right.fft_input[i] = r * t;
    }
}
