pub trait Canvas {
    /// Place a color at x,y relative to the internal origin (offset)
    fn put(&mut self, x: i32, y: i32, color: surface::Rgba);
    /// Max width available to draw into
    fn width(&self) -> u32;
    /// Max height available to draw into
    fn height(&self) -> u32;
}

pub trait Visual {
    fn draw(&mut self, left: &[Frequency], right: &[Frequency], dt: f32, canvas: &mut impl Canvas);
    #[allow(unused)]
    fn resize(&mut self, size: math::Size) {}
}

pub use scram_process::Frequency;

pub mod math;
pub mod surface;
pub mod visualizers;
