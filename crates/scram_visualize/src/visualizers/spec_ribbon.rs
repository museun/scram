use crate::{
    Canvas, Frequency, Visual,
    math::{lerp, lerp_color, spectro_color},
};

pub struct SpecRibbon;

impl Visual for SpecRibbon {
    #[profiling::function]
    fn draw(
        &mut self,
        left: &[Frequency],
        right: &[Frequency],
        _dt: f32,
        renderer: &mut impl Canvas,
    ) {
        const TRAIL_DURATION: f32 = 0.2;
        const TRAIL_POINTS: i32 = 10;
        #[allow(dead_code)]
        const SPEED: f32 = 50.0;
        #[allow(dead_code)]
        const AMPLITUDE: f32 = 200.0;
        #[allow(dead_code)]
        const OSCILLATOR: f32 = 0.5;

        let width = renderer.width() as i32;
        let height = renderer.height() as i32;
        let total = left.len();

        let max = 1.0;
        let max_y = height - 1;

        for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
            let x = (i as f32 / total as f32 * width as f32) as i32;
            if x < 0 || x >= width {
                continue;
            }

            let value = (l.value + r.value) / 2.0;
            // let peak = (l.peak + r.peak) / 2.0;

            let y = (value / max).clamp(0.0, 1.0);
            let y = max_y - (y * height as f32) as i32;

            let ratio = i as f32 / total as f32;
            let velocity = lerp(50.0, -50.0, ratio);

            let color = (value / max).clamp(0.0, 1.0);
            let color = spectro_color(color);

            if y >= 0 && y <= max_y {
                renderer.put(x, y, color);
            }

            for j in 1..=TRAIL_POINTS {
                let dt_offset = j as f32 * (TRAIL_DURATION / TRAIL_POINTS as f32);
                let x_offset = (velocity * dt_offset) as i32;
                let x = x - x_offset;

                let fade = j as f32 / TRAIL_POINTS as f32;
                let color = lerp_color(color, color, fade);

                if x >= 0 && x < width && y >= 0 && y < height {
                    renderer.put(x, y, color);
                }
            }
        }
    }
}
