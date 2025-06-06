#![no_main]
use libfuzzer_sys::fuzz_target;
use weakauras_codec_ace_serialize::{Deserializer, Serializer};

fuzz_target!(|data: &[u8]| {
    if let Ok(data) = core::str::from_utf8(data) {
        if let Ok(Some(value)) = Deserializer::from_str(data).deserialize_first() {
            // No reason to compare with the original data, because same numbers
            // can be encoded in different ways.
            assert!(
                Serializer::serialize_one(&value, None).is_ok(),
                "Couldn't serialize what we deserialized"
            );
        }
    }
});
