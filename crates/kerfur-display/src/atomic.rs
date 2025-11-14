use core::sync::atomic::{AtomicU32, Ordering};

#[repr(transparent)]
#[derive(Debug, Default)]
pub(crate) struct AtomicF32(AtomicU32);

impl AtomicF32 {
    /// Create a new [`AtomicF32`].
    #[must_use]
    pub(crate) const fn new(value: f32) -> Self { Self(AtomicU32::new(value.to_bits())) }

    /// Load the current value of the atomic float.
    #[must_use]
    pub(crate) fn load(&self, order: Ordering) -> f32 { f32::from_bits(self.0.load(order)) }

    /// Store a new value into the atomic float.
    pub(crate) fn store(&self, value: f32, order: Ordering) {
        self.0.store(value.to_bits(), order);
    }
}
