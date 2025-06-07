// Copyright 2025 Velithris
// SPDX-License-Identifier: MIT

pub mod simd128;

#[allow(unused_imports)]
use crate::decode::scalar;
#[allow(unused_imports)]
use core::mem::MaybeUninit;

#[cfg(target_feature = "simd128")]
pub use simd128::decode_into_unchecked;

#[cfg(not(target_feature = "simd128"))]
pub use scalar::decode_into_unchecked;

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::decode::tests::*;

    use alloc::vec::Vec;

    #[test]
    #[cfg(target_feature = "simd128")]
    fn scalar_and_simd128_return_same_values() {
        let data: Vec<u8> = base64_iter().take(1024 * 1024 + 3).collect();

        let capacity = data.len() * 3 / 4;
        let mut buf1 = Vec::with_capacity(capacity);
        let mut buf2 = Vec::with_capacity(capacity);

        unsafe {
            let scalar_len =
                scalar::decode_into_unchecked(&data, buf1.spare_capacity_mut()).unwrap();
            buf1.set_len(scalar_len);

            let simd128_len =
                simd128::decode_into_unchecked(&data, buf2.spare_capacity_mut()).unwrap();
            buf2.set_len(simd128_len);
        }

        assert_eq!(buf1, buf2);
    }

    #[test]
    #[cfg(target_feature = "simd128")]
    fn simd128_returns_index_of_invalid_byte() {
        let test_cases = [
            (
                core::iter::once(b'=')
                    .chain(base64_iter().take(39))
                    .collect::<Vec<_>>(),
                0usize,
            ), // iteration #1
            (
                base64_iter()
                    .take(1)
                    .chain(core::iter::once(b'='))
                    .chain(base64_iter().take(38))
                    .collect::<Vec<_>>(),
                1,
            ), // iteration #1
            (
                base64_iter()
                    .take(16)
                    .chain(core::iter::once(b'='))
                    .chain(base64_iter().take(23))
                    .collect::<Vec<_>>(),
                16,
            ), // iteration #2
            (
                base64_iter()
                    .take(35)
                    .chain(core::iter::once(b'='))
                    .chain(base64_iter().take(4))
                    .collect::<Vec<_>>(),
                35,
            ), // scalar
        ];

        for (data, invalid_byte_at) in test_cases {
            let capacity = data.len() * 3 / 4;
            let mut buf = Vec::with_capacity(capacity);

            let result = unsafe { simd128::decode_into_unchecked(&data, buf.spare_capacity_mut()) };

            assert_eq!(result, Err(invalid_byte_at));
        }
    }
}
