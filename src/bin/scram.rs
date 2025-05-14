use std::sync::Arc;

use mars_app::{Action, Application, Event, Renderer, Rgba, Runner};
use parking_lot::Mutex;

use scram::{
    capture::Context,
    math::Style,
    process::{Buffer, Processor, SAMPLE_SIZE, config},
    visualizer::Visualizer,
};

#[cfg(feature = "profile")]
fn start_puffin() -> impl Drop {
    let server_addr = format!("127.0.0.1:{}", puffin_http::DEFAULT_PORT);
    let puffin_server = puffin_http::Server::new(&server_addr).unwrap();
    profiling::puffin::set_scopes_on(true);
    puffin_server
}

#[cfg(not(feature = "profile"))]
fn start_puffin() -> impl Drop {
    struct Noop;
    impl Drop for Noop {
        fn drop(&mut self) {}
    }
    Noop
}

fn main() -> anyhow::Result<()> {
    let _profile = start_puffin();

    let config = config::Config {
        binning: config::Binning::Bark,
        window: config::Window::Blackman,
        scaling: config::Scaling::Logarithimic,
        peak_smoothing: config::PeakSmoothing {
            attack_rate: 20.0,
            decay_rate: 0.8,
            decay_limit: 1.0,
            peak_threshold: 0.001,
        },
        frequency_cutoff: config::FrequencyCutoff {
            low: 20.0,
            high: 14000.0, // FIXME this should be at 16kHz but the mel intervals aren't wrong
        },
        band_smoothing: config::BandSmoothing::Exponential { factor: 0.3 },
    };

    let left_style = Style {
        color: Rgba::hex("#0FF"),
        accent: Rgba::hex("#F00"),
        ratio: 3.5,
    };

    let right_style = Style {
        color: Rgba::hex("#339"),
        accent: Rgba::hex("#909"),
        ratio: 1.5,
    };

    let (ctx, buffer) = Context::create()?;
    App {
        processor: Processor::new(ctx.sample_rate(), config),
        visualizer: Visualizer::new(left_style, right_style),
        buffer: buffer.drain_in_background(),
        _ctx: ctx,
        dt: 0.0,
    }
    .run(60.0)?;

    Ok(())
}

struct App {
    processor: Processor,
    visualizer: Visualizer,
    buffer: Arc<Mutex<dyn Buffer<SAMPLE_SIZE>>>,
    _ctx: Context,
    dt: f32,
}

impl Application for App {
    fn event(&mut self, event: Event) -> Action {
        if let Event::Resize { size } = event {
            let bands = self.visualizer.axis().cross(size);
            self.processor.set_bands(bands as usize);
            self.visualizer.resize(size);
        }
        Action::Continue
    }

    fn update(&mut self, update: mars_app::Update) -> mars_app::ShouldRender {
        self.dt += update.dt;
        mars_app::ShouldRender::Yes
    }

    #[profiling::function]
    fn render(&mut self, renderer: &mut impl Renderer) {
        profiling::finish_frame!();

        {
            profiling::scope!("lock and update");
            let g = &mut *self.buffer.lock();
            self.processor.update(g);
        }

        let (left, right) = self.processor.current_frequencies();
        self.visualizer.draw(left, right, self.dt / 1.0, renderer);
    }
}
