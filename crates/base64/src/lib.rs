// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

mod byte_map;
pub mod decode;
pub mod encode;
pub use decode::decode_to_vec;
pub use encode::{encode_to_string, encode_to_string_with_prefix};

#[cfg(feature = "expose_internals")]
pub use decode::*;
#[cfg(feature = "expose_internals")]
pub use encode::*;
