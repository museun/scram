use mars_app::{Axis, BlendMode, Drawable as _, Renderer, Size};

use scram_visualize::{Frequency, Visual, visualizers::*};

use crate::half_block::HalfBlockRenderer;

pub struct Visualizer {
    renderer: HalfBlockRenderer,
    spectro: ScrollingSpectro,
}

impl Default for Visualizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Visualizer {
    pub fn new() -> Self {
        Self {
            renderer: HalfBlockRenderer::new(Size::ZERO, Axis::Vertical),
            spectro: ScrollingSpectro::new(),
        }
    }

    pub fn resize(&mut self, size: Size) {
        let _unscaled = self.renderer.resize(size);
        self.spectro.resize(self.renderer.dimensions());
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
        SpecSlice.draw(left, right, dt, &mut self.renderer);

        // self.spectro.draw(left, right, dt, &mut self.renderer);

        // let left = Style {
        //     color: Rgba::hex("#0FF"),
        //     accent: Rgba::hex("#F00"),
        //     ratio: 3.5,
        // };

        // let right = Style {
        //     color: Rgba::hex("#339"),
        //     accent: Rgba::hex("#909"),
        //     ratio: 1.5,
        // };

        // StackedChannels::new(
        //     Style {
        //         color: Rgba::hex("#933"),
        //         accent: Rgba::hex("#909"),
        //         ratio: 1.5,
        //     }, //
        //     Style {
        //         color: Rgba::hex("#339"),
        //         accent: Rgba::hex("#909"),
        //         ratio: 1.5,
        //     },
        // )
        // .draw(left, right, dt, &mut self.renderer);

        // StackedOutline::new(
        //     Style {
        //         color: Rgba::hex("#538"),
        //         accent: Rgba::hex("#FFF"),
        //         ratio: 0.8,
        //     },
        //     Style {
        //         color: Rgba::hex("#538"),
        //         accent: Rgba::hex("#FFF"),
        //         ratio: 0.2,
        //     },
        // )
        // .draw(left, right, dt, &mut self.renderer);

        StackedFreqs.draw(left, right, dt, &mut self.renderer);
        // SpecRibbon.draw(left, right, dt, &mut self.renderer);
        // SpecCircular.draw(left, right, dt, &mut self.renderer);
        RadialBloom.draw(left, right, dt, &mut self.renderer);

        self.renderer.render(renderer, BlendMode::Replace);
        self.renderer.clear();
    }
}
