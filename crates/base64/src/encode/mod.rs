// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

#[cfg(all(
    any(feature = "avx2", test),
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
mod avx2;
mod scalar;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "ssse3"
))]
mod sse;

use core::mem::MaybeUninit;

#[cfg(all(
    feature = "expose_internals",
    any(feature = "avx2", test),
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
pub use avx2::encode_into_unchecked as encode_into_unchecked_avx2;

#[cfg(all(
    feature = "expose_internals",
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "ssse3"
))]
pub use sse::encode_into_unchecked as encode_into_unchecked_sse;

#[cfg(feature = "expose_internals")]
pub use scalar::encode_into_unchecked as encode_into_unchecked_scalar;

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

/// SAFETY: the caller must ensure that `output`'s length is AT LEAST `(input.len() * 4 + 2) / 3`
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "ssse3"
))]
#[inline(always)]
pub unsafe fn encode_into_unchecked(input: &[u8], output: &mut [MaybeUninit<u8>]) -> usize {
    unsafe { sse::encode_into_unchecked(input, output) }
}

/// SAFETY: the caller must ensure that `output`'s length is AT LEAST `(input.len() * 4 + 2) / 3`
#[cfg(any(
    not(any(target_arch = "x86", target_arch = "x86_64")),
    not(target_feature = "ssse3")
))]
#[inline(always)]
pub unsafe fn encode_into_unchecked(input: &[u8], output: &mut [MaybeUninit<u8>]) -> usize {
    unsafe { scalar::encode_into_unchecked(input, output) }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[allow(unused_macros)]
    macro_rules! base64_encode {
        ($input:expr, $output:ident, $module:ident) => {
            let buffer = $output.as_mut_vec();
            let len = $module::encode_into_unchecked($input, buffer.spare_capacity_mut());
            buffer.set_len(len);
        };
    }

    #[test]
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "ssse3"
    ))]
    fn scalar_and_sse_return_same_values() {
        let data: Vec<u8> = (0..=255).cycle().take(1024 * 30 + 3).collect();

        let capacity = (data.len() * 4 + 2) / 3;
        let mut buf1 = String::with_capacity(capacity);
        let mut buf2 = String::with_capacity(capacity);

        unsafe {
            base64_encode!(&data, buf1, scalar);
            base64_encode!(&data, buf2, sse);
        }

        assert_eq!(buf1, buf2);
    }

    #[test]
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    fn scalar_and_avx2_return_same_values() {
        let data: Vec<u8> = (0..=255).cycle().take(1024 * 30 + 3).collect();

        let capacity = (data.len() * 4 + 2) / 3;
        let mut buf1 = String::with_capacity(capacity);
        let mut buf2 = String::with_capacity(capacity);

        unsafe {
            base64_encode!(&data, buf1, scalar);
            base64_encode!(&data, buf2, avx2);
        }

        assert_eq!(buf1, buf2);
    }
}
