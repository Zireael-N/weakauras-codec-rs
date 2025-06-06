// Based on a research done by Wojciech Muła and Daniel Lemire
// https://arxiv.org/abs/1704.00605
// Copyright 2015-2016 Wojciech Muła, Alfred Klomp, Daniel Lemire
// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: BSD-2-Clause

use super::scalar;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// SAFETY: the caller must ensure that `output` can hold AT LEAST `(input.len() * 4 + 2) / 3` more elements
#[allow(unsafe_op_in_unsafe_fn)]
#[cfg(target_feature = "ssse3")]
#[inline]
pub unsafe fn encode_into_unchecked(input: &[u8], output: &mut String) {
    let mut len = input.len();
    let mut out_len = output.len();

    let mut ptr = input.as_ptr();
    let mut out_ptr = output[out_len..].as_mut_ptr();

    let shuf = _mm_set_epi8(10, 9, 11, 10, 7, 6, 8, 7, 4, 3, 5, 4, 1, 0, 2, 1);

    while len >= 16 {
        // input = [ffeeeeee|ddddddcc|ccccbbbb|bbaaaaaa]

        // SAFETY: There's at least 16 bytes of input left.
        // src = [ccccbbbb|bbaaaaaa|ddddddcc|ccccbbbb]
        let src = _mm_shuffle_epi8(unsafe { _mm_loadu_si128(ptr.cast()) }, shuf);

        // t0 = [00000000|00aaaaaa|000000cc|cccc0000]
        let t0 = _mm_and_si128(src, _mm_set1_epi32(0x003f03f0));
        // t1 = [00aaaaaa|00000000|00cccccc|00000000]
        let t1 = _mm_mullo_epi16(t0, _mm_set1_epi32(0x01000010));
        // t2 = [0000bbbb|bb000000|dddddd00|00000000]
        let t2 = _mm_and_si128(src, _mm_set1_epi32(0x0fc0fc00));
        // t3 = [00000000|00bbbbbb|00000000|00dddddd]
        let t3 = _mm_mulhi_epu16(t2, _mm_set1_epi32(0x04000040));

        // t4 = [00aaaaaa|00bbbbbb|00cccccc|00dddddd]
        let t4 = _mm_or_si128(t1, t3);
        // indices = [00dddddd|00cccccc|00bbbbbb|00aaaaaa]
        let indices = _mm_shuffle_epi8(
            t4,
            _mm_set_epi8(12, 13, 14, 15, 8, 9, 10, 11, 4, 5, 6, 7, 0, 1, 2, 3),
        );

        // _mm_subs_epu8 causes values from ranges [0..25] AND [26..51] to be mapped to 0
        // and values in [52..63] to be mapped to [1..12].
        // Then values in [0..25] are readjusted to be mapped to 13.
        let mut result = _mm_or_si128(
            _mm_subs_epu8(indices, _mm_set1_epi8(51)),
            _mm_and_si128(
                _mm_cmpgt_epi8(_mm_set1_epi8(26), indices),
                _mm_set1_epi8(13),
            ),
        );

        //   #  From        Index       To           Delta    Characters
        //   1  [0..25]     13          [97..122]    +97      a..z
        //   2  [26..51]    0           [65..90]     +39      A..Z
        //   3  [52..61]    [1..10]     [48..57]      -4      0..9
        //   4  [62..63]    [11..12]    [40..41]     -22      (,)
        let offsets = _mm_setr_epi8(
            39, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -22, -22, 97, 0, 0,
        );

        result = _mm_add_epi8(_mm_shuffle_epi8(offsets, result), indices);

        // SAFETY: As long as the caller upheld the safety contract,
        // we are at least 16 bytes away from the end of the output.
        unsafe {
            _mm_storeu_si128(out_ptr.cast(), result);
            out_ptr = out_ptr.add(16);
            out_len += 16;

            ptr = ptr.add(12);
            len -= 12;
        }
    }
    unsafe { output.as_mut_vec().set_len(out_len) };

    // SAFETY: Scalar version relies on the same safety contract.
    // The slice is guaranteed to be correct as long as the caller upheld it.
    unsafe { scalar::encode_into_unchecked(core::slice::from_raw_parts(ptr, len), output) }
}
