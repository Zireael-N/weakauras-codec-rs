// Copyright 2020-2025 Velithris
// SPDX-License-Identifier: MIT

// No guarantees about following semver there.
// Both modules are public for benchmarks and fuzzing.
#[doc(hidden)]
pub mod arch;
#[doc(hidden)]
pub mod scalar;

use crate::error::{DecodeError, DecodeIntoSliceError};
/// Decode base64-encoded `input` into the provided slice without validating its length.
///
/// On success `Result::Ok` containing the amount of bytes written is returned.
/// Otherwise, `Result::Err` containing the offset of the first invalid input byte is returned.
///
/// # Safety
///
/// * `output`'s length must be AT LEAST `input.len() * 3 / 4`
///
/// # Example
///
/// ```
/// use weakauras_codec_base64::decode;
///
/// let input = b"ivgBS9glGC3BYXgzHa";
/// let required_capacity = decode::calculate_decoded_len(input).unwrap();
/// let mut output = Vec::with_capacity(required_capacity);
///
/// // SAFETY:
/// // - buffer's capacity is enough for storing decoded base64-input;
/// // - decode_into_unchecked returns the amount of bytes written,
/// //   thus it is safe to call set_len using its return value.
/// unsafe {
///     let bytes_written =
///         decode::decode_into_unchecked(input, output.spare_capacity_mut()).unwrap();
///     output.set_len(bytes_written);
/// }
///
/// assert_eq!(output, b"Hello, world!");
/// ```
pub use arch::decode_into_unchecked;
use core::mem::MaybeUninit;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Calculate the amount of bytes required to store base64-encoded `input`
/// after decoding it.
///
/// `None` indicates an invalid length of the `input`.
///
/// # Example
///
/// ```
/// use weakauras_codec_base64::decode;
///
/// assert_eq!(
///     decode::calculate_decoded_len(b"ivgBS9glGC3BYXgzHa").unwrap(),
///     13
/// );
/// ```
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

/// Decode base64-encoded `input` into a new `Vec<u8>`.
///
/// # Example
///
/// ```
/// use weakauras_codec_base64::{decode::decode_to_vec, error::DecodeError};
///
/// fn main() -> Result<(), DecodeError> {
///     assert_eq!(decode_to_vec(b"ivgBS9glGC3BYXgzHa")?, b"Hello, world!");
///     Ok(())
/// }
/// ```
#[cfg(feature = "alloc")]
pub fn decode_to_vec(input: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let mut buffer =
        Vec::with_capacity(calculate_decoded_len(input).ok_or(DecodeError::InvalidLength)?);

    // SAFETY:
    // - buffer's capacity is enough for storing decoded base64-input;
    // - decode_into_unchecked returns the amount of bytes written,
    //   thus it is safe to call set_len using its return value.
    unsafe {
        let written = decode_into_unchecked(input, buffer.spare_capacity_mut())
            .map_err(DecodeError::InvalidByte)?;
        buffer.set_len(written)
    }

    Ok(buffer)
}

/// Decode base64-encoded `input` into the provided slice.
///
/// Returns the amount of bytes written.
///
/// # Example
///
/// ```
/// use weakauras_codec_base64::{decode, error::DecodeIntoSliceError};
///
/// fn main() -> Result<(), DecodeIntoSliceError> {
///     let input = b"ivgBS9glGC3BYXgzHa";
///     let required_capacity = decode::calculate_decoded_len(input).unwrap();
///     let mut output = Vec::with_capacity(required_capacity);
///
///     let bytes_written = decode::decode_into(input, output.spare_capacity_mut())?;
///     unsafe {
///         output.set_len(bytes_written);
///     }
///     assert_eq!(output, b"Hello, world!");
///     Ok(())
/// }
/// ```
pub fn decode_into(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
) -> Result<usize, DecodeIntoSliceError> {
    let required_capacity = calculate_decoded_len(input).ok_or(DecodeError::InvalidLength)?;
    if output.len() < required_capacity {
        return Err(DecodeIntoSliceError::OutputSliceIsTooSmall);
    }

    // SAFETY: output's len is enough to store decoded base64-input.
    Ok(unsafe { decode_into_unchecked(input, output).map_err(DecodeError::InvalidByte)? })
}

#[cfg(test)]
pub(crate) mod tests;
