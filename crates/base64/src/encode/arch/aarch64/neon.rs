// Based on code from https://github.com/aklomp/base64
// Copyright 2016-2017 Matthieu Darbois
// Copyright 2019-2024 Alfred Klomp
// Copyright 2025 Velithris
// SPDX-License-Identifier: BSD-2-Clause

use crate::encode::scalar;
use core::arch::aarch64::*;
use core::mem::MaybeUninit;

static LUT: [u8; 64] = [
    b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p',
    b'q', b'r', b's', b't', b'u', b'v', b'w', b'x', b'y', b'z', b'A', b'B', b'C', b'D', b'E', b'F',
    b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V',
    b'W', b'X', b'Y', b'Z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'(', b')',
];

#[inline]
#[target_feature(enable = "neon")]
fn reshuffle(v: uint8x16x3_t) -> uint8x16x4_t {
    // Input:
    // in[0]  = a7 a6 a5 a4 a3 a2 a1 a0
    // in[1]  = b7 b6 b5 b4 b3 b2 b1 b0
    // in[2]  = c7 c6 c5 c4 c3 c2 c1 c0

    // Output:
    // out[0] = 00 00 a5 a4 a3 a2 a1 a0
    // out[1] = 00 00 b3 b2 b1 b0 a7 a6
    // out[2] = 00 00 c1 c0 b7 b6 b5 b4
    // out[3] = 00 00 c7 c6 c4 c5 c3 c2

    let mut result = uint8x16x4_t(
        v.0,
        vshrq_n_u8(v.0, 6),
        vshrq_n_u8(v.1, 4),
        vshrq_n_u8(v.2, 2),
    );
    result.1 = vsliq_n_u8(result.1, v.1, 2);
    result.2 = vsliq_n_u8(result.2, v.2, 4);

    result.0 = vandq_u8(result.0, vdupq_n_u8(0x3F));
    result.1 = vandq_u8(result.1, vdupq_n_u8(0x3F));
    result.2 = vandq_u8(result.2, vdupq_n_u8(0x3F));

    result
}

// Refer to the reexport for documentation, crate::encode::encode_into_unchecked.
#[target_feature(enable = "neon")]
pub unsafe fn encode_into_unchecked(input: &[u8], output: &mut [MaybeUninit<u8>]) -> usize {
    let mut len = input.len();
    let out_len = output.len();
    let mut written = 0;

    let mut ptr = input.as_ptr();
    let mut out_ptr = output.as_mut_ptr();

    // SAFETY: LUT is 64 bytes long.
    let lut = unsafe { vld1q_u8_x4(&LUT as *const _) };

    while len >= 48 {
        // SAFETY: There's at least 48 bytes of input left.
        let input_data = unsafe { vld3q_u8(ptr) };
        let mut output_data = reshuffle(input_data);

        output_data.0 = vqtbl4q_u8(lut, output_data.0);
        output_data.1 = vqtbl4q_u8(lut, output_data.1);
        output_data.2 = vqtbl4q_u8(lut, output_data.2);
        output_data.3 = vqtbl4q_u8(lut, output_data.3);

        // SAFETY: As long as the caller upheld the safety contract,
        // we are at least 64 bytes away from the end of the output.
        unsafe {
            vst4q_u8(out_ptr.cast(), output_data);
            out_ptr = out_ptr.add(64);
            written += 64;

            ptr = ptr.add(48);
            len -= 48;
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
