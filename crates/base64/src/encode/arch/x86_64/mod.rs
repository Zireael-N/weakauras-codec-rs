// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

pub mod avx2;
pub mod ssse3;

#[allow(unused_imports)]
use crate::{encode::scalar, macros::unsafe_runtime_dispatch};
#[allow(unused_imports)]
use core::mem::MaybeUninit;

#[cfg(target_feature = "avx2")]
pub use avx2::encode_into_unchecked;

/// SAFETY: the caller must ensure that `output`'s length is AT LEAST `(input.len() * 4 + 2) / 3`
#[cfg(all(target_feature = "ssse3", not(target_feature = "avx2")))]
#[inline(always)]
pub unsafe fn encode_into_unchecked(input: &[u8], output: &mut [MaybeUninit<u8>]) -> usize {
    unsafe_runtime_dispatch!(
        encode_into_unchecked,
        usize,
        input,
        output,
        is_x86_feature_detected,
        ("avx2", avx2),
        ssse3,
    )
}

/// SAFETY: the caller must ensure that `output`'s length is AT LEAST `(input.len() * 4 + 2) / 3`
#[cfg(not(any(target_feature = "ssse3", target_feature = "avx2")))]
#[inline(always)]
pub unsafe fn encode_into_unchecked(input: &[u8], output: &mut [MaybeUninit<u8>]) -> usize {
    unsafe_runtime_dispatch!(
        encode_into_unchecked,
        usize,
        input,
        output,
        is_x86_feature_detected,
        ("avx2", avx2),
        ("ssse3", ssse3),
        scalar,
    )
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::encode::tests::*;

    use alloc::{string::String, vec::Vec};

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
