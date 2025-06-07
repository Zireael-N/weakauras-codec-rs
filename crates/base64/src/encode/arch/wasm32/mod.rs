// Copyright 2025 Velithris
// SPDX-License-Identifier: MIT

pub mod simd128;

#[allow(unused_imports)]
use crate::encode::scalar;
#[allow(unused_imports)]
use core::mem::MaybeUninit;

#[cfg(target_feature = "simd128")]
pub use simd128::encode_into_unchecked;

#[cfg(not(target_feature = "simd128"))]
pub use scalar::encode_into_unchecked;

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::encode::tests::*;

    use alloc::{string::String, vec::Vec};

    #[test]
    #[cfg(target_feature = "simd128")]
    fn scalar_and_simd128_return_same_values() {
        let data: Vec<u8> = (0..=255).cycle().take(1024 * 30 + 3).collect();

        let capacity = (data.len() * 4 + 2) / 3;
        let mut buf1 = String::with_capacity(capacity);
        let mut buf2 = String::with_capacity(capacity);

        unsafe {
            base64_encode!(&data, buf1, scalar);
            base64_encode!(&data, buf2, simd128);
        }

        assert_eq!(buf1, buf2);
    }
}
