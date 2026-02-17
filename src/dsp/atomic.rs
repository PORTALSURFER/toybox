//! Atomic f32 convenience type for lock-free scalar state.

use std::sync::atomic::{AtomicU32, Ordering};

/// Lock-free atomic float storage backed by `AtomicU32`.
///
/// This type is useful for cross-thread plugin state that needs simple
/// non-blocking reads and writes without introducing shared locks.
#[derive(Debug)]
pub struct AtomicF32 {
    /// Packed bit representation used by the atomic integer primitive.
    value: AtomicU32,
}

impl Default for AtomicF32 {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl AtomicF32 {
    /// Create a new atomically-updatable floating-point value.
    pub fn new(value: f32) -> Self {
        Self {
            value: AtomicU32::new(u32::from_ne_bytes(value.to_ne_bytes())),
        }
    }

    /// Read the current value.
    pub fn load(&self, ordering: Ordering) -> f32 {
        f32::from_ne_bytes(self.value.load(ordering).to_ne_bytes())
    }

    /// Store a new value.
    pub fn store(&self, value: f32, ordering: Ordering) {
        self.value
            .store(u32::from_ne_bytes(value.to_ne_bytes()), ordering);
    }
}

#[cfg(test)]
mod tests {
    use super::AtomicF32;
    use std::sync::atomic::Ordering;

    #[test]
    fn atomic_f32_roundtrips_bits_via_store_and_load() {
        let value = AtomicF32::new(1.25);

        value.store(-0.75, Ordering::Relaxed);
        let round_trip = value.load(Ordering::Relaxed);

        assert_eq!(round_trip.to_bits(), (-0.75_f32).to_bits());
    }
}
