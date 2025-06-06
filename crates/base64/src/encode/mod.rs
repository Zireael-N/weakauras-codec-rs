// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

// No guarantees about following semver there.
// Both modules are public for benchmarks and fuzzing.
#[doc(hidden)]
pub mod arch;
#[doc(hidden)]
pub mod scalar;

pub use arch::encode_into_unchecked;
use core::mem::MaybeUninit;

const OVERFLOW_ERROR: &str = "Cannot calculate capacity without overflowing";

#[inline]
pub fn calculate_encoded_len(input: &[u8]) -> Option<usize> {
    // Equivalent to (input.len() * 4 + 2) / 3 but avoids an early overflow
    let len = input.len();
    let leftover = len % 3;

    (len / 3).checked_mul(4).and_then(|len| {
        if leftover > 0 {
            len.checked_add(leftover + 1)
        } else {
            Some(len)
        }
    })
}

pub fn encode_to_string_with_prefix(input: &[u8], prefix: &str) -> Result<String, &'static str> {
    let mut buffer = Vec::with_capacity(
        calculate_encoded_len(input)
            .and_then(|len| len.checked_add(prefix.len()))
            .ok_or(OVERFLOW_ERROR)?,
    );
    buffer.extend_from_slice(prefix.as_bytes());

    // SAFETY:
    // - buffer's capacity is enough for storing both the prefix and base64-encoded input;
    // - encode_into_unchecked returns the amount of bytes written,
    //   thus it is safe to call set_len adding its return value
    //   and the prefix's length (which buffer.len() is currently equal to).
    unsafe {
        let written = encode_into_unchecked(input, buffer.spare_capacity_mut());
        buffer.set_len(buffer.len() + written);
    }

    // SAFETY:
    // - prefix is guaranteed to be valid UTF-8, since it is &str;
    // - encode_into_unchecked writes exclusively ASCII bytes.
    let result = unsafe { String::from_utf8_unchecked(buffer) };
    Ok(result)
}

pub fn encode_to_string(input: &[u8]) -> Result<String, &'static str> {
    let mut buffer = Vec::with_capacity(calculate_encoded_len(input).ok_or(OVERFLOW_ERROR)?);

    // SAFETY:
    // - buffer's capacity is enough for storing base64-encoded input;
    // - encode_into_unchecked returns the amount of bytes written,
    //   thus it is safe to call set_len using its return value.
    unsafe {
        let written = encode_into_unchecked(input, buffer.spare_capacity_mut());
        buffer.set_len(written);
    }

    // SAFETY: encode_into_unchecked writes exclusively ASCII bytes.
    let result = unsafe { String::from_utf8_unchecked(buffer) };
    Ok(result)
}

pub fn encode_into(input: &[u8], output: &mut [MaybeUninit<u8>]) -> Result<usize, &'static str> {
    let required_capacity = calculate_encoded_len(input).ok_or(OVERFLOW_ERROR)?;
    if output.len() < required_capacity {
        return Err("Output slice is too small");
    }

    // SAFETY: output's len is enough to store base64-encoded input.
    Ok(unsafe { encode_into_unchecked(input, output) })
}

#[cfg(test)]
pub(crate) mod tests;
