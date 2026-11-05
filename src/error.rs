//! Error types for arbitrary bit-width integer conversions.

use core::fmt;

/// Error returned when attempting to construct an integer from a value that exceeds
/// the maximum representable value for the given bit-width ($2^{\text{BITS}} - 1$).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutOfRangeError {
    /// Target bit-width.
    pub bits: usize,
}

impl OutOfRangeError {
    /// Creates a new `OutOfRangeError`.
    pub const fn new(bits: usize) -> Self {
        Self { bits }
    }
}

impl fmt::Display for OutOfRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "value out of range for arbitrary integer of {} bits",
            self.bits
        )
    }
}

#[cfg(feature = "std")]
impl std::error::Error for OutOfRangeError {}
