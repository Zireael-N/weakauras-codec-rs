#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    use weakauras_codec_base64::encode::{arch::aarch64::neon, calculate_encoded_len, scalar};

    let Some(capacity) = calculate_encoded_len(data) else {
        return;
    };

    let mut scalar_output = Vec::with_capacity(capacity);
    let mut neon_output = Vec::with_capacity(capacity);

    unsafe {
        let scalar_written =
            scalar::encode_into_unchecked(data, scalar_output.spare_capacity_mut());
        scalar_output.set_len(scalar_written);

        let neon_written = neon::encode_into_unchecked(data, neon_output.spare_capacity_mut());
        neon_output.set_len(neon_written);
    }

    assert!(
        scalar_output == neon_output,
        "Scalar and Neon implementations returned different results"
    );
});
