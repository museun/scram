use core::f32;
use std::f32::consts::TAU;

use mars_app::{Position, Rgba};

use crate::{
    math::{inverse_lerp, lerp, lerp_color},
    process::Frequency,
};

use super::half_block::HalfBlockRenderer;

pub fn draw_stacked_freqs(
    left: &[Frequency],
    right: &[Frequency],
    dt: f32,
    renderer: &mut HalfBlockRenderer,
) {
    let max = 1.0;

    let width = renderer.width() as i32;
    let height = renderer.height() as i32 / 4;

    let w = (width as f32 / left.len() as f32).max(1.0);

    let left_color = Rgba(0, 150, 255, 255);
    let right_color = Rgba(255, 100, 0, 255);

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
            renderer.put(Position::new(cx, y), left_color);
        }
        for y in 0..rvh.min(height) {
            let y = (height * 4) - 1 - y;
            renderer.put(Position::new(offset, y), right_color);
        }
    }
}

pub fn draw_spec_slice(
    left: &[Frequency],
    right: &[Frequency],
    _dt: f32,
    renderer: &mut HalfBlockRenderer,
) {
    let width = renderer.width() as i32;
    let w = (width as f32 / left.len() as f32).min(1.0);

    for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
        let x = (i as f32 * w) as i32;
        let v = (l.value + r.value) / 2.0;
        let color = gradient_color(v); // why is 'v' 50% too small here?
        for dx in 0..(w.ceil() as i32) {
            for y in 0..3 {
                renderer.put(Position::new(x + dx, y), color);
            }
        }
    }
}

pub fn draw_radial_bloom(
    left: &[Frequency],
    right: &[Frequency],
    dt: f32,
    renderer: &mut HalfBlockRenderer,
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

        let color = lerp_color(gradient_color(influence), gradient_color(peak), modulated);

        if x >= 0 && x < width && y >= 0 && y < height {
            renderer.put(Position::new(x, y), color);

            let limit = 50.0;
            if modulated < limit {
                continue;
            }

            if x + 1 < width {
                renderer.put(Position::new(x + 1, y), color)
            }
            if x > 0 {
                renderer.put(Position::new(x - 1, y), color)
            }
            if y + 1 < height {
                renderer.put(Position::new(x, y + 1), color)
            }
            if y > 0 {
                renderer.put(Position::new(x, y - 1), color)
            }
        }
    }
}

pub fn draw_spec_circular(
    left: &[Frequency],
    right: &[Frequency],
    dt: f32,
    renderer: &mut HalfBlockRenderer,
) {
    let total = left.len();
    let w = renderer.width() as i32;
    let h = renderer.height() as i32;
    let cx = w / 2;
    let cy = h / 2;

    let max_radius = (w.min(h) as f32 / 2.0) * 1.3;
    let base_radius = max_radius * 0.1;

    let max = 1.0;

    for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
        let angle = (i as f32 / total as f32) * TAU; // + (2.0 * dt);

        let value = (l.value + r.value) / 2.0;
        let peak = (l.peak + r.peak) / 2.0;

        let norm = (value / max).clamp(0.0, 1.0);
        let current_radius = base_radius + norm * (max_radius - base_radius);

        let color = (peak / max).clamp(0.0, 1.0);
        let color = gradient_color(color);

        let (sin, cos) = angle.sin_cos();
        let x = (cx as f32 + current_radius * cos) as i32;
        let y = (cy as f32 + current_radius * sin) as i32;

        renderer.put(Position::new(x, y), color);
    }
}

pub fn kinetic(left: &[Frequency], right: &[Frequency], dt: f32, renderer: &mut HalfBlockRenderer) {
    const TRAIL_DURATION: f32 = 0.2;
    const TRAIL_POINTS: i32 = 10;
    const SPEED: f32 = 50.0;
    const AMPLITUDE: f32 = 200.0;
    const OSCILLATOR: f32 = 0.5;

    let total = left.len();
    let w = renderer.width() as i32;
    let h = renderer.height() as i32;

    let max = 1.0;
    let max_y = h - 1;

    for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
        let x = (i as f32 / total as f32 * w as f32) as i32;
        if x < 0 || x >= w {
            continue;
        }

        let value = (l.value + r.value) / 2.0;
        // let peak = (l.peak + r.peak) / 2.0;

        let y = (value / max).clamp(0.0, 1.0);
        let y = max_y - (y * h as f32) as i32;

        let ratio = i as f32 / total as f32;
        let velocity = lerp(50.0, -50.0, ratio);

        let color = (value / max).clamp(0.0, 1.0);
        let color = gradient_color(color);

        if y >= 0 && y <= max_y {
            renderer.put(Position::new(x, y), color);
        }

        for j in 1..=TRAIL_POINTS {
            let dt_offset = j as f32 * (TRAIL_DURATION / TRAIL_POINTS as f32);
            let x_offset = (velocity * dt_offset) as i32;
            let x = x - x_offset;

            let fade = j as f32 / TRAIL_POINTS as f32;
            let color = lerp_color(color, color, fade);

            if x >= 0 && x < w && y >= 0 && y < h {
                renderer.put(Position::new(x, y), color);
            }
        }
    }
}

fn gradient_color(t: f32) -> Rgba {
    const COLOR_0: Rgba = Rgba::hex("#303066");
    const COLOR_1: Rgba = Rgba::hex("#0000FF");
    const COLOR_2: Rgba = Rgba::hex("#00FFFF");
    const COLOR_3: Rgba = Rgba::hex("#00FF00");
    const COLOR_4: Rgba = Rgba::hex("#FFFF00");
    const COLOR_5: Rgba = Rgba::hex("#FF0000");
    const COLOR_6: Rgba = Rgba::hex("#FFFFFF");

    match t {
        ..0.20 => lerp_color(COLOR_0, COLOR_1, inverse_lerp(0.0, 0.2, t)),
        ..0.35 => lerp_color(COLOR_1, COLOR_2, inverse_lerp(0.2, 0.35, t)),
        ..0.50 => lerp_color(COLOR_2, COLOR_3, inverse_lerp(0.35, 0.5, t)),
        ..0.60 => lerp_color(COLOR_3, COLOR_4, inverse_lerp(0.5, 0.6, t)),
        ..0.80 => lerp_color(COLOR_4, COLOR_5, inverse_lerp(0.7, 0.8, t)),
        t => lerp_color(COLOR_5, COLOR_6, inverse_lerp(0.8, 1.0, t)),
    }
}
