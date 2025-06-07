#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    use weakauras_codec_base64::decode::{arch::aarch64::neon, calculate_decoded_len, scalar};

    let Some(capacity) = calculate_decoded_len(data) else {
        return;
    };

    let mut scalar_output = Vec::with_capacity(capacity);
    let mut neon_output = Vec::with_capacity(capacity);

    let scalar_result =
        unsafe { scalar::decode_into_unchecked(data, scalar_output.spare_capacity_mut()) };
    let neon_result =
        unsafe { neon::decode_into_unchecked(data, neon_output.spare_capacity_mut()) };

    match (scalar_result, neon_result) {
        (Ok(scalar_written), Ok(neon_written)) if scalar_written == neon_written => {
            unsafe {
                scalar_output.set_len(scalar_written);
                neon_output.set_len(neon_written);
            }

            assert!(
                scalar_output == neon_output,
                "Scalar and Neon implementations returned different results"
            );
        }
        (Err(scalar_invalid_byte_at), Err(neon_invalid_byte_at))
            if scalar_invalid_byte_at == neon_invalid_byte_at => {}
        _ => panic!("Scalar and Neon implementations returned different results"),
    }
});
