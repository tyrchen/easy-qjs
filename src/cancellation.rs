use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

use crate::Cancellation;

impl Cancellation {
    pub fn new(deadline: Option<Instant>, cancellation: Arc<AtomicBool>) -> Cancellation {
        Self {
            deadline,
            cancellation,
        }
    }

    pub fn cancelled(&self) -> bool {
        self.deadline.map(|d| d <= Instant::now()).unwrap_or(false)
            || self.cancellation.load(Ordering::Relaxed)
    }
}
