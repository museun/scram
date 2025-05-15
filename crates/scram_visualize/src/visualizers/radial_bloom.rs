use std::f32::consts::TAU;

use crate::{
    Canvas, Frequency, Visual,
    math::{lerp_color, spectro_color},
};

pub struct RadialBloom;

impl Visual for RadialBloom {
    #[profiling::function]
    fn draw(
        &mut self,
        left: &[Frequency],
        right: &[Frequency],
        dt: f32,
        renderer: &mut impl Canvas,
    ) {
        let width = renderer.width() as i32;
        let height = renderer.height() as i32;

        let cx = width / 2;
        let cy = height / 2;

        let energy = left
            .iter()
            .chain(right.iter())
            .map(|c| c.value)
            .sum::<f32>();

        let avg = energy / left.len() as f32;
        let max = 4.0 * left.len() as f32;
        let norm = (avg / max).clamp(0.0, 1.0);

        let radius = (width.min(height) as f32 / 2.0) * 0.5;
        let current = norm.powf(0.5) * radius;

        let points = 500 + (norm * 1e5) as i32; // 220
        for i in 0..points {
            let p = i as usize % left.len();
            let angle = (i as f32 / points as f32) * TAU + (0.7 * dt);
            let (sin, cos) = angle.sin_cos();

            let influence = (left[p].value + right[p].value) / 2.0;
            let modulated = current + influence * (radius * 3.0);

            let (x, y) = (cx as f32 + modulated * cos, cy as f32 + modulated * sin);
            let (x, y) = (x as i32, y as i32);

            let peak = (left[p].peak + right[p].peak) / 2.0;

            let color = lerp_color(spectro_color(influence), spectro_color(peak), modulated);

            if x >= 0 && x < width && y >= 0 && y < height {
                renderer.put(x, y, color);

                let limit = 50.0;
                if modulated < limit {
                    continue;
                }

                if x + 1 < width {
                    renderer.put(x + 1, y, color)
                }
                if x > 0 {
                    renderer.put(x - 1, y, color)
                }
                if y + 1 < height {
                    renderer.put(x, y + 1, color)
                }
                if y > 0 {
                    renderer.put(x, y - 1, color)
                }
            }
        }
    }
}
