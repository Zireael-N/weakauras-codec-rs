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
fn calculate_encoded_len(input: &[u8]) -> Option<usize> {
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
    let mut result = String::with_capacity(
        calculate_encoded_len(input)
            .and_then(|len| len.checked_add(prefix.len()))
            .ok_or(OVERFLOW_ERROR)?,
    );
    result.push_str(prefix);

    unsafe {
        encode_into_unchecked(input, &mut result);
    }

    Ok(result)
}

pub fn encode_to_string(input: &[u8]) -> Result<String, &'static str> {
    let mut result = String::with_capacity(calculate_encoded_len(input).ok_or(OVERFLOW_ERROR)?);

    unsafe {
        encode_into_unchecked(input, &mut result);
    }

    Ok(result)
}

/// SAFETY: the caller must ensure that `output` can hold AT LEAST `(input.len() * 4 + 2) / 3` more elements
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "ssse3"
))]
#[inline(always)]
unsafe fn encode_into_unchecked(input: &[u8], output: &mut String) {
    unsafe {
        sse::encode_into_unchecked(input, output);
    }
}

/// SAFETY: the caller must ensure that `output` can hold AT LEAST `(input.len() * 4 + 2) / 3` more elements
#[cfg(any(
    not(any(target_arch = "x86", target_arch = "x86_64")),
    not(target_feature = "ssse3")
))]
#[inline(always)]
unsafe fn encode_into_unchecked(input: &[u8], output: &mut String) {
    unsafe {
        scalar::encode_into_unchecked(input, output);
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

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
            scalar::encode_into_unchecked(&data, &mut buf1);
            sse::encode_into_unchecked(&data, &mut buf2);
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
            scalar::encode_into_unchecked(&data, &mut buf1);
            avx2::encode_into_unchecked(&data, &mut buf2);
        }

        assert_eq!(buf1, buf2);
    }
}
