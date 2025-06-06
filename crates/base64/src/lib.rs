// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

mod byte_map;
pub mod decode;
pub mod encode;
pub use decode::{decode_into, decode_into_unchecked, decode_to_vec};
pub use encode::{
    encode_into, encode_into_unchecked, encode_to_string, encode_to_string_with_prefix,
};
