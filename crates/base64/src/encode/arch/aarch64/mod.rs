// Copyright 2025 Velithris
// SPDX-License-Identifier: MIT

pub mod neon;

#[allow(unused_imports)]
use crate::encode::scalar;
#[allow(unused_imports)]
use core::mem::MaybeUninit;

#[cfg(target_feature = "neon")]
pub use neon::encode_into_unchecked;

#[cfg(not(target_feature = "neon"))]
pub use scalar::encode_into_unchecked;

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::encode::tests::*;

    use alloc::{string::String, vec::Vec};

    #[test]
    #[cfg(target_feature = "neon")]
    fn scalar_and_neon_return_same_values() {
        let data: Vec<u8> = (0..=255).cycle().take(1024 * 30 + 3).collect();

        let capacity = (data.len() * 4 + 2) / 3;
        let mut buf1 = String::with_capacity(capacity);
        let mut buf2 = String::with_capacity(capacity);

        unsafe {
            base64_encode!(&data, buf1, scalar);
            base64_encode!(&data, buf2, neon);
        }

        assert_eq!(buf1, buf2);
    }
}
