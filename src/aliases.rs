//! Macro-based automatic generation of all integer bit-width aliases: `U1`..`U128` and `I1`..`I128`.
//!
//! Automatically computes byte count via `(BITS + 7) / 8` at compile time without manual repetition.

#[macro_export]
/// Type alias constructor macro for arbitrary bit-width unsigned integers.
///
/// Automatically computes the exact byte count at compile time via `(BITS + 7) / 8`.
///
/// # Example
/// ```rust
/// use odd_int::uint_t;
///
/// let x: uint_t!(24) = <uint_t!(24)>::from_u64(12345).unwrap();
/// assert_eq!(x.to_u64(), 12345);
/// ```
macro_rules! uint_t {
    ($bits:expr) => {
        $crate::UInt<{$bits}, { ($bits as usize).div_ceil(8) }>
    };
}

#[macro_export]
/// Type alias constructor macro for arbitrary bit-width signed integers.
///
/// Automatically computes the exact byte count at compile time via `BITS.div_ceil(8)`.
///
/// # Example
/// ```rust
/// use odd_int::int_t;
///
/// let x: int_t!(24) = <int_t!(24)>::from_i64(-42).unwrap();
/// assert_eq!(x.to_i64(), -42);
/// ```
macro_rules! int_t {
    ($bits:expr) => {
        $crate::Int<{$bits}, { ($bits as usize).div_ceil(8) }>
    };
}

macro_rules! define_aliases {
    ($($b:literal),* $(,)?) => {
        $(
            paste::paste! {
                #[doc = concat!("Unsigned integer with exact bit-width `", stringify!($b), "` occupying `", stringify!(($b_usize.div_ceil(8))), "` bytes.")]
                pub type [<U $b>] = $crate::UInt<$b, { ($b as usize).div_ceil(8) }>;

                #[doc = concat!("Signed integer with exact bit-width `", stringify!($b), "` occupying `", stringify!(($b_usize.div_ceil(8))), "` bytes.")]
                pub type [<I $b>] = $crate::Int<$b, { ($b as usize).div_ceil(8) }>;
            }
        )*
    };
}

define_aliases!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74,
    75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98,
    99, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117,
    118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128,
    // Standard extended bit-widths for cryptographic, hashing, and SIMD applications:
    256, 512, 1024, 2048, 4096
);
