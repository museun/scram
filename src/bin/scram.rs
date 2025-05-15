use mars_app::{Action, Application, Event, Renderer, Rgba, Runner};

use scram::{
    background::Queue,
    capture::Context,
    math::Style,
    process::{Processor, Source, config},
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
        banding: config::Banding {
            frequency_cutoff: config::FrequencyCutoff {
                low: 20.0,
                high: 20000.0,
            },
            scale: config::FrequencyScale::Mel,
        },
        window: config::Window::Blackman,
        scaling: config::VolumeScale::Logarithimic,
        peak_smoothing: config::PeakSmoothing {
            attack_rate: 20.0,
            decay_rate: 0.3,
            decay_limit: 0.5,
            peak_threshold: 0.001,
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

    let sample_size = Processor::MAX_SAMPLE_SIZE;
    let (source, mut buffer) = Context::create(sample_size)?;

    let mut processor = Processor::new(source.sample_rate(), sample_size, config)?;
    let queue = Queue::default();

    let (tx, rx) = flume::unbounded();

    std::thread::spawn({
        let queue = queue.clone();
        profiling::register_thread!("read samples");
        move || loop {
            match rx.try_recv() {
                Ok(bands) => processor.set_bands(bands),
                Err(flume::TryRecvError::Disconnected) => return,
                _ => {}
            }

            if processor.update(&mut buffer) {
                profiling::scope!("put in current frequencies");
                queue.put(processor.current_frequencies().map(<_>::to_owned));
            }
        }
    });

    App {
        tx,
        queue,
        visualizer: Visualizer::new(left_style, right_style),

        _source: Box::new(source),
        dt: 0.0,
    }
    .run(60.0)?;

    Ok(())
}

struct App {
    queue: Queue,
    tx: flume::Sender<usize>,
    visualizer: Visualizer,
    _source: Box<dyn Source>,
    dt: f32,
}

impl Application for App {
    fn event(&mut self, event: Event) -> Action {
        if let Event::Resize { size } = event {
            let bands = self.visualizer.axis().cross(size);
            _ = self.tx.send(bands as usize);
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
        if let Some([left, right]) = self.queue.take() {
            self.visualizer.draw(&left, &right, self.dt / 1.0, renderer);
        }
    }
}
