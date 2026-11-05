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

pub mod error;
pub mod ops;
pub mod uint;

pub use error::OutOfRangeError;
pub use uint::UInt;

// Ergonomic type aliases for all standard odd bit-widths with computed byte sizes:
pub type U1 = UInt<1, 1>;
pub type U2 = UInt<2, 1>;
pub type U3 = UInt<3, 1>;
pub type U4 = UInt<4, 1>;
pub type U5 = UInt<5, 1>;
pub type U6 = UInt<6, 1>;
pub type U7 = UInt<7, 1>;
pub type U8 = UInt<8, 1>;

pub type U9 = UInt<9, 2>;
pub type U10 = UInt<10, 2>;
pub type U11 = UInt<11, 2>;
pub type U12 = UInt<12, 2>;
pub type U13 = UInt<13, 2>;
pub type U14 = UInt<14, 2>;
pub type U15 = UInt<15, 2>;
pub type U16 = UInt<16, 2>;

pub type U17 = UInt<17, 3>;
pub type U24 = UInt<24, 3>;
pub type U32 = UInt<32, 4>;
pub type U40 = UInt<40, 5>;
pub type U48 = UInt<48, 6>;
pub type U56 = UInt<56, 7>;
pub type U64 = UInt<64, 8>;
pub type U128 = UInt<128, 16>;

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
}
