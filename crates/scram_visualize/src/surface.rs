#[doc(inline)]
pub use mars_surface::{ResizeMode, Rgba, Surface};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    pub color: Rgba,
    pub accent: Rgba,
    pub ratio: f32,
}
