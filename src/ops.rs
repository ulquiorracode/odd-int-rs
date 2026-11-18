//! Arithmetic and bitwise operator implementations for arbitrary bit-width integers.

use crate::int::Int;
use crate::uint::UInt;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

// ============================================================================
// UInt Bitwise Ops
// ============================================================================

impl<const BITS: usize, const BYTES: usize> BitAnd for UInt<BITS, BYTES> {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        let mut raw = [0u8; BYTES];
        let mut i = 0;
        while i < BYTES {
            raw[i] = self.raw[i] & rhs.raw[i];
            i += 1;
        }
        Self { raw }
    }
}

impl<const BITS: usize, const BYTES: usize> BitAndAssign for UInt<BITS, BYTES> {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl<const BITS: usize, const BYTES: usize> BitOr for UInt<BITS, BYTES> {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut raw = [0u8; BYTES];
        let mut i = 0;
        while i < BYTES {
            raw[i] = self.raw[i] | rhs.raw[i];
            i += 1;
        }
        Self { raw }
    }
}

impl<const BITS: usize, const BYTES: usize> BitOrAssign for UInt<BITS, BYTES> {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl<const BITS: usize, const BYTES: usize> BitXor for UInt<BITS, BYTES> {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut raw = [0u8; BYTES];
        let mut i = 0;
        while i < BYTES {
            raw[i] = self.raw[i] ^ rhs.raw[i];
            i += 1;
        }
        Self { raw }
    }
}

impl<const BITS: usize, const BYTES: usize> BitXorAssign for UInt<BITS, BYTES> {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl<const BITS: usize, const BYTES: usize> Not for UInt<BITS, BYTES> {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        let mut raw = [0u8; BYTES];
        let msb_index = (BITS - 1) / 8;
        let mut i = 0;
        while i <= msb_index {
            raw[i] = !self.raw[i];
            i += 1;
        }
        raw[msb_index] &= Self::HIGH_BYTE_MASK;
        Self { raw }
    }
}

// ============================================================================
// UInt Arithmetic Ops
// ============================================================================

impl<const BITS: usize, const BYTES: usize> UInt<BITS, BYTES> {
    /// Checked integer addition. Computes `self + rhs`, returning `None` if overflow occurred.
    #[inline]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        if BITS <= 64 {
            let res = self.to_u64().checked_add(rhs.to_u64())?;
            Self::from_u64(res).ok()
        } else if BITS <= 128 {
            let res = self.to_u128().checked_add(rhs.to_u128())?;
            Self::from_u128(res).ok()
        } else {
            let msb_index = (BITS - 1) / 8;
            let mut raw = [0u8; BYTES];
            let mut carry = 0u16;
            let mut i = 0;
            while i <= msb_index {
                let sum = (self.raw[i] as u16) + (rhs.raw[i] as u16) + carry;
                raw[i] = sum as u8;
                carry = sum >> 8;
                i += 1;
            }
            if carry > 0 || (raw[msb_index] & !Self::HIGH_BYTE_MASK) != 0 {
                None
            } else {
                Some(Self { raw })
            }
        }
    }

    /// Checked integer subtraction. Computes `self - rhs`, returning `None` if overflow/underflow occurred.
    #[inline]
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        if BITS <= 64 {
            let res = self.to_u64().checked_sub(rhs.to_u64())?;
            Self::from_u64(res).ok()
        } else if BITS <= 128 {
            let res = self.to_u128().checked_sub(rhs.to_u128())?;
            Self::from_u128(res).ok()
        } else {
            let msb_index = (BITS - 1) / 8;
            let mut raw = [0u8; BYTES];
            let mut borrow = 0i16;
            let mut i = 0;
            while i <= msb_index {
                let diff = (self.raw[i] as i16) - (rhs.raw[i] as i16) - borrow;
                if diff < 0 {
                    raw[i] = (diff + 256) as u8;
                    borrow = 1;
                } else {
                    raw[i] = diff as u8;
                    borrow = 0;
                }
                i += 1;
            }
            if borrow > 0 { None } else { Some(Self { raw }) }
        }
    }

    /// Saturating integer addition. Computes `self + rhs`, saturating at the numeric bounds instead of overflowing.
    #[inline]
    pub fn saturating_add(self, rhs: Self) -> Self {
        self.checked_add(rhs).unwrap_or(Self::MAX)
    }

    /// Saturating integer subtraction. Computes `self - rhs`, saturating at the numeric bounds instead of underflowing.
    #[inline]
    pub fn saturating_sub(self, rhs: Self) -> Self {
        self.checked_sub(rhs).unwrap_or(Self::ZERO)
    }
}

// ============================================================================
// Int Bitwise & Arithmetic Ops
// ============================================================================

impl<const BITS: usize, const BYTES: usize> BitAnd for Int<BITS, BYTES> {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        let mut raw = [0u8; BYTES];
        let mut i = 0;
        while i < BYTES {
            raw[i] = self.raw[i] & rhs.raw[i];
            i += 1;
        }
        Self { raw }
    }
}

impl<const BITS: usize, const BYTES: usize> BitOr for Int<BITS, BYTES> {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut raw = [0u8; BYTES];
        let mut i = 0;
        while i < BYTES {
            raw[i] = self.raw[i] | rhs.raw[i];
            i += 1;
        }
        Self { raw }
    }
}

impl<const BITS: usize, const BYTES: usize> BitXor for Int<BITS, BYTES> {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut raw = [0u8; BYTES];
        let mut i = 0;
        while i < BYTES {
            raw[i] = self.raw[i] ^ rhs.raw[i];
            i += 1;
        }
        Self { raw }
    }
}

impl<const BITS: usize, const BYTES: usize> Not for Int<BITS, BYTES> {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        let mut raw = [0u8; BYTES];
        let msb_index = (BITS - 1) / 8;
        let mut i = 0;
        while i < BYTES {
            raw[i] = !self.raw[i];
            i += 1;
        }
        let is_negative = (raw[msb_index] & Self::SIGN_BIT_MASK) != 0;
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
        Self { raw }
    }
}

impl<const BITS: usize, const BYTES: usize> Int<BITS, BYTES> {
    /// Checked integer addition. Computes `self + rhs`, returning `None` if overflow occurred.
    #[inline]
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        if BITS <= 64 {
            let res = self.to_i64().checked_add(rhs.to_i64())?;
            Self::from_i64(res).ok()
        } else if BITS <= 128 {
            let res = self.to_i128().checked_add(rhs.to_i128())?;
            Self::from_i128(res).ok()
        } else {
            let msb_index = (BITS - 1) / 8;
            let mut raw = [0u8; BYTES];
            let mut carry = 0u16;
            let mut i = 0;
            while i <= msb_index {
                let sum = (self.raw[i] as u16) + (rhs.raw[i] as u16) + carry;
                raw[i] = sum as u8;
                carry = sum >> 8;
                i += 1;
            }

            let a_neg = self.is_negative();
            let b_neg = rhs.is_negative();
            let res_neg = (raw[msb_index] & Self::SIGN_BIT_MASK) != 0;

            // Two's complement overflow: positive + positive = negative OR negative + negative = positive
            if (a_neg == b_neg) && (a_neg != res_neg) {
                return None;
            }

            // Sign extend into unused high bits of msb byte and subsequent padding bytes
            if BITS % 8 != 0 {
                if res_neg {
                    raw[msb_index] |= !Self::HIGH_BYTE_MASK;
                } else {
                    raw[msb_index] &= Self::HIGH_BYTE_MASK;
                }
            }
            let mut j = msb_index + 1;
            while j < BYTES {
                raw[j] = if res_neg { 0xFF } else { 0x00 };
                j += 1;
            }

            Some(Self { raw })
        }
    }

    /// Checked integer subtraction. Computes `self - rhs`, returning `None` if overflow occurred.
    #[inline]
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        if BITS <= 64 {
            let res = self.to_i64().checked_sub(rhs.to_i64())?;
            Self::from_i64(res).ok()
        } else if BITS <= 128 {
            let res = self.to_i128().checked_sub(rhs.to_i128())?;
            Self::from_i128(res).ok()
        } else {
            let msb_index = (BITS - 1) / 8;
            let mut raw = [0u8; BYTES];
            let mut borrow = 0i16;
            let mut i = 0;
            while i <= msb_index {
                let diff = (self.raw[i] as i16) - (rhs.raw[i] as i16) - borrow;
                if diff < 0 {
                    raw[i] = (diff + 256) as u8;
                    borrow = 1;
                } else {
                    raw[i] = diff as u8;
                    borrow = 0;
                }
                i += 1;
            }

            let a_neg = self.is_negative();
            let b_neg = rhs.is_negative();
            let res_neg = (raw[msb_index] & Self::SIGN_BIT_MASK) != 0;

            // Overflow in subtraction: a - b where signs differ and result has opposite sign of a
            if (a_neg != b_neg) && (a_neg != res_neg) {
                return None;
            }

            if BITS % 8 != 0 {
                if res_neg {
                    raw[msb_index] |= !Self::HIGH_BYTE_MASK;
                } else {
                    raw[msb_index] &= Self::HIGH_BYTE_MASK;
                }
            }
            let mut j = msb_index + 1;
            while j < BYTES {
                raw[j] = if res_neg { 0xFF } else { 0x00 };
                j += 1;
            }

            Some(Self { raw })
        }
    }

    /// Saturating integer addition. Computes `self + rhs`, saturating at the numeric bounds instead of overflowing.
    #[inline]
    pub fn saturating_add(self, rhs: Self) -> Self {
        self.checked_add(rhs).unwrap_or_else(|| {
            if rhs.is_negative() {
                Self::MIN
            } else {
                Self::MAX
            }
        })
    }

    /// Saturating integer subtraction. Computes `self - rhs`, saturating at the numeric bounds instead of underflowing.
    #[inline]
    pub fn saturating_sub(self, rhs: Self) -> Self {
        self.checked_sub(rhs).unwrap_or_else(|| {
            if rhs.is_negative() {
                Self::MAX
            } else {
                Self::MIN
            }
        })
    }
}
