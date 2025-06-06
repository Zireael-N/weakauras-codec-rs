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
use core::mem::MaybeUninit;

/// SAFETY: the caller must ensure that `output`'s length is AT LEAST `input.len() * 3 / 4`
#[allow(unsafe_op_in_unsafe_fn)]
#[cfg(target_feature = "sse4.1")]
#[inline]
pub unsafe fn decode_into_unchecked(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
) -> Result<usize, &'static str> {
    let mut len = input.len();
    let out_len = output.len();
    let mut written = 0;

    let mut ptr = input.as_ptr();
    let mut out_ptr = output.as_mut_ptr();

    //   #  High        Low        Bit
    //   1  2           [8..9]     0x01
    //   2  3           [0..9]     0x02
    //   3  4, 6        [1..15]    0x04
    //   4  5, 7        [0..10]    0x08
    //  (5) The rest    Invalid    0x10
    let lut_hi = _mm_setr_epi8(
        0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
        0x10,
    );

    // This one maps a low nibble to a bitwise OR
    // of bits corresponding to invalid high nibbles:
    //   #  Low         High       Bitset
    //   1  0           3, 5, 7    0x15 = 0x10 | 0x01 (2) | 0x04 (4 and 6)
    //   2  [1..7]      [3..7]     0x11 = 0x10 | 0x01 (2)
    //   3  [8..9]      [2..7]     0x10
    //   4  10          [4..7]     0x13 = 0x10 | 0x01 (2) | 0x02 (3)
    //   5  [11..15]    4, 6       0x1b = 0x10 | 0x01 (2) | 0x02 (3) | 0x08 (5 and 7)
    let lut_lo = _mm_setr_epi8(
        0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x10, 0x10, 0x13, 0x1b, 0x1b, 0x1b, 0x1b,
        0x1b,
    );

    //   #  From         To          Delta    Characters
    //   1  [40..41]     [62..63]    +22      (,)
    //   2  [48..57]     [52..61]     +4      0..9
    //   3  [65..90]     [26..51]    -39      A..Z
    //   4  [97..122]    [0..25]     -97      a..z
    //  (5) Everything else => invalid input
    let lut_roll = _mm_setr_epi8(0, 22, 22, 4, -39, -39, -97, -97, 0, 0, 0, 0, 0, 0, 0, 0);

    let mask_lo_nibble = _mm_set1_epi8(0x0f);

    // We'll be writing 16 bytes at a time, therefore we need to ensure that
    // there's still enough space in the output slice.
    // To do that, we check the remaining length of the input.
    // ceil(16 * 4 / 3) = 22
    while len >= 22 {
        // Lookup:
        // SAFETY: There's at least 16 bytes of input left.
        let src = unsafe { _mm_loadu_si128(ptr.cast()) };
        let hi_nibbles = _mm_and_si128(_mm_srli_epi32(src, 4), mask_lo_nibble);
        let lo_nibbles = _mm_and_si128(src, mask_lo_nibble);
        let lo = _mm_shuffle_epi8(lut_lo, lo_nibbles);
        let hi = _mm_shuffle_epi8(lut_hi, hi_nibbles);

        if _mm_testz_si128(lo, hi) == 0 {
            return Err("Failed to decode base64");
        }

        let roll = _mm_shuffle_epi8(lut_roll, hi_nibbles);

        // Packing:
        // source = [00dddddd|00cccccc|00bbbbbb|00aaaaaa]
        // merged = [0000dddd|ddcccccc|0000bbbb|bbaaaaaa]
        let merged = _mm_maddubs_epi16(_mm_add_epi8(src, roll), _mm_set1_epi32(0x40014001));
        // swapped = [00000000|ddddddcc|ccccbbbb|bbaaaaaa]
        let swapped = _mm_madd_epi16(merged, _mm_set1_epi32(0x10000001));
        // shuffled = [ffeeeeee|ddddddcc|ccccbbbb|bbaaaaaa]
        let shuffled = _mm_shuffle_epi8(
            swapped,
            _mm_setr_epi8(0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, -1, -1, -1, -1),
        );

        // SAFETY: As long as the caller upheld the safety contract,
        // we are at least 16 bytes away from the end of the output.
        unsafe {
            _mm_storeu_si128(out_ptr.cast(), shuffled);
            out_ptr = out_ptr.add(12);
            written += 12;

            ptr = ptr.add(16);
            len -= 16;
        }
    }

    // SAFETY: Scalar version relies on the same safety contract.
    // Slices are guaranteed to be correct as long as the caller upheld it.
    Ok(written
        + unsafe {
            scalar::decode_into_unchecked(
                core::slice::from_raw_parts(ptr, len),
                core::slice::from_raw_parts_mut(out_ptr, out_len - written),
            )
        }?)
}
