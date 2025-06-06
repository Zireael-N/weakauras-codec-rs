// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

pub mod avx2;
pub mod ssse3;

#[cfg(target_feature = "avx2")]
pub use avx2::encode_into_unchecked;

#[cfg(all(target_feature = "ssse3", not(target_feature = "avx2")))]
pub use ssse3::encode_into_unchecked;

#[cfg(not(any(target_feature = "ssse3", target_feature = "avx2")))]
pub use crate::encode::scalar::encode_into_unchecked;

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::encode::{scalar, tests::*};

    #[test]
    #[cfg(target_feature = "ssse3")]
    fn scalar_and_ssse3_return_same_values() {
        let data: Vec<u8> = (0..=255).cycle().take(1024 * 30 + 3).collect();

        let capacity = (data.len() * 4 + 2) / 3;
        let mut buf1 = String::with_capacity(capacity);
        let mut buf2 = String::with_capacity(capacity);

        unsafe {
            base64_encode!(&data, buf1, scalar);
            base64_encode!(&data, buf2, ssse3);
        }

        assert_eq!(buf1, buf2);
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn scalar_and_avx2_return_same_values() {
        let data: Vec<u8> = (0..=255).cycle().take(1024 * 30 + 3).collect();

        let capacity = (data.len() * 4 + 2) / 3;
        let mut buf1 = String::with_capacity(capacity);
        let mut buf2 = String::with_capacity(capacity);

        unsafe {
            base64_encode!(&data, buf1, scalar);
            base64_encode!(&data, buf2, avx2);
        }

        assert_eq!(buf1, buf2);
    }
}
