use std::{collections::VecDeque, f32::consts::TAU};

use crate::process::{Buffer, Source};

pub struct SynthSource {
    sample_rate: u32,
    sample_size: usize,
}

impl SynthSource {
    pub const fn new(sample_rate: u32, sample_size: usize) -> Self {
        Self {
            sample_rate,
            sample_size,
        }
    }
}

impl Source for SynthSource {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn sample_size(&self) -> usize {
        self.sample_size
    }
}

pub struct SynthBuffer {
    buffer: VecDeque<f32>,
    source: Box<dyn Iterator<Item = f32> + Send>,
}

impl SynthBuffer {
    pub fn new(sample_rate: u32, sample_size: usize) -> Self {
        let forward = frequency_sweep(20.0, 18000.0, 2.5);
        let backward = frequency_sweep(18000.0, 20.0, 2.5);

        Self {
            buffer: VecDeque::with_capacity(sample_size),
            source: Box::new(samples(forward, backward, sample_rate as _)),
        }
    }
}

impl Buffer for SynthBuffer {
    fn read_samples(&mut self, sample_size: usize) -> Option<&[f32]> {
        let window_size = sample_size / 4;

        let len = self.buffer.len();
        let delta = len.saturating_add(window_size).saturating_sub(sample_size);

        self.buffer.drain(..delta);
        self.buffer.extend(self.source.by_ref().take(window_size));

        if self.buffer.len() == sample_size {
            return Some(self.buffer.make_contiguous());
        }
        None
    }
}

pub fn samples(
    left: impl Fn(f32) -> f32 + 'static,
    right: impl Fn(f32) -> f32 + 'static,
    sample_rate: f32,
) -> impl Iterator<Item = f32> {
    let mut frame = 0_usize;
    std::iter::from_fn(move || {
        let time = frame as f32 / sample_rate;
        frame += 1;
        Some([left(time), right(time)])
    })
    .flatten()
}

pub fn sine_wave(freq: f32, phase: f32) -> impl Fn(f32) -> f32 {
    move |t| (TAU * freq * t * phase).sin() * 0.5 + 0.5
}

pub fn square_wave(freq: f32, duty_cycle: f32) -> impl Fn(f32) -> f32 {
    move |t| {
        let phase = (t * freq) % 1.0;
        if phase < duty_cycle { 1.0 } else { 0.0 }
    }
}

pub fn frequency_sweep(start: f32, end: f32, duration: f32) -> impl Fn(f32) -> f32 + Copy {
    let df = end - start;
    move |t| {
        if t < 0.0 || t > duration {
            return 0.0;
        }
        let phase = TAU * (start * t + (df * t * t) / (2.0 * duration));
        0.5 + 0.5 * phase.sin()
    }
}

pub fn note_freq(note: u8, ref_note: f32, ref_freq: f32) -> f32 {
    ref_freq * 2.0_f32.powf((note as f32 - ref_note) / 12.0)
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Note(u8);

impl Note {
    pub fn parse(name: &str) -> Option<Self> {
        const SEMITONES: &[(&str, u8)] = &[
            ("C", 0),
            ("C#", 1),
            ("Db", 1),
            ("D", 2),
            ("D#", 3),
            ("Eb", 3),
            ("E", 4),
            ("F", 5),
            ("F#", 6),
            ("Cb", 6),
            ("G", 7),
            ("G#", 8),
            ("Ab", 8),
            ("A", 9),
            ("A#", 10),
            ("Bb", 10),
            ("B", 11),
        ];

        let (note, octave) = name.split_at(if name.len() == 3 { 2 } else { 1 });
        let octave = octave.parse::<i8>().ok().unwrap_or(0);

        let semitone = SEMITONES
            .iter()
            .find_map(|&(n, o)| if n == note { Some(o) } else { None })?;

        let val = 12 * (octave + 1) + semitone as i8;
        if val.is_negative() {
            return None;
        }

        Some(Self(val as u8))
    }

    pub fn to_freq(self) -> f32 {
        const A4_0: f32 = 69.0;
        const A4_1: f32 = 440.0;
        note_freq(self.0, A4_0, A4_1)
    }
}
