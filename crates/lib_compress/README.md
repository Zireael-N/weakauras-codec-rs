# weakauras-codec-lib-compress

This library provides a routine for decompressing data
compressed by a Lua library called LibCompress.

## Example

```rust
use weakauras_codec_lib_compress::{DecompressionError, decompress};

fn main() -> Result<(), DecompressionError> {
    let expected = b"aaaaaaaa bbbbbbbb cccccccc";

    // Huffman code
    assert_eq!(
        &*decompress(
            &[
                0x03, 0x03, 0x1a, 0x00, 0x00, 0x62, 0x0c, 0x52, 0x8f,
                0xe9, 0xb0, 0x5c, 0x55, 0x35, 0x00, 0xc0, 0xaa, 0xaa
            ],
            1024
        )?,
        expected
    );

    // Uncompressed
    assert_eq!(
        &*decompress(b"\x01aaaaaaaa bbbbbbbb cccccccc", 1024)?,
        expected
    );

    Ok(())
}
```
