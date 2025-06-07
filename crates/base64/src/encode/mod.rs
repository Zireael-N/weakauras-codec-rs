// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

// No guarantees about following semver there.
// Both modules are public for benchmarks and fuzzing.
#[doc(hidden)]
pub mod arch;
#[doc(hidden)]
pub mod scalar;

use crate::error::{EncodeError, EncodeIntoSliceError};
/// Encode `input` as base64 into the provided slice without validating its length.
///
/// Returns the amount of bytes written. Written bytes are guaranteed to be ASCII.
///
/// # Safety
///
/// * `output`'s length must be AT LEAST `(input.len() * 4 + 2) / 3`.
///
/// # Example
///
/// ```
/// use weakauras_codec_base64::encode;
///
/// let input = b"Hello, world!";
/// let required_capacity = encode::calculate_encoded_len(input).unwrap();
/// let mut output = Vec::with_capacity(required_capacity);
///
/// // SAFETY:
/// // - buffer's capacity is enough for storing base64-encoded input;
/// // - encode_into_unchecked returns the amount of bytes written,
/// //   thus it is safe to call set_len using its return value.
/// unsafe {
///     let bytes_written = encode::encode_into_unchecked(input, output.spare_capacity_mut());
///     output.set_len(bytes_written);
/// }
///
/// assert_eq!(output, b"ivgBS9glGC3BYXgzHa");
/// ```
pub use arch::encode_into_unchecked;
use core::mem::MaybeUninit;

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

/// Calculate the amount of bytes required to store `input`
/// after encoding it as base64.
///
/// `None` indicates an overflow.
///
/// # Example
///
/// ```
/// use weakauras_codec_base64::encode;
///
/// assert_eq!(encode::calculate_encoded_len(b"Hello, world!").unwrap(), 18);
/// ```
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

/// Encode `input` as base64 into a new `String` with the supplied prefix.
///
/// # Example
///
/// ```
/// use weakauras_codec_base64::{encode::encode_to_string_with_prefix, error::EncodeError};
///
/// fn main() -> Result<(), EncodeError> {
///     assert_eq!(
///         encode_to_string_with_prefix(b"Hello, world!", "!WA:2!")?,
///         "!WA:2!ivgBS9glGC3BYXgzHa"
///     );
///     Ok(())
/// }
/// ```
#[cfg(feature = "alloc")]
pub fn encode_to_string_with_prefix(input: &[u8], prefix: &str) -> Result<String, EncodeError> {
    let mut buffer = Vec::with_capacity(
        calculate_encoded_len(input)
            .and_then(|len| len.checked_add(prefix.len()))
            .ok_or(EncodeError::DataIsTooLarge)?,
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

/// Encode `input` as base64 into a new `String`.
///
/// # Example
///
/// ```
/// use weakauras_codec_base64::{encode::encode_to_string, error::EncodeError};
///
/// fn main() -> Result<(), EncodeError> {
///     assert_eq!(encode_to_string(b"Hello, world!")?, "ivgBS9glGC3BYXgzHa");
///     Ok(())
/// }
/// ```
#[cfg(feature = "alloc")]
pub fn encode_to_string(input: &[u8]) -> Result<String, EncodeError> {
    let mut buffer =
        Vec::with_capacity(calculate_encoded_len(input).ok_or(EncodeError::DataIsTooLarge)?);

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

/// Encode `input` as base64 into the provided slice.
///
/// Returns the amount of bytes written. Written bytes are guaranteed to be ASCII.
///
/// # Example
///
/// ```
/// use weakauras_codec_base64::{encode, error::EncodeIntoSliceError};
///
/// fn main() -> Result<(), EncodeIntoSliceError> {
///     let input = b"Hello, world!";
///     let required_capacity = encode::calculate_encoded_len(input).unwrap();
///     let mut output = Vec::with_capacity(required_capacity);
///
///     let bytes_written = encode::encode_into(input, output.spare_capacity_mut())?;
///     unsafe {
///         output.set_len(bytes_written);
///     }
///     assert_eq!(output, b"ivgBS9glGC3BYXgzHa");
///     Ok(())
/// }
/// ```
pub fn encode_into(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
) -> Result<usize, EncodeIntoSliceError> {
    let required_capacity = calculate_encoded_len(input).ok_or(EncodeError::DataIsTooLarge)?;
    if output.len() < required_capacity {
        return Err(EncodeIntoSliceError::OutputSliceIsTooSmall);
    }

    // SAFETY: output's len is enough to store base64-encoded input.
    Ok(unsafe { encode_into_unchecked(input, output) })
}

#[cfg(test)]
pub(crate) mod tests;
