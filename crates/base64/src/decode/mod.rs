#[cfg(all(
    any(feature = "avx2", test),
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
mod avx2;
mod scalar;
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse4.1"
))]
mod sse;

use core::mem::MaybeUninit;

#[cfg(all(
    feature = "expose_internals",
    any(feature = "avx2", test),
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
pub use avx2::decode_into_unchecked as decode_into_unchecked_avx2;

#[cfg(all(
    feature = "expose_internals",
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse4.1"
))]
pub use sse::decode_into_unchecked as decode_into_unchecked_sse;

#[cfg(feature = "expose_internals")]
pub use scalar::decode_into_unchecked as decode_into_unchecked_scalar;

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

pub fn decode_to_vec(input: &[u8]) -> Result<Vec<u8>, &'static str> {
    let mut buffer =
        Vec::with_capacity(calculate_decoded_len(input).ok_or("Invalid base64 length")?);

    // SAFETY:
    // - buffer's capacity is enough for storing decoded base64-input;
    // - decode_into_unchecked returns the amount of bytes written,
    //   thus it is safe to call set_len using its return value.
    unsafe {
        let written = decode_into_unchecked(input, buffer.spare_capacity_mut())?;
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
    unsafe { decode_into_unchecked(input, output) }
}

/// SAFETY: the caller must ensure that `output`'s length is AT LEAST `input.len() * 3 / 4`
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse4.1"
))]
#[inline(always)]
pub unsafe fn decode_into_unchecked(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
) -> Result<usize, &'static str> {
    unsafe { sse::decode_into_unchecked(input, output) }
}

/// SAFETY: the caller must ensure that `output`'s length is AT LEAST `input.len() * 3 / 4`
#[cfg(any(
    not(any(target_arch = "x86", target_arch = "x86_64")),
    not(target_feature = "sse4.1")
))]
#[inline(always)]
pub unsafe fn decode_into_unchecked(
    input: &[u8],
    output: &mut [MaybeUninit<u8>],
) -> Result<usize, &'static str> {
    unsafe { scalar::decode_into_unchecked(input, output) }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[allow(dead_code)]
    fn base64_iter() -> impl Iterator<Item = u8> {
        (b'a'..=b'z')
            .chain(b'A'..=b'Z')
            .chain(b'0'..=b'9')
            .chain(b'('..=b')')
            .cycle()
    }

    #[test]
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "sse4.1"
    ))]
    fn scalar_and_sse_return_same_values() {
        let data: Vec<u8> = base64_iter().take(1024 * 1024 + 3).collect();

        let capacity = data.len() * 3 / 4;
        let mut buf1 = Vec::with_capacity(capacity);
        let mut buf2 = Vec::with_capacity(capacity);

        unsafe {
            let scalar_len =
                scalar::decode_into_unchecked(&data, buf1.spare_capacity_mut()).unwrap();
            buf1.set_len(scalar_len);

            let sse_len = sse::decode_into_unchecked(&data, buf2.spare_capacity_mut()).unwrap();
            buf2.set_len(sse_len);
        }

        assert_eq!(buf1, buf2);
    }

    #[test]
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    fn scalar_and_avx2_return_same_values() {
        let data: Vec<u8> = base64_iter().take(1024 * 1024 + 3).collect();

        let capacity = data.len() * 3 / 4;
        let mut buf1 = Vec::with_capacity(capacity);
        let mut buf2 = Vec::with_capacity(capacity);

        unsafe {
            let scalar_len =
                scalar::decode_into_unchecked(&data, buf1.spare_capacity_mut()).unwrap();
            buf1.set_len(scalar_len);

            let avx2_len = avx2::decode_into_unchecked(&data, buf2.spare_capacity_mut()).unwrap();
            buf2.set_len(avx2_len);
        }

        assert_eq!(buf1, buf2);
    }
}
