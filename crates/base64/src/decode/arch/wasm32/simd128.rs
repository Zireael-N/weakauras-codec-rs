// Based on a research done by Wojciech Muła and Daniel Lemire
// https://arxiv.org/abs/1704.00605
// Copyright 2015-2016 Wojciech Muła, Alfred Klomp, Daniel Lemire
// Copyright 2025 Velithris
// SPDX-License-Identifier: BSD-2-Clause

use crate::decode::scalar;
use core::arch::wasm32::*;
use core::mem::MaybeUninit;

/// SAFETY: the caller must ensure that `output`'s length is AT LEAST `input.len() * 3 / 4`
#[target_feature(enable = "simd128")]
pub unsafe fn decode_into_unchecked(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
) -> Result<usize, usize> {
    // Refer to the SSE4.1 version for a detailed explanation,
    // only packing is different.

    let mut len = input.len();
    let out_len = output.len();
    let mut written = 0;

    let mut ptr = input.as_ptr();
    let mut out_ptr = output.as_mut_ptr();

    let lut_hi = u8x16(
        0x10, 0x10, 0x01, 0x02, 0x04, 0x08, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
        0x10,
    );
    let lut_lo = u8x16(
        0x15, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x10, 0x10, 0x13, 0x1b, 0x1b, 0x1b, 0x1b,
        0x1b,
    );
    let lut_roll = i8x16(0, 22, 22, 4, -39, -39, -97, -97, 0, 0, 0, 0, 0, 0, 0, 0);

    let mask_lo_nibble = u8x16_splat(0x0f);

    // We'll be writing 16 bytes at a time, therefore we need to ensure that
    // there's still enough space in the output slice.
    // To do that, we check the remaining length of the input.
    // ceil(16 * 4 / 3) = 22
    while len >= 22 {
        // Lookup:
        // SAFETY: There's at least 16 bytes of input left.
        let src = unsafe { v128_load(ptr.cast()) };
        let hi_nibbles = v128_and(u32x4_shr(src, 4), mask_lo_nibble);
        let lo_nibbles = v128_and(src, mask_lo_nibble);
        let lo = u8x16_swizzle(lut_lo, lo_nibbles);
        let hi = u8x16_swizzle(lut_hi, hi_nibbles);

        if v128_any_true(v128_and(lo, hi)) {
            // SAFETY: We were working on 16 bytes just now.
            let last_chunk = unsafe { core::slice::from_raw_parts(ptr, 128 / 8) };
            let invalid_byte_at = scalar::find_invalid_byte(last_chunk).unwrap();

            return Err(input.len() - len + invalid_byte_at);
        }

        let roll = u8x16_swizzle(lut_roll, hi_nibbles);

        // Packing:
        // [00dddddd|00cccccc|00bbbbbb|00aaaaaa]
        let src = i8x16_add(src, roll);

        // [00000000|00cccccc|00000000|00aaaaaa]
        let mask_ac = u16x8_splat(0x003f);
        let ac = v128_and(src, mask_ac);

        // [00dddddd|00000000|00bbbbbb|00000000]
        let mask_db = u16x8_splat(0x3f00);
        let db = v128_and(src, mask_db);

        // [0000dddd|ddcccccc|0000bbbb|bbaaaaaa]
        let t0 = v128_or(u32x4_shr(db, 2), ac);

        // [00000000|ddddddcc|ccccbbbb|bbaaaaaa]
        let t1 = v128_or(
            u32x4_shr(v128_and(t0, u32x4_splat(0xffff0000)), 4),
            v128_and(t0, u32x4_splat(0x0000ffff)),
        );

        let result = u8x16_swizzle(
            t1,
            i8x16(0, 1, 2, 4, 5, 6, 8, 9, 10, 12, 13, 14, -1, -1, -1, -1),
        );

        // SAFETY: As long as the caller upheld the safety contract,
        // we are at least 16 bytes away from the end of the output.
        unsafe {
            v128_store(out_ptr.cast(), result);
            out_ptr = out_ptr.add(12);
            written += 12;

            ptr = ptr.add(16);
            len -= 16;
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
