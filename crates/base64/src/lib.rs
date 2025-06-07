// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

#![no_std]

#[cfg(any(test, feature = "std"))]
extern crate std;

#[cfg(any(test, feature = "alloc"))]
extern crate alloc;

mod byte_map;
pub mod decode;
pub mod encode;
pub(crate) mod macros;
pub use decode::{decode_into, decode_into_unchecked};
pub use encode::{encode_into, encode_into_unchecked};

#[cfg(feature = "alloc")]
pub use decode::decode_to_vec;
#[cfg(feature = "alloc")]
pub use encode::{encode_to_string, encode_to_string_with_prefix};
