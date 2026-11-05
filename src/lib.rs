//! # odd-int
//!
//! Arbitrary bit-width integer types with exact byte footprint, zero heap allocations,
//! and `#![no_std]` support.
//!
//! Conforms to the "Scrooge" Systems Mindset:
//! - Memory footprint: `size_of::<UInt<BITS, BYTES>>() == BYTES` bytes.
//! - Alignment: 1 byte (`align_of::<UInt<BITS, BYTES>>() == 1`).
//! - Strictly enforced bounds: $[0, 2^{\text{BITS}} - 1]$.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

pub mod aliases;
pub mod error;
pub mod int;
pub mod ops;
pub mod uint;

pub use aliases::*;
pub use error::OutOfRangeError;
pub use int::Int;
pub use uint::UInt;

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{align_of, size_of};

    #[test]
    fn test_exact_byte_size_and_alignment() {
        // Sub-byte odd integers fit in 1 byte
        assert_eq!(size_of::<U1>(), 1);
        assert_eq!(size_of::<U3>(), 1);
        assert_eq!(size_of::<U7>(), 1);
        assert_eq!(size_of::<U8>(), 1);

        // 9-16 bits fit in 2 bytes
        assert_eq!(size_of::<U9>(), 2);
        assert_eq!(size_of::<U11>(), 2);
        assert_eq!(size_of::<U16>(), 2);

        // 24 bits fit in 3 bytes
        assert_eq!(size_of::<U24>(), 3);

        // 40 bits fit in 5 bytes
        assert_eq!(size_of::<U40>(), 5);

        // 48 bits fit in 6 bytes (e.g. MAC address, 48-bit virtual address)
        assert_eq!(size_of::<U48>(), 6);

        // Alignment is always 1 byte
        assert_eq!(align_of::<U3>(), 1);
        assert_eq!(align_of::<U24>(), 1);
        assert_eq!(align_of::<U48>(), 1);
    }

    #[test]
    fn test_value_domain_and_limits() {
        // U3: range 0..=7
        let max_u3 = U3::MAX;
        assert_eq!(max_u3.to_u64(), 7);
        assert!(U3::from_u64(7).is_ok());
        assert_eq!(U3::from_u64(8), Err(OutOfRangeError::new(3)));

        // U11: range 0..=2047
        let max_u11 = U11::MAX;
        assert_eq!(max_u11.to_u64(), 2047);
        assert!(U11::from_u64(2047).is_ok());
        assert_eq!(U11::from_u64(2048), Err(OutOfRangeError::new(11)));

        // U24: range 0..=16_777_215
        let max_u24 = U24::MAX;
        assert_eq!(max_u24.to_u64(), 0xFF_FFFF);
        assert!(U24::from_u64(0xFF_FFFF).is_ok());
        assert_eq!(U24::from_u64(0x100_0000), Err(OutOfRangeError::new(24)));

        // U48: range 0..=281_474_976_710_655
        let max_u48 = U48::MAX;
        assert_eq!(max_u48.to_u64(), 0xFFFF_FFFF_FFFF);
        assert!(U48::from_u64(0xFFFF_FFFF_FFFF).is_ok());
        assert_eq!(
            U48::from_u64(0x1_0000_0000_0000),
            Err(OutOfRangeError::new(48))
        );
    }

    #[test]
    fn test_bitwise_operations() {
        let a = U11::from_u64(0b101_0101_0101).unwrap();
        let b = U11::from_u64(0b010_1010_1010).unwrap();

        assert_eq!((a & b).to_u64(), 0);
        assert_eq!((a | b).to_u64(), 0b111_1111_1111);
        assert_eq!((a ^ b).to_u64(), 0b111_1111_1111);
        assert_eq!((!a).to_u64(), 0b010_1010_1010);
    }

    #[test]
    fn test_checked_and_saturating_arithmetic() {
        let max = U11::MAX;
        let one = U11::from_u64(1).unwrap();

        assert_eq!(max.checked_add(one), None);
        assert_eq!(max.saturating_add(one), max);

        let zero = U11::ZERO;
        assert_eq!(zero.checked_sub(one), None);
        assert_eq!(zero.saturating_sub(one), zero);

        let five = U11::from_u64(5).unwrap();
        let six = U11::from_u64(6).unwrap();
        assert_eq!(five.checked_add(six).unwrap().to_u64(), 11);
    }

    #[test]
    fn test_signed_int_exact_sizes_and_sign_extension() {
        assert_eq!(size_of::<I8>(), 1);
        assert_eq!(size_of::<I16>(), 2);
        assert_eq!(size_of::<I24>(), 3);
        assert_eq!(size_of::<I32>(), 4);
        assert_eq!(size_of::<I40>(), 5);
        assert_eq!(size_of::<I48>(), 6);
        assert_eq!(size_of::<I64>(), 8);
        assert_eq!(size_of::<U128>(), 16);
        assert_eq!(size_of::<I128>(), 16);
        assert_eq!(size_of::<U17>(), 3);
        assert_eq!(size_of::<I17>(), 3);
        assert_eq!(size_of::<U33>(), 5);
        assert_eq!(size_of::<I33>(), 5);
        assert_eq!(size_of::<uint_t!(47)>(), 6);
        assert_eq!(size_of::<int_t!(47)>(), 6);

        // Sign extension for I24: range [-8_388_608, 8_388_607]
        let neg_one = I24::from_i64(-1).unwrap();
        assert_eq!(neg_one.to_i64(), -1);
        assert_eq!(neg_one.to_isize(), -1);
        assert_eq!(neg_one.to_le_bytes(), [0xFF, 0xFF, 0xFF]);

        let min_i24 = I24::from_i64(-8_388_608).unwrap();
        assert_eq!(min_i24.to_i64(), -8_388_608);

        let max_i24 = I24::from_i64(8_388_607).unwrap();
        assert_eq!(max_i24.to_i64(), 8_388_607);

        // Bounds checks
        assert!(I24::from_i64(-8_388_609).is_err());
        assert!(I24::from_i64(8_388_608).is_err());

        // Negative small value
        let neg_42 = I24::from_i64(-42).unwrap();
        assert_eq!(neg_42.to_i64(), -42);
        assert_eq!(neg_42.to_isize(), -42);
    }
}
