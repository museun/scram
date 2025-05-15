pub trait Buffer: Send + 'static {
    fn read_samples(&mut self, sample_size: usize) -> Option<&[f32]>;
}

impl<T: Buffer> Buffer for Box<T> {
    fn read_samples(&mut self, sample_size: usize) -> Option<&[f32]> {
        (**self).read_samples(sample_size)
    }
}

pub trait Source {
    fn sample_rate(&self) -> u32;
    fn sample_size(&self) -> usize;
}
