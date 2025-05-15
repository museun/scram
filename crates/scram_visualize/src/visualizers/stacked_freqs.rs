use crate::{Canvas, Frequency, Visual, surface::Rgba};

pub struct StackedFreqs;

impl Visual for StackedFreqs {
    #[profiling::function]
    fn draw(
        &mut self,
        left: &[Frequency],
        right: &[Frequency],
        _dt: f32,
        renderer: &mut impl Canvas,
    ) {
        let max = 1.0;

        let width = renderer.width() as i32;
        let height = renderer.height() as i32;
        let height = height / 4;

        let w = (width as f32 / left.len() as f32).max(1.0);

        let left_color = Rgba::new(0, 150, 255, 255);
        let right_color = Rgba::new(255, 100, 0, 255);

        for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
            let cx = (i as f32 * w + w / 2.0) as i32;
            if cx < 0 || cx > width {
                continue;
            }
            let lvn = (l.value / max).clamp(0.0, 1.0);
            let lvh = (lvn * height as f32) as i32;

            let rvn = (r.value / max).clamp(0.0, 1.0);
            let rvh = (rvn * height as f32 * 0.6) as i32;

            let offset = cx + (w / 4.0) as i32;
            if offset < 0 || offset >= width {
                continue;
            }

            for y in 0..lvh.min(height) {
                let y = (height * 4) - 1 - y;
                renderer.put(cx, y, left_color);
            }
            for y in 0..rvh.min(height) {
                let y = (height * 4) - 1 - y;
                renderer.put(offset, y, right_color);
            }
        }
    }
}
