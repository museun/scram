use crate::{Canvas, Frequency, Visual, math::spectro_color};

pub struct SpecSlice;

impl Visual for SpecSlice {
    #[profiling::function]
    fn draw(
        &mut self,
        left: &[Frequency],
        right: &[Frequency],
        _dt: f32,
        renderer: &mut impl Canvas,
    ) {
        let width = (renderer.width() as f32 / left.len() as f32).min(1.0);

        let freqs = left
            .iter()
            .zip(right.iter())
            .map(|(l, r)| (l.value + r.value) / 2.0)
            .map(spectro_color);

        for (i, color) in freqs.enumerate() {
            let x = (i as f32 * width) as i32;
            for dx in 0..(width.ceil() as i32) {
                for y in 0..3 {
                    renderer.put(x + dx, y, color);
                }
            }
        }
    }
}
