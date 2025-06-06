// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

pub mod avx2;
pub mod sse41;

#[cfg(target_feature = "avx2")]
pub use avx2::decode_into_unchecked;

#[cfg(all(target_feature = "sse4.1", not(target_feature = "avx2")))]
pub use sse41::decode_into_unchecked;

#[cfg(not(any(target_feature = "sse4.1", target_feature = "avx2")))]
pub use crate::decode::scalar::decode_into_unchecked;

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::decode::{scalar, tests::*};

    #[test]
    #[cfg(target_feature = "sse4.1")]
    fn scalar_and_sse41_return_same_values() {
        let data: Vec<u8> = base64_iter().take(1024 * 1024 + 3).collect();

        let capacity = data.len() * 3 / 4;
        let mut buf1 = Vec::with_capacity(capacity);
        let mut buf2 = Vec::with_capacity(capacity);

        unsafe {
            let scalar_len =
                scalar::decode_into_unchecked(&data, buf1.spare_capacity_mut()).unwrap();
            buf1.set_len(scalar_len);

            let sse41_len = sse41::decode_into_unchecked(&data, buf2.spare_capacity_mut()).unwrap();
            buf2.set_len(sse41_len);
        }

        assert_eq!(buf1, buf2);
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn scalar_and_avx2_return_same_values() {
        let data: Vec<u8> = base64_iter().take(1024 * 1024 + 3).collect();

        let capacity = data.len() * 3 / 4;
        let mut buf1 = Vec::with_capacity(capacity);
        let mut buf2 = Vec::with_capacity(capacity);

        unsafe {
            let scalar_len =
                scalar::decode_into_unchecked(&data, buf1.spare_capacity_mut()).unwrap();
            buf1.set_len(scalar_len);

            let avx2_len = avx2::decode_into_unchecked(&data, buf2.spare_capacity_mut()).unwrap();
            buf2.set_len(avx2_len);
        }

        assert_eq!(buf1, buf2);
    }

    #[test]
    #[cfg(target_feature = "sse4.1")]
    fn sse41_returns_index_of_invalid_byte() {
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

            let result = unsafe { sse41::decode_into_unchecked(&data, buf.spare_capacity_mut()) };

            assert_eq!(result, Err(invalid_byte_at));
        }
    }

    #[test]
    #[cfg(target_feature = "avx2")]
    fn avx2_returns_index_of_invalid_byte() {
        let test_cases = [
            (
                core::iter::once(b'=')
                    .chain(base64_iter().take(79))
                    .collect::<Vec<_>>(),
                0usize,
            ), // iteration #1
            (
                base64_iter()
                    .take(1)
                    .chain(core::iter::once(b'='))
                    .chain(base64_iter().take(78))
                    .collect::<Vec<_>>(),
                1,
            ), // iteration #1
            (
                base64_iter()
                    .take(32)
                    .chain(core::iter::once(b'='))
                    .chain(base64_iter().take(47))
                    .collect::<Vec<_>>(),
                32,
            ), // iteration #2
            (
                base64_iter()
                    .take(70)
                    .chain(core::iter::once(b'='))
                    .chain(base64_iter().take(9))
                    .collect::<Vec<_>>(),
                70,
            ), // scalar
        ];

        for (data, invalid_byte_at) in test_cases {
            let capacity = data.len() * 3 / 4;
            let mut buf = Vec::with_capacity(capacity);

            let result = unsafe { avx2::decode_into_unchecked(&data, buf.spare_capacity_mut()) };

            assert_eq!(result, Err(invalid_byte_at));
        }
    }
}
