#![no_main]
use libfuzzer_sys::fuzz_target;

// Fuzz this with a timeout, there's an infinite loop:
// cargo +nightly fuzz run decompression --release -- -timeout=10
fuzz_target!(|data: &[u8]| {
    let _ = weakauras_codec_lib_compress::decompress(data);
});
