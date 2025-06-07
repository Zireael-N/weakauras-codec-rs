#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    use weakauras_codec_base64::decode::{arch::x86_64::sse41, calculate_decoded_len, scalar};

    let Some(capacity) = calculate_decoded_len(data) else {
        return;
    };

    let mut scalar_output = Vec::with_capacity(capacity);
    let mut sse41_output = Vec::with_capacity(capacity);

    let scalar_result =
        unsafe { scalar::decode_into_unchecked(data, scalar_output.spare_capacity_mut()) };
    let sse41_result =
        unsafe { sse41::decode_into_unchecked(data, sse41_output.spare_capacity_mut()) };

    match (scalar_result, sse41_result) {
        (Ok(scalar_written), Ok(sse41_written)) if scalar_written == sse41_written => {
            unsafe {
                scalar_output.set_len(scalar_written);
                sse41_output.set_len(sse41_written);
            }

            assert!(
                scalar_output == sse41_output,
                "Scalar and SSE4.1 implementations returned different results"
            );
        }
        (Err(scalar_invalid_byte_at), Err(sse41_invalid_byte_at))
            if scalar_invalid_byte_at == sse41_invalid_byte_at => {}
        _ => panic!("Scalar and SSE4.1 implementations returned different results"),
    }
});
