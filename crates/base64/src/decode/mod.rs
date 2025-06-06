// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

// No guarantees about following semver there.
// Both modules are public for benchmarks and fuzzing.
#[doc(hidden)]
pub mod arch;
#[doc(hidden)]
pub mod scalar;

pub use arch::decode_into_unchecked;
use core::mem::MaybeUninit;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[inline]
pub fn calculate_decoded_len(input: &[u8]) -> Option<usize> {
    // Equivalent to input.len() * 3 / 4 but does not overflow
    let len = input.len();

    let leftover = len % 4;
    if leftover == 1 {
        return None;
    }
    let mut result = len / 4 * 3;

    if leftover > 0 {
        result += leftover - 1;
    }

    Some(result)
}

#[cfg(feature = "alloc")]
pub fn decode_to_vec(input: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut buffer =
        Vec::with_capacity(calculate_decoded_len(input).ok_or("Invalid base64 length")?);

    // SAFETY:
    // - buffer's capacity is enough for storing decoded base64-input;
    // - decode_into_unchecked returns the amount of bytes written,
    //   thus it is safe to call set_len using its return value.
    unsafe {
        let written = decode_into_unchecked(input, buffer.spare_capacity_mut())
            .map_err(|_| "Failed to decode base64")?;
        buffer.set_len(written)
    }

    Ok(buffer)
}

pub fn decode_into(input: &[u8], output: &mut [MaybeUninit<u8>]) -> Result<usize, &'static str> {
    let required_capacity = calculate_decoded_len(input).ok_or("Invalid base64 length")?;
    if output.len() < required_capacity {
        return Err("Output slice is too small");
    }

    // SAFETY: output's len is enough to store decoded base64-input.
    unsafe { decode_into_unchecked(input, output).map_err(|_| "Failed to decode base64") }
}

#[cfg(test)]
pub(crate) mod tests;
