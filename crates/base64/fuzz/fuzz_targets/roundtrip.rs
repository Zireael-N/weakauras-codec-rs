#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(encoded) = weakauras_codec_base64::encode_to_string(data) {
        let decoded = weakauras_codec_base64::decode_to_vec(encoded.as_bytes())
            .expect("Failed to decode what we've encoded");
        assert!(decoded == data, "Decoded data differs from the input");
    }
});
