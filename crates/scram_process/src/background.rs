use std::sync::Arc;

use parking_lot::Mutex;

use crate::Frequency;

#[derive(Clone, Default)]
pub struct Slot(Arc<Mutex<Option<[Vec<Frequency>; 2]>>>);

impl Slot {
    pub fn take(&self) -> Option<[Vec<Frequency>; 2]> {
        self.0.try_lock()?.clone()
    }

    pub fn put(&self, frequencies: [Vec<Frequency>; 2]) {
        _ = self.0.lock().replace(frequencies);
    }
}
