//! The high-performance flags for indicating the state.

use std::sync::{atomic::AtomicBool, Arc};

#[derive(Clone)]
/// The flag storing a binary ([`bool`]) value.
pub struct BinaryFlag(Arc<AtomicBool>);

impl BinaryFlag {
    pub fn new() -> Self {
        Self::default()
    }

    /// Should we continue running?
    pub fn is_running(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Set the running flag.
    pub fn set_running(&self, value: bool) {
        self.0.store(value, std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for BinaryFlag {
    fn default() -> Self {
        Self(Arc::new(AtomicBool::new(true)))
    }
}
