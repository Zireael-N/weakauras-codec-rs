#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    use weakauras_codec_base64::decode::{arch::x86_64::avx2, calculate_decoded_len, scalar};

    let Some(capacity) = calculate_decoded_len(data) else {
        return;
    };

    let mut scalar_output = Vec::with_capacity(capacity);
    let mut avx2_output = Vec::with_capacity(capacity);

    let scalar_result =
        unsafe { scalar::decode_into_unchecked(data, scalar_output.spare_capacity_mut()) };
    let avx2_result =
        unsafe { avx2::decode_into_unchecked(data, avx2_output.spare_capacity_mut()) };

    match (scalar_result, avx2_result) {
        (Ok(scalar_written), Ok(avx2_written)) if scalar_written == avx2_written => {
            unsafe {
                scalar_output.set_len(scalar_written);
                avx2_output.set_len(avx2_written);
            }

            assert!(
                scalar_output == avx2_output,
                "Scalar and AVX2 implementations returned different results"
            );
        }
        (Err(scalar_invalid_byte_at), Err(avx2_invalid_byte_at))
            if scalar_invalid_byte_at == avx2_invalid_byte_at => {}
        _ => panic!("Scalar and AVX2 implementations returned different results"),
    }
});
