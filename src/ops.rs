//! Arithmetic and bitwise operator implementations for `UInt<const BITS: usize, const BYTES: usize>`.

use crate::uint::UInt;
use core::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

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
        let mut i = 0;
        while i < BYTES {
            raw[i] = !self.raw[i];
            i += 1;
        }
        if BYTES > 0 {
            raw[BYTES - 1] &= Self::HIGH_BYTE_MASK;
        }
        Self { raw }
    }
}

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
            let mut raw = [0u8; BYTES];
            let mut carry = 0u16;
            let mut i = 0;
            while i < BYTES {
                let sum = (self.raw[i] as u16) + (rhs.raw[i] as u16) + carry;
                raw[i] = sum as u8;
                carry = sum >> 8;
                i += 1;
            }
            if carry > 0 || (raw[BYTES - 1] & !Self::HIGH_BYTE_MASK) != 0 {
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
            let mut raw = [0u8; BYTES];
            let mut borrow = 0i16;
            let mut i = 0;
            while i < BYTES {
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
