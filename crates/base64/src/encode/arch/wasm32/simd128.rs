// Based on a research done by Wojciech Muła and Daniel Lemire
// https://arxiv.org/abs/1704.00605
// Copyright 2015-2016 Wojciech Muła, Alfred Klomp, Daniel Lemire
// Copyright 2025 Velithris
// SPDX-License-Identifier: BSD-2-Clause

use crate::encode::scalar;
use core::arch::wasm32::*;
use core::mem::MaybeUninit;

// Refer to the reexport for documentation, crate::encode::encode_into_unchecked.
#[target_feature(enable = "simd128")]
pub unsafe fn encode_into_unchecked(input: &[u8], output: &mut [MaybeUninit<u8>]) -> usize {
    // Refer to the SSSE3 version for a detailed explanation,
    // only unpacking is different.

    let mut len = input.len();
    let out_len = output.len();
    let mut written = 0;

    let mut ptr = input.as_ptr();
    let mut out_ptr = output.as_mut_ptr();

    let shuf = u8x16(1, 2, 0, 1, 4, 5, 3, 4, 7, 8, 6, 7, 10, 11, 9, 10);

    while len >= 16 {
        // input = [ffeeeeee|ddddddcc|ccccbbbb|bbaaaaaa]

        // SAFETY: There's at least 16 bytes of input left.
        // src = [ccccbbbb|bbaaaaaa|ddddddcc|ccccbbbb]
        let src = u8x16_swizzle(unsafe { v128_load(ptr.cast()) }, shuf);

        // [00000000|00000000|00000000|00aaaaaa]
        let index_a = v128_and(u32x4_shr(src, 16), u32x4_splat(0x0000003f));
        // [00000000|00000000|00bbbbbb|00000000]
        let index_b = v128_and(u32x4_shr(src, 14), u32x4_splat(0x00003f00));
        // [00000000|00cccccc|00000000|00000000]
        let index_c = v128_and(u32x4_shl(src, 12), u32x4_splat(0x003f0000));
        // [00dddddd|00000000|00000000|00000000]
        let index_d = v128_and(u32x4_shl(src, 14), u32x4_splat(0x3f000000));

        let indices = v128_or(index_a, v128_or(index_b, v128_or(index_c, index_d)));

        let mut result = v128_or(
            u8x16_sub_sat(indices, u8x16_splat(51)),
            v128_and(u8x16_gt(u8x16_splat(26), indices), u8x16_splat(13)),
        );

        let offsets = i8x16(
            39, -4, -4, -4, -4, -4, -4, -4, -4, -4, -4, -22, -22, 97, 0, 0,
        );

        result = i8x16_add(u8x16_swizzle(offsets, result), indices);

        // SAFETY: As long as the caller upheld the safety contract,
        // we are at least 16 bytes away from the end of the output.
        unsafe {
            v128_store(out_ptr.cast(), result);
            out_ptr = out_ptr.add(16);
            written += 16;

            ptr = ptr.add(12);
            len -= 12;
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
