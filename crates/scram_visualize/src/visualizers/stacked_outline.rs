use crate::{
    Canvas, Frequency, Visual,
    math::{Axis, Direction, gradient},
    surface::Style,
};

pub struct StackedOutline {
    left: Style,
    right: Style,
    axis: Axis,
}

impl Visual for StackedOutline {
    #[profiling::function]
    fn draw(
        &mut self,
        left: &[Frequency],
        right: &[Frequency],
        _dt: f32,
        canvas: &mut impl Canvas,
    ) {
        for (pos, freq) in left.iter().enumerate().map(|(p, b)| (p as i32, b)) {
            self.draw_outline(freq, self.left, pos, Direction::Up, canvas);
        }
        for (pos, freq) in right.iter().enumerate().map(|(p, b)| (p as i32, b)) {
            self.draw_outline(freq, self.right, pos, Direction::Down, canvas);
        }
    }
}

impl StackedOutline {
    pub fn new(left: Style, right: Style) -> Self {
        Self {
            left,
            right,
            axis: Axis::Vertical,
        }
    }

    fn draw_outline(
        &self,
        bar: &Frequency,
        style: Style,
        offset: i32,
        direction: Direction,
        canvas: &mut impl Canvas,
    ) {
        let main = self.axis.main((canvas.width(), canvas.height()));

        let center = main / 2;
        let v = bar.value;

        let lenf = (v * center as f32).max(0.0).min(center as f32);
        let len = lenf.round() as u32;

        let end = if direction.is_down() {
            center.saturating_add(len)
        } else {
            center.saturating_sub(len)
        };

        let mut iter = gradient(bar.value, len, style);
        let v = if direction.is_down() {
            iter.next()
        } else {
            iter.next_back()
        };

        if let Some(v) = v {
            let (x, y) = self.axis.pack(end as i32, offset);
            canvas.put(x, y, v);
        }
    }
}
