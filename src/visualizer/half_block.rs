use mars_app::{
    Axis, BlendMode, Color, Drawable, Pixel, Placer, Position, ResizeMode, Size, Surface,
};

pub struct HalfBlockRenderer {
    size: Size,
    surface: Surface<Color>,
    axis: Axis,
}

impl HalfBlockRenderer {
    pub fn new(size: Size, axis: Axis) -> Self {
        let size = match axis {
            Axis::Horizontal => size * Size::new(2, 1),
            Axis::Vertical => size * Size::new(1, 2),
        };

        Self {
            size,
            surface: Surface::new(size, Color::Default),
            axis,
        }
    }

    pub fn axis(&self) -> Axis {
        self.axis
    }

    pub fn clear(&mut self) {
        self.surface.clear();
    }

    pub fn put(&mut self, pos: Position, color: mars_app::Rgba) {
        self.surface.set(pos, color.into());
    }

    pub fn resize(&mut self, mut size: Size) {
        size *= match self.axis {
            Axis::Horizontal => Size::new(2, 1),
            Axis::Vertical => Size::new(1, 2),
        };
        self.size = size;
        self.surface.resize(size, ResizeMode::Discard)
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn dimensions(&self) -> Size {
        self.size
    }
}

impl Drawable for HalfBlockRenderer {
    #[profiling::function]
    fn draw(&self, placer: &mut dyn Placer, pos: Position, _blend: BlendMode) {
        match self.axis {
            Axis::Horizontal => self.draw_horizontal(placer, pos),
            Axis::Vertical => self.draw_vertical(placer, pos),
        }
    }

    fn size(&self, _input: Size) -> Size {
        self.size
    }
}

impl HalfBlockRenderer {
    // TODO use the axis math so this isn't duplicated
    fn draw_vertical(&self, placer: &mut dyn Placer, pos: Position) {
        const UPPER: char = '▀';
        const LOWER: char = '▄';

        for y1 in 0..(self.size.height as i32 / 2) {
            for x1 in 0..self.size.width as i32 {
                let (x, y) = (pos.x + x1, pos.y + y1);
                let top = self.surface[Position::new(x1, 2 * y1)];
                let bottom = self.surface[Position::new(x1, 2 * y1 + 1)];
                let (ch, fg, bg) = match (top, bottom) {
                    (Color::Default, Color::Default) => continue,
                    (color, Color::Default) => (UPPER, color, Color::Default),
                    (Color::Default, color) => (LOWER, color, Color::Default),
                    (top, bottom) => (UPPER, top, bottom),
                };
                let pixel = Pixel::new(ch).fg(fg).bg(bg);
                placer.put(Position::new(x, y), pixel, BlendMode::Replace);
            }
        }
    }

    fn draw_horizontal(&self, placer: &mut dyn Placer, pos: Position) {
        const LEFT: char = '▌';
        const RIGHT: char = '▐';

        for y1 in 0..self.size.height as i32 {
            for x1 in 0..(self.size.width as i32 / 2) {
                let (x, y) = (pos.x + x1, pos.y + y1);
                let left = self.surface[Position::new(2 * x1, y1)];
                let right = self.surface[Position::new(2 * x1 + 1, y1)];
                let (ch, fg, bg) = match (left, right) {
                    (Color::Default, Color::Default) => continue,
                    (color, Color::Default) => (LEFT, color, Color::Default),
                    (Color::Default, color) => (RIGHT, color, Color::Default),
                    (top, bottom) => (LEFT, top, bottom),
                };
                let pixel = Pixel::new(ch).fg(fg).bg(bg);
                placer.put(Position::new(x, y), pixel, BlendMode::Replace);
            }
        }
    }
}
