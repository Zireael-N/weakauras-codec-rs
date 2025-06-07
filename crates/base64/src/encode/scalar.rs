// Based on code from https://github.com/client9/stringencoders
// Copyright 2005-2016 Nick Galbreath
// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

use crate::byte_map::ENCODE_LUT;
use core::mem::MaybeUninit;

// Refer to the reexport for documentation, crate::encode::encode_into_unchecked.
#[inline]
pub unsafe fn encode_into_unchecked(input: &[u8], output: &mut [MaybeUninit<u8>]) -> usize {
    let mut chunks = input.chunks_exact(3);

    let mut ptr = output.as_mut_ptr().cast::<u8>();
    let mut written = 0;

    for chunk in chunks.by_ref() {
        written += 4;

        let b0 = chunk[0];
        let b1 = chunk[1];
        let b2 = chunk[2];

        // SAFETY: As long as the caller upheld the safety contract,
        // we are at least 4 bytes away from the end of the output.
        unsafe {
            ptr.write(ENCODE_LUT[b0]);
            ptr = ptr.add(1);
            ptr.write(ENCODE_LUT[(b0 >> 6) | (b1 << 2)]);
            ptr = ptr.add(1);
            ptr.write(ENCODE_LUT[(b1 >> 4) | (b2 << 4)]);
            ptr = ptr.add(1);
            ptr.write(ENCODE_LUT[b2 >> 2]);
            ptr = ptr.add(1);
        }
    }

    let remainder = chunks.remainder();
    match remainder.len() {
        2 => {
            written += 3;
            let b0 = remainder[0];
            let b1 = remainder[1];

            // SAFETY: As long as the caller upheld the safety contract,
            // we are at least 3 bytes away from the end of the output.
            unsafe {
                ptr.write(ENCODE_LUT[b0]);
                ptr = ptr.add(1);
                ptr.write(ENCODE_LUT[(b0 >> 6) | (b1 << 2)]);
                ptr = ptr.add(1);
                ptr.write(ENCODE_LUT[b1 >> 4]);
            }
        }
        1 => {
            written += 2;
            let b0 = remainder[0];

            // SAFETY: As long as the caller upheld the safety contract,
            // we are at least 2 bytes away from the end of the output.
            unsafe {
                ptr.write(ENCODE_LUT[b0]);
                ptr = ptr.add(1);
                ptr.write(ENCODE_LUT[b0 >> 6]);
            }
        }
        _ => {}
    }

    written
}
