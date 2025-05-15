pub mod capture;
pub mod math;
pub mod process;
pub mod synth;
pub mod visualizer;

pub mod background {
    use std::sync::Arc;

    use parking_lot::Mutex;

    use crate::process::Frequency;

    #[derive(Clone, Default)]
    pub struct Queue(Arc<Mutex<Option<[Vec<Frequency>; 2]>>>);

    impl Queue {
        pub fn take(&self) -> Option<[Vec<Frequency>; 2]> {
            self.0.try_lock()?.clone()
        }

        pub fn put(&self, frequencies: [Vec<Frequency>; 2]) {
            _ = self.0.lock().replace(frequencies);
        }
    }
}
