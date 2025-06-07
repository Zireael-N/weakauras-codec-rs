// Copyright 2025 Velithris
// SPDX-License-Identifier: MIT
pub mod neon;

#[allow(unused_imports)]
use crate::decode::scalar;
#[allow(unused_imports)]
use core::mem::MaybeUninit;

#[cfg(target_feature = "neon")]
pub use neon::decode_into_unchecked;

#[cfg(not(target_feature = "neon"))]
pub use scalar::decode_into_unchecked;

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::*;
    use crate::decode::tests::*;

    use alloc::vec::Vec;

    #[test]
    #[cfg(target_feature = "neon")]
    fn scalar_and_neon_return_same_values() {
        let data: Vec<u8> = base64_iter().take(1024 * 1024 + 3).collect();

        let capacity = data.len() * 3 / 4;
        let mut buf1 = Vec::with_capacity(capacity);
        let mut buf2 = Vec::with_capacity(capacity);

        unsafe {
            let scalar_len =
                scalar::decode_into_unchecked(&data, buf1.spare_capacity_mut()).unwrap();
            buf1.set_len(scalar_len);

            let neon_len = neon::decode_into_unchecked(&data, buf2.spare_capacity_mut()).unwrap();
            buf2.set_len(neon_len);
        }

        assert_eq!(buf1, buf2);
    }

    #[test]
    #[cfg(target_feature = "neon")]
    fn neon_returns_index_of_invalid_byte() {
        let test_cases = [
            (
                core::iter::once(b'=')
                    .chain(base64_iter().take(159))
                    .collect::<Vec<_>>(),
                0usize,
            ), // iteration #1
            (
                base64_iter()
                    .take(1)
                    .chain(core::iter::once(b'='))
                    .chain(base64_iter().take(158))
                    .collect::<Vec<_>>(),
                1,
            ), // iteration #1
            (
                base64_iter()
                    .take(64)
                    .chain(core::iter::once(b'='))
                    .chain(base64_iter().take(95))
                    .collect::<Vec<_>>(),
                64,
            ), // iteration #2
            (
                base64_iter()
                    .take(128)
                    .chain(core::iter::once(b'='))
                    .chain(base64_iter().take(31))
                    .collect::<Vec<_>>(),
                128,
            ), // scalar
        ];

        for (data, invalid_byte_at) in test_cases {
            let capacity = data.len() * 3 / 4;
            let mut buf = Vec::with_capacity(capacity);

            let result = unsafe { neon::decode_into_unchecked(&data, buf.spare_capacity_mut()) };

            assert_eq!(result, Err(invalid_byte_at));
        }
    }
}
