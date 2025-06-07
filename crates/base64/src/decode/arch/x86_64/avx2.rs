// Based on a research done by Wojciech Muła and Daniel Lemire
// https://arxiv.org/abs/1704.00605
// Copyright 2015-2016 Wojciech Muła, Alfred Klomp, Daniel Lemire
// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: BSD-2-Clause

use crate::decode::scalar;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::mem::MaybeUninit;

// Refer to the reexport for documentation, crate::decode::decode_into_unchecked.
#[target_feature(enable = "avx2")]
pub unsafe fn decode_into_unchecked(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
) -> Result<usize, usize> {
    // Refer to the SSE4.1 version for a detailed explanation,
    // the only difference is an extra call to _mm256_permutevar8x32_epi32
    // to merge two lanes in the end.

    let mut len = input.len();
    let out_len = output.len();
    let mut written = 0;

    let mut ptr = input.as_ptr();
    let mut out_ptr = output.as_mut_ptr();

    let lut_hi = _mm256_setr_epi8(
        0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
        0x10, 0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
        0x10, 0x10,
    );
    let lut_lo = _mm256_setr_epi8(
        0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x10, 0x10, 0x13, 0x1b, 0x1b, 0x1b, 0x1b,
        0x1b, 0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x10, 0x10, 0x13, 0x1b, 0x1b, 0x1b,
        0x1b, 0x1b,
    );
    let lut_roll = _mm256_setr_epi8(
        0, 22, 22, 4, -39, -39, -97, -97, 0, 0, 0, 0, 0, 0, 0, 0, 0, 22, 22, 4, -39, -39, -97, -97,
        0, 0, 0, 0, 0, 0, 0, 0,
    );

    let mask_lo_nibble = _mm256_set1_epi8(0x0f);

    // We'll be writing 32 bytes at a time, therefore we need to ensure that
    // there's still enough space in the output slice.
    // To do that, we check the remaining length of the input.
    // ceil(32 * 4 / 3) = 43
    while len >= 43 {
        // Lookup:
        // SAFETY: There's at least 32 bytes of input left.
        let src = unsafe { _mm256_loadu_si256(ptr.cast()) };
        let hi_nibbles = _mm256_and_si256(_mm256_srli_epi32(src, 4), mask_lo_nibble);
        let lo_nibbles = _mm256_and_si256(src, mask_lo_nibble);
        let lo = _mm256_shuffle_epi8(lut_lo, lo_nibbles);
        let hi = _mm256_shuffle_epi8(lut_hi, hi_nibbles);

        if _mm256_testz_si256(lo, hi) == 0 {
            // SAFETY: We were working on 32 bytes just now.
            let last_chunk = unsafe { core::slice::from_raw_parts(ptr, 256 / 8) };
            let invalid_byte_at = scalar::find_invalid_byte(last_chunk).unwrap();

            return Err(input.len() - len + invalid_byte_at);
        }

        let roll = _mm256_shuffle_epi8(lut_roll, hi_nibbles);

        // Packing:
        let merged =
            _mm256_maddubs_epi16(_mm256_add_epi8(src, roll), _mm256_set1_epi32(0x40014001));
        let swapped = _mm256_madd_epi16(merged, _mm256_set1_epi32(0x10000001));
        let shuffled = _mm256_shuffle_epi8(
            swapped,
            _mm256_setr_epi8(
                0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, -1, -1, -1, -1, 0, 1, 2, 4, 5, 6, 8, 9, 10,
                12, 13, 14, -1, -1, -1, -1,
            ),
        );
        let shuffled =
            _mm256_permutevar8x32_epi32(shuffled, _mm256_setr_epi32(0, 1, 2, 4, 5, 6, -1, -1));

        // SAFETY: As long as the caller upheld the safety contract,
        // we are at least 32 bytes away from the end of the output.
        unsafe {
            _mm256_storeu_si256(out_ptr.cast(), shuffled);
            out_ptr = out_ptr.add(24);
            written += 24;

            ptr = ptr.add(32);
            len -= 32;
        }
    }

    // SAFETY: Scalar version relies on the same safety contract.
    // Slices are guaranteed to be correct as long as the caller upheld it.
    let scalar_result = unsafe {
        scalar::decode_into_unchecked(
            core::slice::from_raw_parts(ptr, len),
            core::slice::from_raw_parts_mut(out_ptr, out_len - written),
        )
    };
    match scalar_result {
        Ok(scalar_written) => Ok(written + scalar_written),
        Err(invalid_byte_at) => Err(input.len() - len + invalid_byte_at),
    }
}
