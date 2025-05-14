use mars_app::Rgba;

pub fn inverse_lerp(a: f32, b: f32, t: f32) -> f32 {
    (t - a) / (b - a)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn lerp_color(left: Rgba, right: Rgba, t: f32) -> Rgba {
    let [r0, g0, b0, a0] = left.to_float();
    let [r1, g1, b1, a1] = right.to_float();
    Rgba::from_float([
        lerp(r0, r1, t),
        lerp(g0, g1, t),
        lerp(b0, b1, t),
        a0.max(a1),
    ])
}

pub fn gradient(t: f32, steps: u32, style: Style) -> impl DoubleEndedIterator<Item = Rgba> {
    let t = t.clamp(0.0, 1.0) * style.ratio;
    (0..steps).map(move |y| {
        let p = inverse_lerp(0.0, steps as f32, y as f32);
        lerp_color(style.color, style.accent, t * p)
    })
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    pub color: Rgba,
    pub accent: Rgba,
    pub ratio: f32,
}
