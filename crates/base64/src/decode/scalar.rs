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
