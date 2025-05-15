use crate::{
    Canvas, Frequency, Visual,
    math::{Position, Size, spectro_color},
    surface::{ResizeMode, Rgba, Surface},
};

pub struct ScrollingSpectro {
    buffer: Surface<Rgba>,
    max_value: f32,
}

impl ScrollingSpectro {
    pub fn new() -> Self {
        Self {
            buffer: Surface::new(Size::ZERO, spectro_color(0.0)),
            max_value: 1.0,
        }
    }

    fn get_color(&self, magnitude: f32) -> Rgba {
        spectro_color((magnitude / self.max_value).clamp(0.0, 1.0))
    }
}

impl Visual for ScrollingSpectro {
    #[profiling::function]
    fn draw(
        &mut self,
        left: &[Frequency],
        right: &[Frequency],
        _dt: f32,
        renderer: &mut impl Canvas,
    ) {
        let total = left.len();

        for y in 0..self.buffer.size().height - 1 {
            self.buffer.scroll_up_copy(y as usize);
        }

        let y = self.buffer.size().height - 1;
        let w = (self.buffer.size().width as f32 / total as f32).max(1.0);
        let offset = w.ceil() as usize;

        for i in 0..total {
            let x = (i as f32 * w) as usize;

            let value = (left[i].value + right[i].value) / 2.0 * 1.3;
            let color = self.get_color(value);

            for dx in 0..offset {
                let x = x + dx;
                self.buffer.set(Position::new(x as i32, y as i32), color);
            }
        }

        for y in 0..self.buffer.size().height {
            for x in 0..self.buffer.size().width {
                renderer.put(x as i32, y as i32, self.buffer[(x, y)]);
            }
        }
    }

    fn resize(&mut self, size: Size) {
        self.buffer.resize(size, ResizeMode::Discard);
    }
}
