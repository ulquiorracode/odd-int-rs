# odd-int-rs

[![Crates.io](https://img.shields.io/crates/v/odd-int.svg)](https://crates.io/crates/odd-int)
[![Documentation](https://docs.rs/odd-int/badge.svg)](https://docs.rs/odd-int)
[![CI](https://github.com/ulquiorracode/odd-int-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/ulquiorracode/odd-int-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

Arbitrary bit-width unsigned integers with exact byte footprints, zero heap allocations, and `#![no_std]` support.

Conforms to the **"Scrooge" Systems Mindset** (zero memory waste, mechanical sympathy).

---

## Features

- **Exact Byte Sizing**: An integer of `BITS` bits consumes **strictly `(BITS + 7) / 8` bytes** in memory without compiler padding holes (`#[repr(C, packed)]`).
  - `U3` occupies **1 byte** (domain $0..=7$)
  - `U11` occupies **2 bytes** (domain $0..=2047$)
  - `U24` occupies **3 bytes** (domain $0..=16\,777\,215$)
  - `U48` occupies **6 bytes** (e.g. MAC address, 48-bit virtual address pointers)
- **1-Byte Alignment**: Struct alignment is guaranteed to be 1 byte (`align_of == 1`), ideal for zero-copy memory mapping (`mmap`), binary network protocols, and packet headers.
- **Strict Bounds Invariants**: Unused high bits in the most significant byte are guaranteed to be zero.
- **`#![no_std]` by Default**: Zero heap allocations, usable in embedded firmware, OS kernels, and WASM runtimes.
- **Ergonomic Arithmetic**: `checked_add`, `checked_sub`, `saturating_add`, `saturating_sub`, bitwise operations (`&`, `|`, `^`, `!`), and formatting (`Display`, `LowerHex`, `UpperHex`, `Binary`).

---

## Quick Start

```rust
use odd_int::{U3, U11, U24, U48};

// Construct within valid bounds
let val = U11::from_u64(1024).unwrap();
assert_eq!(val.to_u64(), 1024);

// Exceeding domain returns an OutOfRangeError
assert!(U11::from_u64(2048).is_err());

// Exact byte sizes:
assert_eq!(core::mem::size_of::<U3>(), 1);   // 3 bits -> 1 byte
assert_eq!(core::mem::size_of::<U11>(), 2);  // 11 bits -> 2 bytes
assert_eq!(core::mem::size_of::<U24>(), 3);  // 24 bits -> 3 bytes
assert_eq!(core::mem::size_of::<U48>(), 6);  // 48 bits -> 6 bytes
```

---

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
