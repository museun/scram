use std::f32::consts::TAU;

use crate::{Canvas, Frequency, Visual, math::spectro_color};

pub struct SpecCircular;

impl Visual for SpecCircular {
    #[profiling::function]
    fn draw(
        &mut self,
        left: &[Frequency],
        right: &[Frequency],
        _dt: f32,
        renderer: &mut impl Canvas,
    ) {
        let width = renderer.width() as i32;
        let height = renderer.height() as i32;
        let cx = width / 2;
        let cy = height / 2;

        let total = left.len();

        let max_radius = (width.min(height) as f32 / 2.0) * 1.3;
        let base_radius = max_radius * 0.1;

        let max = 1.0;

        for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
            let angle = (i as f32 / total as f32) * TAU; // + (2.0 * dt);

            let value = (l.value + r.value) / 2.0;
            let peak = (l.peak + r.peak) / 2.0;

            let norm = (value / max).clamp(0.0, 1.0);
            let current_radius = base_radius + norm * (max_radius - base_radius);

            let color = (peak / max).clamp(0.0, 1.0);
            let color = spectro_color(color);

            let (sin, cos) = angle.sin_cos();
            let x = (cx as f32 + current_radius * cos) as i32;
            let y = (cy as f32 + current_radius * sin) as i32;

            renderer.put(x, y, color);
        }
    }
}
