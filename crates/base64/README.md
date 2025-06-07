# weakauras-codec-base64

This library provides heavily optimized routines for decoding and encoding
base64 used for [WeakAuras]-compatible strings.

Why does this library exist?
Base64 used in WeakAuras packs bits in a non-standard way,
this makes it impossible to use existing implementations,
even if they do support custom alphabets.

## Decoding example

This is how you can use the library to decode base64-encoded data.

```rust
use weakauras_codec_base64::{DecodeError, decode_to_vec};

fn main() -> Result<(), DecodeError> {
    assert_eq!(decode_to_vec(b"ivgBS9glGC3BYXgzHa")?, b"Hello, world!");
    Ok(())
}
```

## Encoding example

This is how you can use the library to encode data as base64.

```rust
use weakauras_codec_base64::{EncodeError, encode_to_string};

fn main() -> Result<(), EncodeError> {
    assert_eq!(encode_to_string(b"Hello, world!")?, "ivgBS9glGC3BYXgzHa");
    Ok(())
}
```

## Crate features

* **std** - Enable features that require the standard library. As of now, it's used only for runtime SIMD feature detection on x86_64 and x86 CPUs. **Enabled** by default.
* **alloc** - Enable APIs that allocate, like `decode_to_vec` and `encode_to_string`. **Enabled** by default.

[WeakAuras]: https://weakauras.wtf
