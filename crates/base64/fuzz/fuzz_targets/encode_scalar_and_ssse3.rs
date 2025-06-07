#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    use weakauras_codec_base64::encode::{arch::x86_64::ssse3, calculate_encoded_len, scalar};

    let Some(capacity) = calculate_encoded_len(data) else {
        return;
    };

    let mut scalar_output = Vec::with_capacity(capacity);
    let mut ssse3_output = Vec::with_capacity(capacity);

    unsafe {
        let scalar_written =
            scalar::encode_into_unchecked(data, scalar_output.spare_capacity_mut());
        scalar_output.set_len(scalar_written);

        let ssse3_written = ssse3::encode_into_unchecked(data, ssse3_output.spare_capacity_mut());
        ssse3_output.set_len(ssse3_written);
    }

    assert!(
        scalar_output == ssse3_output,
        "Scalar and SSSE3 implementations returned different results"
    );
});
