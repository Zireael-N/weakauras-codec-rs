// Based on a research done by Wojciech Muła and Daniel Lemire
// https://arxiv.org/abs/1704.00605
// Copyright 2015-2016 Wojciech Muła, Alfred Klomp, Daniel Lemire
// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: BSD-2-Clause

use crate::encode::scalar;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::mem::MaybeUninit;

// Refer to the reexport for documentation, crate::encode::encode_into_unchecked.
#[target_feature(enable = "avx2")]
pub unsafe fn encode_into_unchecked(input: &[u8], output: &mut [MaybeUninit<u8>]) -> usize {
    // Refer to the SSSE3 version for a detailed explanation.

    let mut len = input.len();
    let out_len = output.len();
    let mut written = 0;

    let mut ptr = input.as_ptr();
    let mut out_ptr = output.as_mut_ptr();

    let shuf = _mm256_set_epi8(
        10, 9, 11, 10, 7, 6, 8, 7, 4, 3, 5, 4, 1, 0, 2, 1, 10, 9, 11, 10, 7, 6, 8, 7, 4, 3, 5, 4,
        1, 0, 2, 1,
    );

    while len >= 32 {
        // SAFETY: There's at least 32 bytes of input left.
        let lo = unsafe { _mm_loadu_si128(ptr.cast()) };
        let hi = unsafe { _mm_loadu_si128(ptr.add(12).cast()) };

        let src = _mm256_shuffle_epi8(_mm256_set_m128i(hi, lo), shuf);

        let t1 = _mm256_mullo_epi16(
            _mm256_and_si256(src, _mm256_set1_epi32(0x003f03f0)),
            _mm256_set1_epi32(0x01000010),
        );
        let t2 = _mm256_mulhi_epu16(
            _mm256_and_si256(src, _mm256_set1_epi32(0x0fc0fc00)),
            _mm256_set1_epi32(0x04000040),
        );

        let indices = _mm256_shuffle_epi8(
            _mm256_or_si256(t1, t2),
            _mm256_set_epi8(
                12, 13, 14, 15, 8, 9, 10, 11, 4, 5, 6, 7, 0, 1, 2, 3, 12, 13, 14, 15, 8, 9, 10, 11,
                4, 5, 6, 7, 0, 1, 2, 3,
            ),
        );

        let mut result = _mm256_or_si256(
            _mm256_subs_epu8(indices, _mm256_set1_epi8(51)),
            _mm256_and_si256(
                _mm256_cmpgt_epi8(_mm256_set1_epi8(26), indices),
                _mm256_set1_epi8(13),
            ),
        );

        let offsets = _mm256_setr_epi8(
            39, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -22, -22, 97, 0, 0, 39, -4, -4, -4, -4, -4,
            -4, -4, -4, -4, -4, -22, -22, 97, 0, 0,
        );

        result = _mm256_add_epi8(_mm256_shuffle_epi8(offsets, result), indices);

        // SAFETY: As long as the caller upheld the safety contract,
        // we are at least 32 bytes away from the end of the output.
        unsafe {
            _mm256_storeu_si256(out_ptr.cast(), result);
            out_ptr = out_ptr.add(32);
            written += 32;

            ptr = ptr.add(24);
            len -= 24;
        }
    }

    // SAFETY: Scalar version relies on the same safety contract.
    // Slices are guaranteed to be correct as long as the caller upheld it.
    written
        + unsafe {
            scalar::encode_into_unchecked(
                core::slice::from_raw_parts(ptr, len),
                core::slice::from_raw_parts_mut(out_ptr, out_len - written),
            )
        }
}
