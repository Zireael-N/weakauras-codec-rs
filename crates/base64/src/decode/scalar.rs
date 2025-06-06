// Based on code from https://github.com/client9/stringencoders
// Copyright 2005-2016 Nick Galbreath
// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

use crate::byte_map::{BAD_SYMBOL, DECODE_LUT0, DECODE_LUT1, DECODE_LUT2, DECODE_LUT3};
use core::mem::MaybeUninit;

/// SAFETY: the caller must ensure that `output`'s length is AT LEAST `input.len() * 3 / 4`
#[inline]
pub unsafe fn decode_into_unchecked(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
) -> Result<usize, usize> {
    let mut chunks = input.chunks_exact(4);

    let mut ptr = output.as_mut_ptr().cast::<u8>();
    let mut written = 0;
    let mut read = 0;

    for chunk in chunks.by_ref() {
        written += 3;

        let word = DECODE_LUT0[chunk[0]]
            | DECODE_LUT1[chunk[1]]
            | DECODE_LUT2[chunk[2]]
            | DECODE_LUT3[chunk[3]];

        if word == BAD_SYMBOL {
            let invalid_byte_at = find_invalid_byte(chunk).unwrap();

            return Err(read + invalid_byte_at);
        }

        // SAFETY: As long as the caller upheld the safety contract,
        // we are at least 3 bytes away from the end of the output.
        unsafe {
            core::ptr::copy((&word as *const u32).cast(), ptr, 3);
            ptr = ptr.add(3);
        }

        read += 4;
    }

    let remainder = chunks.remainder();
    match remainder.len() {
        3 => {
            written += 2;

            let word =
                DECODE_LUT0[remainder[0]] | DECODE_LUT1[remainder[1]] | DECODE_LUT2[remainder[2]];

            if word == BAD_SYMBOL {
                let invalid_byte_at = find_invalid_byte(remainder).unwrap();

                return Err(read + invalid_byte_at);
            }

            // SAFETY: As long as the caller upheld the safety contract,
            // we are at least 2 bytes away from the end of the output.
            unsafe {
                core::ptr::copy((&word as *const u32).cast(), ptr, 2);
            }
        }
        2 => {
            written += 1;

            let word = DECODE_LUT0[remainder[0]] | DECODE_LUT1[remainder[1]];

            if word == BAD_SYMBOL {
                let invalid_byte_at = find_invalid_byte(remainder).unwrap();

                return Err(read + invalid_byte_at);
            }

            // SAFETY: As long as the caller upheld the safety contract,
            // we are at least a byte away from the end of the output.
            unsafe {
                core::ptr::copy((&word as *const u32).cast(), ptr, 1);
            }
        }
        _ => {}
    }

    Ok(written)
}

pub(crate) fn find_invalid_byte(bytes: &[u8]) -> Option<usize> {
    bytes.iter().copied().position(|b| !is_valid_byte(b))
}

fn is_valid_byte(byte: u8) -> bool {
    matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'(' | b')')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decode::tests::*;

    #[test]
    fn scalar_returns_index_of_invalid_byte() {
        let test_cases = [
            (
                core::iter::once(b'=')
                    .chain(base64_iter().take(7))
                    .collect::<Vec<_>>(),
                0usize,
            ), // chunk #1
            (
                base64_iter()
                    .take(1)
                    .chain(core::iter::once(b'='))
                    .chain(base64_iter().take(6))
                    .collect::<Vec<_>>(),
                1,
            ), // chunk #1
            (
                base64_iter()
                    .take(4)
                    .chain(core::iter::once(b'='))
                    .chain(base64_iter().take(3))
                    .collect::<Vec<_>>(),
                4,
            ), // chunk #2
            (
                base64_iter()
                    .take(9)
                    .chain(core::iter::once(b'='))
                    .collect::<Vec<_>>(),
                9,
            ), // remainder.len() == 2
            (
                base64_iter()
                    .take(9)
                    .chain(core::iter::once(b'='))
                    .chain(base64_iter().take(1))
                    .collect::<Vec<_>>(),
                9,
            ), // remainder.len() == 3
        ];

        for (data, invalid_byte_at) in test_cases {
            let capacity = data.len() * 3 / 4;
            let mut buf = Vec::with_capacity(capacity);

            let result = unsafe { decode_into_unchecked(&data, buf.spare_capacity_mut()) };

            assert_eq!(result, Err(invalid_byte_at));
        }
    }
}
