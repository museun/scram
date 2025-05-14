pub trait Buffer<const SAMPLE_SIZE: usize>: Send + 'static {
    fn read_samples(&mut self) -> Option<&[f32; SAMPLE_SIZE]>;
}

impl<const N: usize, T: Buffer<N>> Buffer<N> for Box<T> {
    fn read_samples(&mut self) -> Option<&[f32; N]> {
        (**self).read_samples()
    }
}
