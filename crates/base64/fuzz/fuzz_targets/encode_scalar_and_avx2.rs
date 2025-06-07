#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    use weakauras_codec_base64::encode::{arch::x86_64::avx2, calculate_encoded_len, scalar};

    let Some(capacity) = calculate_encoded_len(data) else {
        return;
    };

    let mut scalar_output = Vec::with_capacity(capacity);
    let mut avx2_output = Vec::with_capacity(capacity);

    unsafe {
        let scalar_written =
            scalar::encode_into_unchecked(data, scalar_output.spare_capacity_mut());
        scalar_output.set_len(scalar_written);

        let avx2_written = avx2::encode_into_unchecked(data, avx2_output.spare_capacity_mut());
        avx2_output.set_len(avx2_written);
    }

    assert!(
        scalar_output == avx2_output,
        "Scalar and AVX2 implementations returned different results"
    );
});
