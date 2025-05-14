use std::{collections::VecDeque, sync::Arc};

use anyhow::Context as _;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use parking_lot::Mutex;

use crate::process::{Buffer, SAMPLE_SIZE};

pub struct CpalBuffer {
    rx: std::sync::mpsc::Receiver<Box<[f32]>>,
    buffer: VecDeque<f32>,
}

impl CpalBuffer {
    pub fn drain_in_background(self) -> Arc<Mutex<dyn Buffer<SAMPLE_SIZE>>> {
        let buffer = Arc::new(Mutex::new(self)) as Arc<Mutex<dyn Buffer<SAMPLE_SIZE>>>;

        let _ = std::thread::spawn({
            let buffer = buffer.clone();
            move || {
                loop {
                    if let Some(mut g) = buffer.try_lock() {
                        g.read_samples();
                    }
                    std::thread::park_timeout(std::time::Duration::from_micros(1));
                    std::thread::yield_now();
                }
            }
        });

        buffer
    }
}

impl Buffer<SAMPLE_SIZE> for CpalBuffer {
    // this shouldn't be non-blocking, it should be async
    // so, the UI can update at 60 fps, but the buffer is populated at 300~ (with a sample rate of 96kHz) fps
    fn read_samples(&mut self) -> Option<&[f32; SAMPLE_SIZE]> {
        let Ok(data) = self.rx.try_recv() else {
            return None;
        };

        let current = self.buffer.len();
        let delta = current
            .saturating_add(data.len())
            .saturating_sub(SAMPLE_SIZE);

        self.buffer.drain(..delta);
        self.buffer.extend(data);

        if self.buffer.len() == SAMPLE_SIZE {
            let slice = self.buffer.make_contiguous();
            let data = <&[f32; SAMPLE_SIZE]>::try_from(&*slice).unwrap();
            return Some(data);
        }

        None
    }
}

pub struct Context {
    _stream: cpal::Stream,
    sample_rate: cpal::SampleRate,
}

impl Context {
    pub fn create() -> anyhow::Result<(Self, CpalBuffer)> {
        let host = cpal::default_host();

        let output = host
            .default_output_device()
            .with_context(|| anyhow::anyhow!("no default output device"))?;

        let config = output.default_output_config()?;
        let mut config = config.config();
        config.buffer_size = cpal::BufferSize::Fixed(100 as _);

        let (tx, rx) = std::sync::mpsc::channel();

        let stream = output.build_input_stream(
            &config,
            move |data: &[f32], _| drop(tx.send(Box::from(data))),
            |err| eprintln!("cpal input stream read err: {err}"),
            None,
        )?;

        stream
            .play()
            .with_context(|| "cannot start output stream")?;

        let handle = CpalBuffer {
            buffer: VecDeque::with_capacity(SAMPLE_SIZE),
            rx,
        };

        let this = Self {
            _stream: stream,
            sample_rate: config.sample_rate,
        };

        Ok((this, handle))
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate.0
    }
}
