//! Signed arbitrary bit-width integer implementation: `Int<const BITS: usize, const BYTES: usize>`.

use crate::error::OutOfRangeError;
use core::cmp::Ordering;
use core::fmt;

/// A signed integer of arbitrary bit-width `BITS` occupying exactly `BYTES` bytes.
///
/// # Physical Invariants
/// - Exact memory size: `size_of::<Int<BITS, BYTES>>() == BYTES`.
/// - Alignment: 1 byte (`align_of::<Int<BITS, BYTES>>() == 1`).
/// - Value domain: $[-2^{\text{BITS} - 1}, 2^{\text{BITS} - 1} - 1]$ (two's complement).
/// - Little-endian byte order with two's complement sign-extension across any unused high bits.
#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "zerocopy",
    derive(zerocopy::IntoBytes, zerocopy::KnownLayout, zerocopy::Immutable)
)]
pub struct Int<const BITS: usize, const BYTES: usize> {
    pub(crate) raw: [u8; BYTES],
}

impl<const BITS: usize, const BYTES: usize> Int<BITS, BYTES> {
    /// Number of bytes required to hold `BITS` bits.
    pub const BYTES: usize = BYTES;

    /// Bitmask for the highest (most significant) byte containing data bits.
    pub const HIGH_BYTE_MASK: u8 = if BITS % 8 == 0 {
        0xFF
    } else {
        ((1u16 << (BITS % 8)) - 1) as u8
    };

    /// Sign bit mask in the most significant data byte.
    pub const SIGN_BIT_MASK: u8 = 1 << ((BITS - 1) % 8);

    /// Represents the value 0.
    pub const ZERO: Self = {
        assert!(BYTES > 0, "BYTES must be greater than 0");
        assert!(BITS > 0, "BITS must be greater than 0");
        assert!(BITS <= BYTES * 8, "BITS cannot exceed BYTES * 8");
        Self { raw: [0u8; BYTES] }
    };

    /// Represents the minimum representable value $-2^{\text{BITS} - 1}$.
    pub const MIN: Self = {
        assert!(BYTES > 0, "BYTES must be greater than 0");
        assert!(BITS > 0, "BITS must be greater than 0");
        assert!(BITS <= BYTES * 8, "BITS cannot exceed BYTES * 8");
        let mut bytes = [0u8; BYTES];
        let msb_index = (BITS - 1) / 8;
        bytes[msb_index] = Self::SIGN_BIT_MASK | (!Self::HIGH_BYTE_MASK);
        let mut i = msb_index + 1;
        while i < BYTES {
            bytes[i] = 0xFF;
            i += 1;
        }
        Self { raw: bytes }
    };

    /// Represents the maximum representable value $2^{\text{BITS} - 1} - 1$.
    pub const MAX: Self = {
        assert!(BYTES > 0, "BYTES must be greater than 0");
        assert!(BITS > 0, "BITS must be greater than 0");
        assert!(BITS <= BYTES * 8, "BITS cannot exceed BYTES * 8");
        let mut bytes = [0u8; BYTES];
        let msb_index = (BITS - 1) / 8;
        let mut i = 0;
        while i < msb_index {
            bytes[i] = 0xFF;
            i += 1;
        }
        bytes[msb_index] = Self::SIGN_BIT_MASK - 1;
        Self { raw: bytes }
    };

    /// Returns the bit-width of this integer.
    #[inline(always)]
    pub const fn bits(&self) -> usize {
        BITS
    }

    /// Returns true if this integer is negative.
    #[inline(always)]
    pub const fn is_negative(&self) -> bool {
        let msb_index = (BITS - 1) / 8;
        (self.raw[msb_index] & Self::SIGN_BIT_MASK) != 0
    }

    /// Returns the raw byte representation in little-endian order.
    #[inline(always)]
    pub const fn to_le_bytes(self) -> [u8; BYTES] {
        self.raw
    }

    /// Creates an instance directly from raw bytes.
    ///
    /// # Safety
    /// Caller must ensure that any unused high bits beyond `BITS` strictly match the two's complement sign
    /// of the value.
    #[inline(always)]
    pub const unsafe fn from_le_bytes_unchecked(raw: [u8; BYTES]) -> Self {
        Self { raw }
    }

    /// Attempts to construct an instance from raw little-endian bytes, verifying canonical two's complement sign-extension.
    pub const fn from_le_bytes(raw: [u8; BYTES]) -> Result<Self, OutOfRangeError> {
        assert!(BYTES > 0, "BYTES must be greater than 0");
        assert!(BITS > 0, "BITS must be greater than 0");
        assert!(BITS <= BYTES * 8, "BITS cannot exceed BYTES * 8");

        let msb_index = (BITS - 1) / 8;
        let is_negative = (raw[msb_index] & Self::SIGN_BIT_MASK) != 0;
        let expected_high_bits = if is_negative {
            !Self::HIGH_BYTE_MASK
        } else {
            0
        };
        if (raw[msb_index] & !Self::HIGH_BYTE_MASK) != expected_high_bits {
            return Err(OutOfRangeError::new(BITS));
        }
        let expected_extension_byte = if is_negative { 0xFF } else { 0x00 };
        let mut i = msb_index + 1;
        while i < BYTES {
            if raw[i] != expected_extension_byte {
                return Err(OutOfRangeError::new(BITS));
            }
            i += 1;
        }
        Ok(Self { raw })
    }

    /// Converts this `Int<BITS, BYTES>` into a standard 64-bit signed integer with sign extension.
    #[inline]
    pub const fn to_i64(&self) -> i64 {
        assert!(BITS <= 64, "to_i64 requires BITS <= 64");
        let mut unsigned = 0u64;
        let limit = if BYTES < 8 { BYTES } else { 8 };
        let mut i = 0;
        while i < limit {
            unsigned |= (self.raw[i] as u64) << (i * 8);
            i += 1;
        }

        if self.is_negative() && BITS < 64 {
            let sign_extend_mask = !((1u64 << BITS) - 1);
            (unsigned | sign_extend_mask) as i64
        } else {
            unsigned as i64
        }
    }

    /// Converts this `Int<BITS, BYTES>` into a standard 128-bit signed integer with sign extension.
    #[inline]
    pub const fn to_i128(&self) -> i128 {
        assert!(BITS <= 128, "to_i128 requires BITS <= 128");
        let mut unsigned = 0u128;
        let limit = if BYTES < 16 { BYTES } else { 16 };
        let mut i = 0;
        while i < limit {
            unsigned |= (self.raw[i] as u128) << (i * 8);
            i += 1;
        }

        if self.is_negative() && BITS < 128 {
            let sign_extend_mask = !((1u128 << BITS) - 1);
            (unsigned | sign_extend_mask) as i128
        } else {
            unsigned as i128
        }
    }

    /// Converts this `Int<BITS, BYTES>` into an `isize` with sign extension.
    #[inline(always)]
    pub const fn to_isize(&self) -> isize {
        self.to_i64() as isize
    }

    /// Creates an `Int<BITS, BYTES>` from an `i64`, checking range bounds.
    pub const fn from_i64(val: i64) -> Result<Self, OutOfRangeError> {
        assert!(BYTES > 0, "BYTES must be greater than 0");
        assert!(BITS > 0, "BITS must be greater than 0");
        assert!(BITS <= BYTES * 8, "BITS cannot exceed BYTES * 8");
        if BITS < 64 {
            let min = -(1i64 << (BITS - 1));
            let max = (1i64 << (BITS - 1)) - 1;
            if val < min || val > max {
                return Err(OutOfRangeError::new(BITS));
            }
        }
        let is_negative = val < 0;
        let mut raw = if is_negative {
            [0xFFu8; BYTES]
        } else {
            [0x00u8; BYTES]
        };
        let bytes_to_copy = if BYTES < 8 { BYTES } else { 8 };
        let val_bytes = val.to_le_bytes();
        let mut i = 0;
        while i < bytes_to_copy {
            raw[i] = val_bytes[i];
            i += 1;
        }
        let msb_index = (BITS - 1) / 8;
        if BITS % 8 != 0 {
            if is_negative {
                raw[msb_index] |= !Self::HIGH_BYTE_MASK;
            } else {
                raw[msb_index] &= Self::HIGH_BYTE_MASK;
            }
        }
        let mut j = msb_index + 1;
        while j < BYTES {
            raw[j] = if is_negative { 0xFF } else { 0x00 };
            j += 1;
        }
        Ok(Self { raw })
    }

    /// Creates an `Int<BITS, BYTES>` from an `i128`, checking range bounds.
    pub const fn from_i128(val: i128) -> Result<Self, OutOfRangeError> {
        assert!(BYTES > 0, "BYTES must be greater than 0");
        assert!(BITS > 0, "BITS must be greater than 0");
        assert!(BITS <= BYTES * 8, "BITS cannot exceed BYTES * 8");
        if BITS < 128 {
            let min = -(1i128 << (BITS - 1));
            let max = (1i128 << (BITS - 1)) - 1;
            if val < min || val > max {
                return Err(OutOfRangeError::new(BITS));
            }
        }
        let is_negative = val < 0;
        let mut raw = if is_negative {
            [0xFFu8; BYTES]
        } else {
            [0x00u8; BYTES]
        };
        let bytes_to_copy = if BYTES < 16 { BYTES } else { 16 };
        let val_bytes = val.to_le_bytes();
        let mut i = 0;
        while i < bytes_to_copy {
            raw[i] = val_bytes[i];
            i += 1;
        }
        let msb_index = (BITS - 1) / 8;
        if BITS % 8 != 0 {
            if is_negative {
                raw[msb_index] |= !Self::HIGH_BYTE_MASK;
            } else {
                raw[msb_index] &= Self::HIGH_BYTE_MASK;
            }
        }
        let mut j = msb_index + 1;
        while j < BYTES {
            raw[j] = if is_negative { 0xFF } else { 0x00 };
            j += 1;
        }
        Ok(Self { raw })
    }
}

impl<const BITS: usize, const BYTES: usize> Ord for Int<BITS, BYTES> {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_neg = self.is_negative();
        let other_neg = other.is_negative();

        match (self_neg, other_neg) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => {
                let msb_index = (BITS - 1) / 8;
                let mut i = msb_index + 1;
                while i > 0 {
                    i -= 1;
                    match self.raw[i].cmp(&other.raw[i]) {
                        Ordering::Equal => continue,
                        ord => return ord,
                    }
                }
                Ordering::Equal
            }
        }
    }
}

impl<const BITS: usize, const BYTES: usize> PartialOrd for Int<BITS, BYTES> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
        } else if BITS <= 128 {
            write!(f, "Int<{}, {}>({})", BITS, BYTES, self.to_i128())
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
        } else if BITS <= 128 {
            write!(f, "{}", self.to_i128())
        } else {
            let raw = self.raw;
            if self.is_negative() {
                write!(f, "-0x")?;
            } else {
                write!(f, "0x")?;
            }
            for b in raw.iter().rev() {
                write!(f, "{:02x}", b)?;
            }
            Ok(())
        }
    }
}

impl<const BITS: usize, const BYTES: usize> fmt::Binary for Int<BITS, BYTES> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut bit = BITS;
        while bit > 0 {
            bit -= 1;
            let byte_idx = bit / 8;
            let bit_in_byte = bit % 8;
            let val = (self.raw[byte_idx] >> bit_in_byte) & 1;
            write!(f, "{}", val)?;
        }
        Ok(())
    }
}

impl<const BITS: usize, const BYTES: usize> fmt::LowerHex for Int<BITS, BYTES> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if BITS <= 64 {
            fmt::LowerHex::fmt(&self.to_i64(), f)
        } else if BITS <= 128 {
            fmt::LowerHex::fmt(&self.to_i128(), f)
        } else {
            let raw = self.raw;
            for b in raw.iter().rev() {
                write!(f, "{:02x}", b)?;
            }
            Ok(())
        }
    }
}

impl<const BITS: usize, const BYTES: usize> fmt::UpperHex for Int<BITS, BYTES> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if BITS <= 64 {
            fmt::UpperHex::fmt(&self.to_i64(), f)
        } else if BITS <= 128 {
            fmt::UpperHex::fmt(&self.to_i128(), f)
        } else {
            let raw = self.raw;
            for b in raw.iter().rev() {
                write!(f, "{:02X}", b)?;
            }
            Ok(())
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

impl<const BITS: usize, const BYTES: usize> TryFrom<i128> for Int<BITS, BYTES> {
    type Error = OutOfRangeError;

    #[inline(always)]
    fn try_from(value: i128) -> Result<Self, Self::Error> {
        Self::from_i128(value)
    }
}

#[cfg(feature = "serde")]
struct IntByteVisitor<const BYTES: usize>;

#[cfg(feature = "serde")]
impl<'de, const BYTES: usize> serde::de::Visitor<'de> for IntByteVisitor<BYTES> {
    type Value = [u8; BYTES];

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(formatter, "a byte sequence of length {}", BYTES)
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() == BYTES {
            let mut arr = [0u8; BYTES];
            arr.copy_from_slice(v);
            Ok(arr)
        } else {
            Err(E::invalid_length(v.len(), &self))
        }
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut arr = [0u8; BYTES];
        let mut i = 0;
        while i < BYTES {
            arr[i] = seq
                .next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
            i += 1;
        }
        Ok(arr)
    }
}

#[cfg(feature = "serde")]
impl<const BITS: usize, const BYTES: usize> serde::Serialize for Int<BITS, BYTES> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if BITS <= 64 {
            serializer.serialize_i64(self.to_i64())
        } else if BITS <= 128 {
            serializer.serialize_i128(self.to_i128())
        } else {
            let raw = self.raw;
            serializer.serialize_bytes(&raw)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de, const BITS: usize, const BYTES: usize> serde::Deserialize<'de> for Int<BITS, BYTES> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if BITS <= 64 {
            let val = i64::deserialize(deserializer)?;
            Self::from_i64(val).map_err(serde::de::Error::custom)
        } else if BITS <= 128 {
            let val = i128::deserialize(deserializer)?;
            Self::from_i128(val).map_err(serde::de::Error::custom)
        } else {
            let raw = deserializer.deserialize_bytes(IntByteVisitor::<BYTES>)?;
            Self::from_le_bytes(raw).map_err(serde::de::Error::custom)
        }
    }
}
