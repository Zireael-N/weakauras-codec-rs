// Based on code from https://github.com/aklomp/base64
// Copyright 2016-2017 Matthieu Darbois
// Copyright 2019-2024 Alfred Klomp
// Copyright 2025 Velithris
// SPDX-License-Identifier: BSD-2-Clause

use crate::decode::scalar;
use core::arch::aarch64::*;
use core::mem::MaybeUninit;

// The input consists of five valid character sets in the Base64 alphabet,
// which we need to map back to the 6-bit values they represent.
//
//   #  From       To        LUT  Characters
//   1  [0..39]    [255]      #1  invalid input
//   2  [40..41]   [62..63]   #1  (,)
//   3  [42..47]   [255]      #1  invalid input
//   4  [48..57]   [52..61]   #1  0..9
//   5  [58..63]   [255]      #1  invalid input
//   6  [64]       [255]      #2  invalid input
//   7  [65..90]   [26..51]   #2  A..Z
//   8  [91..96]   [255]      #2  invalid input
//   9  [97..122]  [0..25]    #2  a..z
//  10  [123..126] [255]      #2  invalid input
// (11) Everything else => invalid input

// The first LUT will use the VTBL instruction (out of range indices are set to
// 0 in destination).
static LUT1: [u8; 64] = [
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 62, 63, 255, 255, 255, 255, 255, 255, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 255,
    255, 255, 255, 255, 255,
];

// The second LUT will use the VTBX instruction (out of range indices will be
// unchanged in destination). Input [64..126] will be mapped to index [1..63]
// in this LUT. Index 0 means that value comes from LUT #1.
static LUT2: [u8; 64] = [
    0, 255, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47,
    48, 49, 50, 51, 255, 255, 255, 255, 255, 255, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
    15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 255, 255, 255, 255,
];

// Refer to the reexport for documentation, crate::decode::decode_into_unchecked.
#[target_feature(enable = "neon")]
pub unsafe fn decode_into_unchecked(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
) -> Result<usize, usize> {
    let mut len = input.len();
    let out_len = output.len();
    let mut written = 0;

    let mut ptr = input.as_ptr();
    let mut out_ptr = output.as_mut_ptr();

    // SAFETY: Both LUTs are 64 bytes long.
    let lut1 = unsafe { vld1q_u8_x4(&LUT1 as *const _) };
    let lut2 = unsafe { vld1q_u8_x4(&LUT2 as *const _) };

    while len >= 64 {
        let offset = vdupq_n_u8(63);

        // SAFETY: There's at least 64 bytes of input left.
        let input_data = unsafe { vld4q_u8(ptr) };

        // Get values from the first LUT:
        let t1 = uint8x16x4_t(
            vqtbl4q_u8(lut1, input_data.0),
            vqtbl4q_u8(lut1, input_data.1),
            vqtbl4q_u8(lut1, input_data.2),
            vqtbl4q_u8(lut1, input_data.3),
        );

        // Get indices for the second LUT:
        let mut t2 = uint8x16x4_t(
            vqsubq_u8(input_data.0, offset),
            vqsubq_u8(input_data.1, offset),
            vqsubq_u8(input_data.2, offset),
            vqsubq_u8(input_data.3, offset),
        );

        // Get values from the second LUT:
        t2.0 = vqtbx4q_u8(t2.0, lut2, t2.0);
        t2.1 = vqtbx4q_u8(t2.1, lut2, t2.1);
        t2.2 = vqtbx4q_u8(t2.2, lut2, t2.2);
        t2.3 = vqtbx4q_u8(t2.3, lut2, t2.3);

        // Get final values
        let t3 = uint8x16x4_t(
            vorrq_u8(t1.0, t2.0),
            vorrq_u8(t1.1, t2.1),
            vorrq_u8(t1.2, t2.2),
            vorrq_u8(t1.3, t2.3),
        );

        // Check for invalid input, any value larger than 63:
        let invalid_mask = vorrq_u8(
            vcgtq_u8(t3.0, offset),
            vorrq_u8(
                vcgtq_u8(t3.1, offset),
                vorrq_u8(vcgtq_u8(t3.2, offset), vcgtq_u8(t3.3, offset)),
            ),
        );

        if vmaxvq_u8(invalid_mask) != 0 {
            // SAFETY: We were working on 64 bytes just now.
            let last_chunk = unsafe { core::slice::from_raw_parts(ptr, 64) };
            let invalid_byte_at = scalar::find_invalid_byte(last_chunk).unwrap();

            return Err(input.len() - len + invalid_byte_at);
        }

        let output_data = uint8x16x3_t(
            vorrq_u8(t3.0, vshlq_n_u8(t3.1, 6)),
            vorrq_u8(vshrq_n_u8(t3.1, 2), vshlq_n_u8(t3.2, 4)),
            vorrq_u8(vshrq_n_u8(t3.2, 4), vshlq_n_u8(t3.3, 2)),
        );

        // SAFETY: As long as the caller upheld the safety contract,
        // we are at least 48 bytes away from the end of the output.
        unsafe {
            vst3q_u8(out_ptr.cast(), output_data);
            out_ptr = out_ptr.add(48);
            written += 48;

            ptr = ptr.add(64);
            len -= 64;
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
