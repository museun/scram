use mars_app::{Axis, BlendMode, Drawable, Renderer, Rgba, Size};

use crate::{
    math::{Style, gradient},
    process::Frequency,
};

mod half_block;
use half_block::HalfBlockRenderer;

mod visualizers;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
enum VisualStyle {
    #[default]
    Bar,
    Outline,
}

pub struct Visualizer {
    renderer: HalfBlockRenderer,
    left_style: Style,
    right_style: Style,
    style: VisualStyle,
    zoom: f32,
}

impl Default for Visualizer {
    fn default() -> Self {
        Self::new(Self::DEFAULT_LEFT_STYLE, Self::DEFAULT_RIGHT_STYLE)
    }
}

impl Visualizer {
    pub const DEFAULT_LEFT_STYLE: Style = Style {
        color: Rgba::hex("#933"),
        accent: Rgba::hex("#909"),
        ratio: 1.5,
    };

    pub const DEFAULT_RIGHT_STYLE: Style = Style {
        color: Rgba::hex("#339"),
        accent: Rgba::hex("#909"),
        ratio: 1.5,
    };

    pub fn new(left_style: Style, right_style: Style) -> Self {
        Self {
            renderer: HalfBlockRenderer::new(Size::ZERO, Axis::Vertical),
            left_style,
            right_style,
            style: VisualStyle::Bar,
            zoom: 1.3,
        }
    }

    pub fn resize(&mut self, size: Size) {
        self.renderer.resize(size)
    }

    pub fn axis(&self) -> Axis {
        self.renderer.axis()
    }

    #[profiling::function]
    pub fn draw(
        &mut self,
        left: &[Frequency],
        right: &[Frequency],
        dt: f32,
        renderer: &mut impl Renderer,
    ) {
        if left.is_empty() || right.is_empty() {
            return;
        }

        // for reference
        visualizers::draw_spec_slice(left, right, dt, &mut self.renderer);

        // TODO if vertical renderer then we should draw the bars in reverse order
        let draw = match self.style {
            VisualStyle::Bar => Self::draw_bar,
            VisualStyle::Outline => Self::draw_outline,
        };
        for (pos, bar) in left.iter().enumerate().map(|(p, b)| (p as i32, b)) {
            draw(self, bar, self.left_style, pos, Direction::Up);
        }
        for (pos, bar) in right.iter().enumerate().map(|(p, b)| (p as i32, b)) {
            draw(self, bar, self.right_style, pos, Direction::Down);
        }

        // visualizers::kinetic(left, right, dt, &mut self.renderer);
        // visualizers::draw_stacked_freqs(left, right, dt, &mut self.renderer);
        // visualizers::draw_spec_circular(left, right, dt, &mut self.renderer);
        // visualizers::draw_radial_bloom(left, right, dt, &mut self.renderer);

        self.renderer.render(renderer, BlendMode::Replace);
        self.renderer.clear();
    }

    fn draw_bar(&mut self, bar: &Frequency, style: Style, offset: i32, direction: Direction) {
        let axis = self.axis();
        let main = axis.main(self.renderer.dimensions());

        let center = main / 2;
        let v = bar.value * self.zoom;

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

        let mut iter = gradient(bar.value, len, style);
        let vertical_gradient = std::iter::from_fn(move || next(&mut iter));

        for (p, v) in (center.min(end)..center.max(end)).zip(vertical_gradient) {
            let pos = axis.pack(p as i32, offset);
            self.renderer.put(pos, v);
        }
    }

    fn draw_outline(&mut self, bar: &Frequency, style: Style, offset: i32, direction: Direction) {
        let axis = self.axis();
        let main = axis.main(self.renderer.dimensions());

        let center = main / 2;
        let v = bar.value * self.zoom;

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
            let pos = axis.pack(end as i32, offset);
            self.renderer.put(pos, v);
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Direction {
    Up,
    Down,
}

impl Direction {
    const fn is_down(&self) -> bool {
        matches!(self, Self::Down)
    }
}
