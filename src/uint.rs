//! Core implementation of `UInt<const BITS: usize, const BYTES: usize>`.

use crate::error::OutOfRangeError;
use core::cmp::Ordering;
use core::fmt;

/// An unsigned integer of arbitrary bit-width `BITS` occupying exactly `BYTES` bytes.
///
/// # Physical Invariants
/// - Exact memory size: `size_of::<UInt<BITS, BYTES>>() == BYTES`.
/// - Alignment: 1 byte (`align_of::<UInt<BITS, BYTES>>() == 1`).
/// - Value domain: $[0, 2^{\text{BITS}} - 1]$.
/// - Canonical representation: Stored in Little-Endian byte order. All unused high bits
///   in the most significant byte and any padding bytes are strictly guaranteed to be zero.
#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "zerocopy",
    derive(zerocopy::IntoBytes, zerocopy::KnownLayout, zerocopy::Immutable)
)]
pub struct UInt<const BITS: usize, const BYTES: usize> {
    pub(crate) raw: [u8; BYTES],
}

impl<const BITS: usize, const BYTES: usize> UInt<BITS, BYTES> {
    /// Number of bytes required to hold `BITS` bits.
    pub const BYTES: usize = BYTES;

    /// Bitmask for the highest (most significant) byte containing data bits.
    pub const HIGH_BYTE_MASK: u8 = if BITS % 8 == 0 {
        0xFF
    } else {
        ((1u16 << (BITS % 8)) - 1) as u8
    };

    /// Represents the value 0.
    pub const ZERO: Self = {
        assert!(BYTES > 0, "BYTES must be greater than 0");
        assert!(BITS > 0, "BITS must be greater than 0");
        assert!(BITS <= BYTES * 8, "BITS cannot exceed BYTES * 8");
        Self { raw: [0u8; BYTES] }
    };

    /// Represents the maximum value $2^{\text{BITS}} - 1$.
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
        bytes[msb_index] = Self::HIGH_BYTE_MASK;
        Self { raw: bytes }
    };

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

    /// Creates an instance directly from raw bytes without checking high-bit bounds.
    ///
    /// # Safety
    /// Caller must ensure that all unused bits beyond `BITS` in the most significant byte
    /// and any subsequent bytes are strictly 0.
    #[inline(always)]
    pub const unsafe fn from_le_bytes_unchecked(raw: [u8; BYTES]) -> Self {
        Self { raw }
    }

    /// Attempts to construct an instance from raw little-endian bytes, verifying that unused high bits are zero.
    pub const fn from_le_bytes(raw: [u8; BYTES]) -> Result<Self, OutOfRangeError> {
        assert!(BYTES > 0, "BYTES must be greater than 0");
        assert!(BITS > 0, "BITS must be greater than 0");
        assert!(BITS <= BYTES * 8, "BITS cannot exceed BYTES * 8");

        let msb_index = (BITS - 1) / 8;
        let high_byte = raw[msb_index];
        if (high_byte & !Self::HIGH_BYTE_MASK) != 0 {
            return Err(OutOfRangeError::new(BITS));
        }
        let mut i = msb_index + 1;
        while i < BYTES {
            if raw[i] != 0 {
                return Err(OutOfRangeError::new(BITS));
            }
            i += 1;
        }
        Ok(Self { raw })
    }

    /// Converts this `UInt<BITS, BYTES>` into a standard 64-bit unsigned integer.
    #[inline]
    pub const fn to_u64(&self) -> u64 {
        assert!(BITS <= 64, "to_u64 requires BITS <= 64");
        let mut result = 0u64;
        let limit = if BYTES < 8 { BYTES } else { 8 };
        let mut i = 0;
        while i < limit {
            result |= (self.raw[i] as u64) << (i * 8);
            i += 1;
        }
        result
    }

    /// Converts this `UInt<BITS, BYTES>` into a standard 128-bit unsigned integer.
    #[inline]
    pub const fn to_u128(&self) -> u128 {
        assert!(BITS <= 128, "to_u128 requires BITS <= 128");
        let mut result = 0u128;
        let limit = if BYTES < 16 { BYTES } else { 16 };
        let mut i = 0;
        while i < limit {
            result |= (self.raw[i] as u128) << (i * 8);
            i += 1;
        }
        result
    }

    /// Creates a `UInt<BITS, BYTES>` from a `u64`, returning an error if the value exceeds $2^{\text{BITS}} - 1$.
    pub const fn from_u64(val: u64) -> Result<Self, OutOfRangeError> {
        assert!(BYTES > 0, "BYTES must be greater than 0");
        assert!(BITS > 0, "BITS must be greater than 0");
        assert!(BITS <= BYTES * 8, "BITS cannot exceed BYTES * 8");
        if BITS < 64 && val >= (1u64 << BITS) {
            return Err(OutOfRangeError::new(BITS));
        }
        let mut raw = [0u8; BYTES];
        let bytes_to_copy = if BYTES < 8 { BYTES } else { 8 };
        let val_bytes = val.to_le_bytes();
        let mut i = 0;
        while i < bytes_to_copy {
            raw[i] = val_bytes[i];
            i += 1;
        }
        Ok(Self { raw })
    }

    /// Creates a `UInt<BITS, BYTES>` from a `u128`, returning an error if the value exceeds $2^{\text{BITS}} - 1$.
    pub const fn from_u128(val: u128) -> Result<Self, OutOfRangeError> {
        assert!(BYTES > 0, "BYTES must be greater than 0");
        assert!(BITS > 0, "BITS must be greater than 0");
        assert!(BITS <= BYTES * 8, "BITS cannot exceed BYTES * 8");
        if BITS < 128 && val >= (1u128 << BITS) {
            return Err(OutOfRangeError::new(BITS));
        }
        let mut raw = [0u8; BYTES];
        let bytes_to_copy = if BYTES < 16 { BYTES } else { 16 };
        let val_bytes = val.to_le_bytes();
        let mut i = 0;
        while i < bytes_to_copy {
            raw[i] = val_bytes[i];
            i += 1;
        }
        Ok(Self { raw })
    }
}

impl<const BITS: usize, const BYTES: usize> Ord for UInt<BITS, BYTES> {
    fn cmp(&self, other: &Self) -> Ordering {
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

impl<const BITS: usize, const BYTES: usize> PartialOrd for UInt<BITS, BYTES> {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const BITS: usize, const BYTES: usize> Default for UInt<BITS, BYTES> {
    #[inline(always)]
    fn default() -> Self {
        Self::ZERO
    }
}

impl<const BITS: usize, const BYTES: usize> fmt::Debug for UInt<BITS, BYTES> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if BITS <= 64 {
            write!(f, "UInt<{}, {}>({})", BITS, BYTES, self.to_u64())
        } else if BITS <= 128 {
            write!(f, "UInt<{}, {}>({})", BITS, BYTES, self.to_u128())
        } else {
            let raw = self.raw;
            write!(f, "UInt<{}, {}>({:?})", BITS, BYTES, raw)
        }
    }
}

impl<const BITS: usize, const BYTES: usize> fmt::Display for UInt<BITS, BYTES> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if BITS <= 64 {
            write!(f, "{}", self.to_u64())
        } else if BITS <= 128 {
            write!(f, "{}", self.to_u128())
        } else {
            let raw = self.raw;
            write!(f, "0x")?;
            for b in raw.iter().rev() {
                write!(f, "{:02x}", b)?;
            }
            Ok(())
        }
    }
}

impl<const BITS: usize, const BYTES: usize> fmt::Binary for UInt<BITS, BYTES> {
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

impl<const BITS: usize, const BYTES: usize> fmt::LowerHex for UInt<BITS, BYTES> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if BITS <= 64 {
            fmt::LowerHex::fmt(&self.to_u64(), f)
        } else if BITS <= 128 {
            fmt::LowerHex::fmt(&self.to_u128(), f)
        } else {
            let raw = self.raw;
            for b in raw.iter().rev() {
                write!(f, "{:02x}", b)?;
            }
            Ok(())
        }
    }
}

impl<const BITS: usize, const BYTES: usize> fmt::UpperHex for UInt<BITS, BYTES> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if BITS <= 64 {
            fmt::UpperHex::fmt(&self.to_u64(), f)
        } else if BITS <= 128 {
            fmt::UpperHex::fmt(&self.to_u128(), f)
        } else {
            let raw = self.raw;
            for b in raw.iter().rev() {
                write!(f, "{:02X}", b)?;
            }
            Ok(())
        }
    }
}

impl<const BITS: usize, const BYTES: usize> TryFrom<u64> for UInt<BITS, BYTES> {
    type Error = OutOfRangeError;

    #[inline(always)]
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::from_u64(value)
    }
}

impl<const BITS: usize, const BYTES: usize> TryFrom<u128> for UInt<BITS, BYTES> {
    type Error = OutOfRangeError;

    #[inline(always)]
    fn try_from(value: u128) -> Result<Self, Self::Error> {
        Self::from_u128(value)
    }
}

#[cfg(feature = "serde")]
struct UIntByteVisitor<const BYTES: usize>;

#[cfg(feature = "serde")]
impl<'de, const BYTES: usize> serde::de::Visitor<'de> for UIntByteVisitor<BYTES> {
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
impl<const BITS: usize, const BYTES: usize> serde::Serialize for UInt<BITS, BYTES> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if BITS <= 64 {
            serializer.serialize_u64(self.to_u64())
        } else if BITS <= 128 {
            serializer.serialize_u128(self.to_u128())
        } else {
            let raw = self.raw;
            serializer.serialize_bytes(&raw)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de, const BITS: usize, const BYTES: usize> serde::Deserialize<'de> for UInt<BITS, BYTES> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if BITS <= 64 {
            let val = u64::deserialize(deserializer)?;
            Self::from_u64(val).map_err(serde::de::Error::custom)
        } else if BITS <= 128 {
            let val = u128::deserialize(deserializer)?;
            Self::from_u128(val).map_err(serde::de::Error::custom)
        } else {
            let raw = deserializer.deserialize_bytes(UIntByteVisitor::<BYTES>)?;
            Self::from_le_bytes(raw).map_err(serde::de::Error::custom)
        }
    }
}
