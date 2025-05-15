use std::collections::VecDeque;

use anyhow::Context as _;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use scram_process::{Buffer, Source};

pub struct CpalBuffer {
    rx: flume::Receiver<Box<[f32]>>,
    buffer: VecDeque<f32>,
}

impl Buffer for CpalBuffer {
    #[profiling::function]
    fn read_samples(&mut self, sample_size: usize) -> Option<&[f32]> {
        let data = self.rx.recv().ok()?;

        profiling::scope!("append data");
        let current = self.buffer.len();
        let delta = current
            .saturating_add(data.len())
            .saturating_sub(sample_size);

        self.buffer.drain(..delta);
        self.buffer.extend(data);

        if self.buffer.len() == sample_size {
            profiling::scope!("vecdeque to slice");
            return Some(self.buffer.make_contiguous());
        }

        None
    }
}

pub struct Context {
    _stream: cpal::Stream,
    sample_rate: cpal::SampleRate,
    sample_size: usize,
}

impl Context {
    pub fn create(sample_size: usize) -> anyhow::Result<(impl Source, CpalBuffer)> {
        let host = cpal::default_host();

        let output = host
            .default_output_device()
            .with_context(|| anyhow::anyhow!("no default output device"))?;

        let config = output.default_output_config()?;
        let config = config.config();

        let (tx, rx) = flume::bounded(4); // gave it some headroom
        let stream = output.build_input_stream(
            &config,
            move |data: &[f32], _| {
                _ = tx.send(Box::from(data));
            },
            |err| eprintln!("cpal input stream read err: {err}"),
            None,
        )?;

        stream
            .play()
            .with_context(|| "cannot start output stream")?;

        let handle = CpalBuffer {
            buffer: VecDeque::with_capacity(sample_size),
            rx,
        };

        let this = Self {
            _stream: stream,
            sample_rate: config.sample_rate,
            sample_size,
        };

        Ok((this, handle))
    }
}

impl Source for Context {
    fn sample_rate(&self) -> u32 {
        self.sample_rate.0
    }

    fn sample_size(&self) -> usize {
        self.sample_size
    }
}
