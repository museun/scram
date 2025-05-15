use crate::{
    Canvas, Frequency, Visual,
    math::{Axis, Direction, gradient},
    surface::Style,
};

pub struct StackedChannels {
    left: Style,
    right: Style,
    axis: Axis,
}

impl Visual for StackedChannels {
    #[profiling::function]
    fn draw(
        &mut self,
        left: &[Frequency],
        right: &[Frequency],
        _dt: f32,
        canvas: &mut impl Canvas,
    ) {
        for (pos, freq) in left.iter().enumerate().map(|(p, b)| (p as i32, b)) {
            self.draw_bar(freq, self.left, pos, Direction::Up, canvas);
        }
        for (pos, freq) in right.iter().enumerate().map(|(p, b)| (p as i32, b)) {
            self.draw_bar(freq, self.right, pos, Direction::Down, canvas);
        }
    }
}

impl StackedChannels {
    pub fn new(left: Style, right: Style) -> Self {
        Self {
            left,
            right,
            axis: Axis::Vertical,
        }
    }

    fn draw_bar(
        &self,
        freq: &Frequency,
        style: Style,
        offset: i32,
        direction: Direction,
        canvas: &mut impl Canvas,
    ) {
        let main = self.axis.main((canvas.width(), canvas.height()));

        let center = main / 2;
        let v = freq.value;

        let lenf = (v * center as f32).max(0.0).min(center as f32);
        let len = (lenf.round() as u32).max(1);

        let end = if direction.is_down() {
            center.saturating_add(len).saturating_add(1)
        } else {
            center.saturating_sub(len).saturating_sub(1)
        };

        let next = if direction.is_down() {
            <_ as Iterator>::next
        } else {
            <_ as DoubleEndedIterator>::next_back
        };

        let mut iter = gradient(v, len, style);
        let vertical_gradient = std::iter::from_fn(move || next(&mut iter));

        for (p, v) in (center.min(end)..center.max(end)).zip(vertical_gradient) {
            let (x, y) = self.axis.pack(p as i32, offset);
            canvas.put(x, y, v);
        }
    }
}
