use std::f32::consts::TAU;

use super::{Channel, Window};

#[inline(always)]
fn hann(d: f32, n: f32) -> f32 {
    0.5 * (1.0 - (TAU * d / n - 1.0).cos())
}

#[inline(always)]
fn hamming(d: f32, n: f32) -> f32 {
    0.54 - 0.46 * (TAU * d / n - 1.0).cos()
}

#[inline(always)]
fn blackman(d: f32, n: f32) -> f32 {
    0.42 - 0.5 * (TAU * d / n - 1.0).cos() + 0.08 * (TAU * 2.0 * d / n - 1.0).cos()
}

#[profiling::function]
pub fn preprocess(
    samples: &[f32],
    left: &mut Channel,
    right: &mut Channel,
    config: &Window,
    sample_size: usize,
) {
    let f = match config {
        Window::Hann => hann,
        Window::Hamming => hamming,
        Window::Blackman => blackman,
    };

    let nyquist = sample_size as f32;
    for (i, chunk) in samples.chunks_exact(2).enumerate() {
        let t = f(i as f32, nyquist);
        let &[l, r] = chunk else { unreachable!() };
        left.fft_input[i] = l * t;
        right.fft_input[i] = r * t
    }
}
