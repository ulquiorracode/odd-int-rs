//! Signed arbitrary bit-width integer implementation: `Int<const BITS: usize, const BYTES: usize>`.

use crate::error::OutOfRangeError;
use core::fmt;

/// A signed integer of arbitrary bit-width `BITS` occupying exactly `BYTES` bytes.
///
/// # Physical Invariants
/// - Exact memory size: `size_of::<Int<BITS, BYTES>>() == BYTES`.
/// - Alignment: 1 byte (`align_of::<Int<BITS, BYTES>>() == 1`).
/// - Value domain: $[-2^{\text{BITS} - 1}, 2^{\text{BITS} - 1} - 1]$ (two's complement).
/// - Little-endian byte order with two's complement sign-extension.
#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "zerocopy",
    derive(
        zerocopy::FromBytes,
        zerocopy::IntoBytes,
        zerocopy::KnownLayout,
        zerocopy::Immutable
    )
)]
pub struct Int<const BITS: usize, const BYTES: usize> {
    pub(crate) raw: [u8; BYTES],
}

impl<const BITS: usize, const BYTES: usize> Int<BITS, BYTES> {
    /// Number of bytes required to hold `BITS` bits.
    pub const BYTES: usize = BYTES;

    /// Bitmask for the highest (most significant) byte.
    pub const HIGH_BYTE_MASK: u8 = if BITS % 8 == 0 {
        0xFF
    } else {
        ((1u16 << (BITS % 8)) - 1) as u8
    };

    /// Sign bit position in the most significant byte.
    pub const SIGN_BIT_MASK: u8 = 1 << ((BITS - 1) % 8);

    /// Represents the value 0.
    pub const ZERO: Self = Self { raw: [0u8; BYTES] };

    /// Returns the bit-width of this integer.
    #[inline(always)]
    pub const fn bits(&self) -> usize {
        BITS
    }

    /// Returns the raw byte representation in little-endian order.
    #[inline(always)]
    pub const fn to_le_bytes(self) -> [u8; BYTES] {
        self.raw
    }

    /// Creates an instance directly from raw bytes.
    ///
    /// # Safety
    ///
    /// Caller must ensure that any unused high bits beyond `BITS` match the two's complement sign
    /// of the value.
    #[inline(always)]
    pub const unsafe fn from_le_bytes_unchecked(raw: [u8; BYTES]) -> Self {
        Self { raw }
    }

    /// Converts this `Int<BITS, BYTES>` into a standard 64-bit signed integer with sign extension.
    #[inline]
    pub const fn to_i64(&self) -> i64 {
        assert!(BITS <= 64, "to_i64 requires BITS <= 64");
        if BYTES == 0 {
            return 0;
        }
        let mut unsigned = 0u64;
        let mut i = 0;
        while i < BYTES {
            unsigned |= (self.raw[i] as u64) << (i * 8);
            i += 1;
        }

        let is_negative = (self.raw[BYTES - 1] & Self::SIGN_BIT_MASK) != 0;
        if is_negative && BITS < 64 {
            let sign_extend_mask = !((1u64 << BITS) - 1);
            (unsigned | sign_extend_mask) as i64
        } else {
            unsigned as i64
        }
    }

    /// Converts this `Int<BITS, BYTES>` into an `isize` with sign extension.
    #[inline(always)]
    pub const fn to_isize(&self) -> isize {
        self.to_i64() as isize
    }

    /// Creates an `Int<BITS, BYTES>` from an `i64`, checking range bounds.
    pub const fn from_i64(val: i64) -> Result<Self, OutOfRangeError> {
        if BITS < 64 {
            let min = -(1i64 << (BITS - 1));
            let max = (1i64 << (BITS - 1)) - 1;
            if val < min || val > max {
                return Err(OutOfRangeError::new(BITS));
            }
        }
        let mut raw = [0u8; BYTES];
        let bytes_to_copy = if BYTES < 8 { BYTES } else { 8 };
        let val_bytes = val.to_le_bytes();
        let mut i = 0;
        while i < bytes_to_copy {
            raw[i] = val_bytes[i];
            i += 1;
        }
        // Mask unused bits in high byte
        if BYTES > 0 && BITS % 8 != 0 {
            let is_negative = val < 0;
            if is_negative {
                raw[BYTES - 1] |= !Self::HIGH_BYTE_MASK;
            } else {
                raw[BYTES - 1] &= Self::HIGH_BYTE_MASK;
            }
        }
        Ok(Self { raw })
    }
}

impl<const BITS: usize, const BYTES: usize> Default for Int<BITS, BYTES> {
    #[inline(always)]
    fn default() -> Self {
        Self::ZERO
    }
}

impl<const BITS: usize, const BYTES: usize> fmt::Debug for Int<BITS, BYTES> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if BITS <= 64 {
            write!(f, "Int<{}, {}>({})", BITS, BYTES, self.to_i64())
        } else {
            let raw = self.raw;
            write!(f, "Int<{}, {}>({:?})", BITS, BYTES, raw)
        }
    }
}

impl<const BITS: usize, const BYTES: usize> fmt::Display for Int<BITS, BYTES> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if BITS <= 64 {
            write!(f, "{}", self.to_i64())
        } else {
            let raw = self.raw;
            write!(f, "{:?}", raw)
        }
    }
}

impl<const BITS: usize, const BYTES: usize> TryFrom<i64> for Int<BITS, BYTES> {
    type Error = OutOfRangeError;

    #[inline(always)]
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Self::from_i64(value)
    }
}

impl<const BITS: usize, const BYTES: usize> TryFrom<isize> for Int<BITS, BYTES> {
    type Error = OutOfRangeError;

    #[inline(always)]
    fn try_from(value: isize) -> Result<Self, Self::Error> {
        Self::from_i64(value as i64)
    }
}
